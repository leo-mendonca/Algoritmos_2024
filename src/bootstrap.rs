use std::cmp::{max, min};
use rand::random;
use ndarray::{range, Array2, ErrorKind, ShapeError};
use ndarray;
use plotters::prelude::*;
use std::sync::{mpsc,};
use crossbeam;
use Algoritmos::tabela_chave_valor::ConjuntoIteravel;

const ALTURA:u32 = 1050;
const LARGURA:u32 = 1680;
const D_HASH:u64 = 1000;

fn gerar_estado_inicial(p: f64, n1: usize, n2: usize) -> Array2<bool> {
    let mut tabuleiro = Vec::<bool>::new();
    // let mut tabuleiro = Array2::;
    // Array2::zeros()
    for _i in 0..n1 {
        // tabuleiro.push(Vec::<bool>::new());
        for _j in 0..n2 {
            let x = random::<f64>();
            // if x>=p {tabuleiro[i].push(false)} //Celula nao infectada
            // else {tabuleiro[i].push(true)} //Celula infectada com porbabilidade p
            tabuleiro.push(x<p);
            // tabuleiro[[i,j]] = x<p //Celula infectada com probabilidade p

        }
    }
    Array2::from_shape_vec((n1,n2), tabuleiro).expect("Saída será um array de ordem 2 e dimensões n1×n2")
    // tabuleiro
}

fn checar_propagacao(tabuleiro: &Array2<bool>, fam_update:&FamiliaUpdate, i: usize, j: usize) -> bool {
    //Avalia se a infecção se propaga para a célula (i,j)
    let &[n1,n2] = tabuleiro.shape() else { panic!("O shape de um vetor 2D tem que ter 2 elementos") };
    for conjunto in fam_update.v.iter() {
        let mut infectado = true;
        'loop_int: for elemento in conjunto.iter() {
            let (x,y) = (elemento[0]+i as i32, elemento[1] + j as i32);
            if 0<=x && x<(n1 as i32) && 0<=y && y<(n2 as i32) { //Se estivermos dentro dos limites do array, checamos a infecção
                if ! tabuleiro[[x as usize,y as usize]] { //Se a celula (x,y) não está infectada, paramos de conferir esse conjunto da família Update
                    infectado=false;
                    break 'loop_int
                }
            }
            // if let Some(true) = tabuleiro.get([i+elemento[0], j + elemento[1]]) {
            //     //pass
            // }
            else {
                infectado=false;
                break 'loop_int
            }
        }
        if infectado {return true}
    }
    return false
}

fn array_para_vec<T:Clone+Copy>(array:&ndarray::Array2<T>) ->Vec<Vec<T>> {
    let mut v = Vec::<Vec<T>>::new();
    let &[n1,n2] = array.shape() else { panic!("O array deve ter dimensao 2") };
    for i in 0..n1 {
        v.push(Vec::<T>::new());
        for j in 0..n2 {
            v[i].push(array[[i,j]])
        }
    }
    v
}

fn passo_sub(n:usize, tabuleiro: &Array2<bool>, familia_update: &FamiliaUpdate, i1:usize, i2:usize, j1:usize, j2:usize) -> Result<(Array2<bool>, bool), ShapeError> {
    //Executa um passo do processo, mas só no sub-tabuleiro retangular com extremidades (i1,j1) e (i2,j2)
    //Retorna um array (i2-i1)×(j2-j1) indicando o estado atualizado do sub-tabuleiro
    //Também retorna um bool indicando se o sub-tabuleiro está igual ao passo anterior
    if ! (i1<i2 && i2<=n)
        | !  (j1<j2 && j2<=n) {
        return Err(ShapeError::from_kind(ErrorKind::OutOfBounds))
    };
    let mut configuracao_final = true; //Vira false se qualquer celula for atualizada
    let mut sub_vec = Vec::<bool>::new();
    for i in i1..i2 {
        for j in j1..j2 {
            if  tabuleiro[[i,j]] { sub_vec.push(true) } //Se está infectado, salvamos como infectado
            else { //Se a célula não está infectada, verificamos a propagação da infecção
                if checar_propagacao(tabuleiro, familia_update, i, j) {
                    configuracao_final = false; //Se alguma infecção se propagar, então não estamos na configuração final
                    sub_vec.push(true);
                }
                else { sub_vec.push(false) }
            }
        }
    }

    let sub_tab = ndarray::Array2::from_shape_vec((i2-i1,j2-j1), sub_vec);
    match sub_tab {
        Ok(array) => {Ok((array, configuracao_final))}
        Err(e) => {Err(e)}
    }
}

#[derive(Clone)]
struct FamiliaUpdate {
    //Conjunto de conjuntos X \in Z^2 tal que, se y+X está infectado, então y se torna infectado
    v: Vec<Vec<[i32;2]>>
}

struct ProcessoBootstrap {
    // tabuleiro: Vec<bool>,
    tabuleiro: Array2<bool>,
    fam_update: FamiliaUpdate,
    n:usize,
    t: u32,
    config_final: bool,
}

impl ProcessoBootstrap {
    fn novo(n: usize, p:f64, fam_update: FamiliaUpdate) -> Self {
        let tabuleiro = gerar_estado_inicial(p,n,n);
        ProcessoBootstrap{
            tabuleiro,
            fam_update,
            n,
            t:0,
            config_final: false
        }
    }
    fn atualiza_passo(self: &mut Self) {
        //Versão incial "burra", sem paralelização
        if self.config_final {return;} //Se estamos no final, não há o que fazer
        let (tab_novo,config_final) = passo_sub(self.n,&self.tabuleiro, &self.fam_update, 0, self.n,0,self.n)
            .unwrap();
        self.tabuleiro = tab_novo;
        self.t+=1;
        self.config_final = config_final;
    }

    fn passo_paralelo(self: &mut Self, n_threads: usize) {
        //Atualiza um passo do processo de bootstrap paralelizando o processamnto entre threads diferentes
        //Divide o tabuleiro em faixas horizontais e associa cada uma a uma thread própria
        if self.config_final {return;} //Se estamos no final, não há o que fazer
        let mut config_final: bool = true; //Indica se estamos na configuracao final
        let n_threads = min(self.n,n_threads);
        let tamanho_thread = (self.n+n_threads-1)/n_threads;
        let n = self.n;
        let mut novo_tabuleiro = self.tabuleiro.clone();
        // let mut senders = Vec::new();
        let mut receivers = Vec::new();
        let mut handles = Vec::new();
        crossbeam::scope(|scope| {
            for i0 in (0..self.n).step_by(tamanho_thread) {
                let i1 = min(i0+tamanho_thread, self.n); //Não podemos passar do limite do tabuleiro
                let (tx,rx) = mpsc::channel();
                // senders.push(tx);
                receivers.push(rx);
                // let ref_self: &ProcessoBootstrap = self;
                let ( tabuleiro, familia_update) = (&self.tabuleiro, &self.fam_update);

                handles.push(scope.spawn(move || {
                    let (tab_novo, config_estavel) = passo_sub(n,tabuleiro, familia_update,i0,i1, 0, n).expect("Erro no passo do sub-tabuleiro");
                    tx.send((tab_novo,config_estavel)).expect("Não podemos ter erro de comunicação entre as threads");
                }));
            }
            for (i0, rx) in (0..n).step_by(tamanho_thread).zip(receivers.iter()) {
                let i1 = min(i0+tamanho_thread, n);
                let (sub_tabuleiro, config_estavel) = rx.recv().expect("Erro na comunicação entre threads");
                if ! config_estavel {config_final = false}
                let mut fatia = novo_tabuleiro.slice_mut(ndarray::s![i0..i1,0..n]);
                fatia.assign(&sub_tabuleiro);
            }
        });
        for h in handles.into_iter() {h.join()};
        self.tabuleiro = novo_tabuleiro;
        self.config_final = config_final;
        self.t+=1;

    }



    fn plotar(self: &Self, path:&str) {
        //Representa o tabuleiro no estado atual. Células infectadas são pintadas de cinza, e células limpas são brancas
        // let OUT_FILE_NAME = "Output/plot_tabuleiro.png";
        let out_file_name = format!("Output/{} t={} .png", path, self.t);

        let root = BitMapBackend::new(&out_file_name, (LARGURA, ALTURA)).into_drawing_area();

        root.fill(&WHITE).expect("Erro no preenchimento");
        let titulo = format!("t = {}",self.t);
        let mut chart = ChartBuilder::on(&root)
            .caption(titulo, ("sans-serif", 80))
            .margin(5)
            // .top_x_label_area_size(40)
            // .y_label_area_size(40)
            .build_cartesian_2d(0i32..(self.n as i32), 0i32..(self.n as i32)).expect("Erro na construção do chart");

        chart
            .configure_mesh()
            // .x_labels(15)
            // .y_labels(15)
            .max_light_lines(15)
            .light_line_style(ShapeStyle{
                color: BLACK.into(),
                filled:false,
                stroke_width: 5
            })

            // .x_label_offset(35)
            // .y_label_offset(25)
            // .disable_x_mesh()
            // .disable_y_mesh()
            .label_style(("sans-serif", 20))
            .draw().expect("Erro no plot");

        let tab = array_para_vec(&self.tabuleiro);

        chart.draw_series(
            // matrix
            tab
                .iter()
                .zip(0..)
                .flat_map(|(l, y)| l.iter().zip(0..).map(move |(v, x)| (x, y, v)))
                .map(|(x, y, v)| {
                    Rectangle::new(
                        [(x, y), (x + 1, y + 1)],
                        match v {
                            true =>     RGBAColor(150,150,150,0.5),
                            false =>    RGBAColor(255,255,255,0.5),
                        }
                            .filled()
                    )
                }),
        ).expect("Erro na hora de desenhar");

        // To avoid the IO failure being ignored silently, we manually call the present function
        root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
        println!("Result has been saved to {}", out_file_name);
    }

    fn fazer_imagens(self: &mut Self, path: &str, paralelo: bool) {
        //Avança iterativamente o rpocesso de bootstrap, plotando e salvando a imagem a cada etapa
        // //Produz e salva imagens sucessivas e também um gif
        while !self.config_final {
            println!("Passo t={}", self.t);
            self.plotar(path);
            if paralelo { self.passo_paralelo(8)}
            else {self.atualiza_passo()}
        }
        println!("Chegamos à configuração final após {} iterações",self.t-1);
    }
    fn resolver_total(self: &mut Self, n_threads: usize) {

        while !self.config_final {
            println!("Passo t={}", self.t);
            if n_threads==1 { self.atualiza_passo() }
            else { self.passo_paralelo(n_threads) }
        }
        println!("Chegamos à configuração final após {} iterações", self.t-1);
    }
}
#[derive(PartialEq)]
enum Orientacao {
    Horizontal,
    Vertical,
}
#[derive(Debug)]
struct ColecaoRetangulos{
    //TODO() implementar de forma a obter eficientemente o retângulo com menor x1, maior y1 etc.
    //Cada retangulo é definido por [x1,x2,y1,y2]
    conjunto: ConjuntoIteravel<[usize;4]>,
    limites: [usize;4], //Área do sub-tabuleiro [x1,x2,y1,y2]. Deve conter todos os retângulos na área x1<=x<x2 e y1<=y<y2
}
impl ColecaoRetangulos {


    fn merge(mut self: Self, mut other: Self, ) -> Self {
        //Funde as coleções de retângulos correspondentes a áreas diferentes do plano
        //Pressupõe que as áreas são disjuntas e que other está diretamente acima ou diretamente à direita de self
        let x_fronteira = self.limites[1];
        let y_fronteira = self.limites[3];
        let candidatos = (&self.conjunto).into_iter()
            .filter(| [_x1, x2, _y1, y2] | (*x2 >= x_fronteira) | (*y2 >= y_fronteira));
        let mut novos_retangulos = ColecaoRetangulos{
                // conjunto:Vec::<[usize;4]>::new(),
                conjunto: ConjuntoIteravel::<[usize;4]>::novo(D_HASH),
                limites: [min(self.limites[0],other.limites[0]),
                    max(self.limites[1],other.limites[1]),
                    min(self.limites[2],other.limites[2]),
                    max(self.limites[3],other.limites[3]),]
            } ;
        for ret in candidatos {
            // self.remover(ret);
            novos_retangulos.conjunto.inserir(&ret);
        }
        let mut terminar_processo=false;
        while ! terminar_processo
        {
            terminar_processo=true;
            //Indices para deletar apos cada loop
            let mut novos_acrescentar = Vec::<[usize; 4]>::new();
            let mut novos_deletar = Vec::<[usize; 4]>::new();
            let mut self_deletar = Vec::<[usize; 4]>::new();
            let mut other_deletar = Vec::<[usize; 4]>::new();

            for (i,[x1, x2, y1, y2]) in (&novos_retangulos.conjunto).into_iter().enumerate() {
                for [x1a,x2a,y1a,y2a] in (&self.conjunto).into_iter() {
                    if retangulos_tocam([x1,x2,y1,y2], [x1a,x2a,y1a,y2a]) {
                        terminar_processo=false;
                        let novo_ret = [min(x1, x1a),  max(x2, x2a),min(y1, y1a), max(y2, y2a)];
                        novos_deletar.push([x1,x2,y1,y2]);
                        novos_acrescentar.push(novo_ret);
                        self_deletar.push([x1a, x2a, y1a, y2a]);
                    }
                }
                for [x1b, x2b, y1b, y2b] in (&other.conjunto).into_iter() {
                    //Checamos se o retangulo tem alguma quase-interseção com os retângulos candidatos
                    if retangulos_tocam([x1, x2, y1, y2], [x1b, x2b, y1b, y2b]) {
                        terminar_processo=false; //Se achamos alguma mudança nesse loop, continuamos o processo
                        let novo_ret = [min(x1, x1b),  max(x2, x2b),min(y1, y1b), max(y2, y2b)];
                        novos_deletar.push([x1,x2,y1,y2]);
                        novos_acrescentar.push(novo_ret);
                        other_deletar.push([x1b, x2b, y1b, y2b]);
                    }
                }
                for (j,[x1c,x2c,y1c,y2c]) in (&novos_retangulos.conjunto).into_iter().enumerate() {
                    if retangulos_tocam([x1,x2,y1,y2],[x1c,x2c,y1c,y2c]) && i!=j {
                        terminar_processo=false;
                        let novo_ret = [min(x1, x1c),  max(x2, x2c),min(y1, y1c), max(y2, y2c)];
                        novos_deletar.push([x1,x2,y1,y2]);
                        novos_deletar.push([x1c,x2c,y1c,y2c]);
                        novos_acrescentar.push(novo_ret);
                    }
                }
            }
            for item in novos_deletar.iter() { novos_retangulos.conjunto.remover(item) };
            for item in novos_acrescentar.iter() { novos_retangulos.conjunto.inserir(item)};
            for item in self_deletar.iter() { self.conjunto.remover(item) };
            for item in other_deletar.iter() { other.conjunto.remover(item) };
        }

        //Agora é preciso passar para o conjunto de novos retângulos todos aqueles que sobraram em self e em other
        for ret in self.conjunto.into_iter() {novos_retangulos.conjunto.inserir(&ret)};
        for ret in other.conjunto.into_iter() {novos_retangulos.conjunto.inserir(&ret)};
        novos_retangulos
    }
}

fn retangulos_tocam(ret1: [usize; 4], ret2: [usize; 4]) -> bool {
    //Retorna true se os retângulos delimitados por p0 e p1 se tocam pelo criterio do processo modified two-neighbor
    let [x1a,x2a,y1a,y2a] = ret1;
    let [x1b,x2b,y1b,y2b] = ret2;
    x1b<=x2a && x2b>=x1a && y1b<=y2a && y2b>=y1a
}

struct BootstrapMod2Neighbor {
    processo_bootstrap: ProcessoBootstrap
}

impl BootstrapMod2Neighbor {
    //Objeto do processo de Bootstrap do tipo Modified Two-neighbor
    //Implementa um algoritmode dividir e conquistar para resolver o estado final
    fn novo(n:usize, p: f64) -> Self {
        let familia_update = FamiliaUpdate{
            v: vec![
                vec![[1,0],[0,1]],
                vec![[0,1],[-1,0]],
                vec![[-1,0],[0,-1]],
                vec![[0,-1],[1,0]],
            ],
        };
        let processo_bootstrap = ProcessoBootstrap::novo(n,p,familia_update);
        BootstrapMod2Neighbor {
            processo_bootstrap
        }
    }

    fn resolver_sub(self: &Self, i0:usize, j0:usize, largura:usize) -> ColecaoRetangulos {
        //Olha para o sub-tabuleiro definido por (i0,j0)+[0,m)×[0,m)  e retorna os retângulos infectados que se formarão na configuração final desse sub-tabuleiro isolado
        //largura deve ser uma potência de 2
        //Aplica uma estratégia de dividir e conquistar separando a área em 4 quadrados
        if largura ==1 { //Se temos uma única célula, ela é um retângulo infectado de lado 1×1 ou nenhum retângulo
            return match self.processo_bootstrap.tabuleiro[[i0,j0]] {
                true => {
                    let mut conjunto = ConjuntoIteravel::<[usize;4]>::novo(D_HASH);
                    conjunto.inserir(&[i0, i0 + 1, j0, j0 + 1]);
                    ColecaoRetangulos{
                        conjunto,
                        limites: [i0, i0 + 1, j0, j0 + 1]
                    }
                }
                false => {
                    ColecaoRetangulos{
                        conjunto: ConjuntoIteravel::<[usize;4]>::novo(D_HASH),
                        limites: [i0, i0 + 1, j0, j0 + 1]
                    }
                }
            }
        }
        assert!(largura%2==0);
        //Dividindo o quadrado em 4 quadrados menores:
        let A_11 = self.resolver_sub(i0,j0,largura/2);
        let A_12 = self.resolver_sub(i0,j0+largura/2, largura/2);
        let A_21 = self.resolver_sub(i0+largura/2,j0,largura/2);
        let A_22 = self.resolver_sub(i0+largura/2,j0+largura/2,largura/2);

        //Fundindo os retângulos encontrados para os diferentes sub-tabuleiros:
        let B_1 = A_11.merge(A_12);
        let B_2 = A_21.merge(A_22);
        B_1.merge(B_2)
        }
    fn resolver_total(self: &mut Self) {
        let n = self.processo_bootstrap.n;
        let retangulos = self.resolver_sub(0, 0,n);
        println!("{:?}",retangulos);
        let mut tab = &mut self.processo_bootstrap.tabuleiro;
        for [x1,x2,y1,y2] in (&retangulos.conjunto).into_iter() {
            for i in x1..x2 {
                for j in y1..y2 {
                    tab[[i,j]] = true;
                }
            }
        }
        self.processo_bootstrap.config_final=true;
        self.processo_bootstrap.t+=1;
    }

}

fn teste_metodos() {
    //Verifica se os diferentes métodos dão o mesmo resultado
    let FAMILIA_UPDATE: FamiliaUpdate = FamiliaUpdate{
        v: vec![
            vec![[1,0],[0,1]],
            vec![[0,1],[-1,0]],
            vec![[-1,0],[0,-1]],
            vec![[0,-1],[1,0]],
        ],
    };
    let n:usize = 256;
    let A = gerar_estado_inicial(0.1,n,n);
    let mut boot_serial = ProcessoBootstrap{
        tabuleiro:A.clone(),
        fam_update: FAMILIA_UPDATE.clone(),
        n,
        t: 0,
        config_final: false,
    };
    let mut boot_paralelo = ProcessoBootstrap{
        tabuleiro:A.clone(),
        fam_update: FAMILIA_UPDATE.clone(),
        n,
        t: 0,
        config_final: false,
    };
    let mut boot_DC = BootstrapMod2Neighbor{
        processo_bootstrap: ProcessoBootstrap{
            tabuleiro:A.clone(),
            fam_update: FAMILIA_UPDATE.clone(),
            n,
            t: 0,
            config_final: false,
        }
    };
    // boot_serial.fazer_imagens(format!("Teste serial n={}",n).as_str(), false);
    // boot_paralelo.fazer_imagens(format!("Teste paralelo n={}",n).as_str(), true);
    // boot_DC.processo_bootstrap.plotar(format!("Teste DC n={} incial",n).as_str());
    boot_serial.resolver_total(1);
    boot_paralelo.resolver_total(8);
    boot_DC.resolver_total();
    // boot_DC.processo_bootstrap.plotar(format!("Teste DC n={} final",n).as_str());
    assert_eq!(boot_DC.processo_bootstrap.tabuleiro, boot_serial.tabuleiro);
    assert_eq!(boot_serial.tabuleiro,boot_paralelo.tabuleiro);

}


fn main() {
    teste_metodos();


    let FAMILIA_UPDATE: FamiliaUpdate = FamiliaUpdate{
        v: vec![
            vec![[1,0],[0,1]],
            vec![[0,1],[-1,0]],
            vec![[-1,0],[0,-1]],
            vec![[0,-1],[1,0]],
        ],
    };
    let A = ndarray::array![[true,true,true,false],[false,false,false,true],[true,false,false,false],[false,false,false,false]] as Array2<bool>;
    let mut boot = ProcessoBootstrap{
        tabuleiro: A,
        fam_update: FAMILIA_UPDATE,
        n: 4,
        t: 0,
        config_final: false,
    };
    boot.fazer_imagens("teste serial", false);
    let FAMILIA_UPDATE: FamiliaUpdate = FamiliaUpdate{
        v: vec![
            vec![[1,0],[0,1]],
            vec![[0,1],[-1,0]],
            vec![[-1,0],[0,-1]],
            vec![[0,-1],[1,0]],
        ],
    };
    // let A = gerar_estado_inicial(0.2,8,8);
    let A = ndarray::array![[true,true,true,false],[false,false,false,true],[true,false,false,false],[false,false,false,false]] as Array2<bool>;
    let mut boot = BootstrapMod2Neighbor {
        processo_bootstrap:ProcessoBootstrap{
        tabuleiro: A,
        fam_update: FAMILIA_UPDATE,
        n: 4,
        t: 0,
        config_final: false,
    }};
    boot.processo_bootstrap.plotar("DC - antes");
    boot.resolver_total();
    boot.processo_bootstrap.plotar("DC - depois")


}
