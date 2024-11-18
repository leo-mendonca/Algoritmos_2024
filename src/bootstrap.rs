use std::cmp::{max, min};
use std::error::Error;
use std::fs::File;
use rand::random;
use ndarray::{Array2, ArrayBase, ErrorKind, Ix2, OwnedRepr, ShapeError};
use ndarray;
use plotters::prelude::*;
use gif;
use std::sync::{mpsc,Arc,Mutex};
use std::thread;
use gif::streaming_decoder::OutputBuffer;
use crossbeam;

const ALTURA:u32 = 1050;
const LARGURA:u32 = 1680;

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
    if ! (0<=i1 && i1<i2 && i2<=n)
        | !  (0<=j1 && j1<j2 && j2<=n) {
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
        Err(E) => {Err(E)}
    }
}

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
        // let mut tab_novo = self.tabuleiro.clone();
        // let mut config_final=true; //Indica se o tabuleiro esta na configuracao final
        // for i in 0..self.n {
        //     for j in 0..self.n {
        //        if !self.tabuleiro[[i,j]] {
        //            if checar_propagacao(&self.tabuleiro, &self.fam_update, i, j) {
        //                config_final=false; //Se alteramos alguma célula, então não estamos na configuração final
        //                tab_novo[[i, j]] = true;
        //            }
        //        }
        //     }
        // }
        //
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
        let OUT_FILE_NAME = format!("Output/{} t={} .png", path,self.t);

        let root = BitMapBackend::new(&OUT_FILE_NAME, (LARGURA, ALTURA)).into_drawing_area();

        root.fill(&WHITE);
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
            // .max_light_lines(15)
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
        println!("Result has been saved to {}", OUT_FILE_NAME);
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
}



fn main() {
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
    // let mut boot = ProcessoBootstrap::novo(4,0.2,FAMILIA_UPDATE);
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
    // let mut boot = ProcessoBootstrap::novo(4,0.2,FAMILIA_UPDATE);
    let mut boot = ProcessoBootstrap{
        tabuleiro: A,
        fam_update: FAMILIA_UPDATE,
        n: 4,
        t: 0,
        config_final: false,
    };
    boot.fazer_imagens("teste paralelo", true);

    // boot.plotar("Teste 16×16");
    // println!("Estado inicial:\n{:?}",boot.tabuleiro);
    // boot.atualiza_passo();
    // boot.plotar("Teste 16×16");
    // println!("Atualizando o tabuleiro uma vez:\n{:?}",boot.tabuleiro);
    // boot.atualiza_passo();
    // boot.plotar("Teste 16×16");
    // println!("Atualizando o tabuleiro de novo:\n{:?}",boot.tabuleiro);
    // boot.plotar("Teste 16×16");
}
