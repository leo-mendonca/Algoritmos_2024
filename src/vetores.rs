use std::alloc::{Layout, alloc, dealloc};
use rand::Rng;
use std::time::{Instant};


pub trait Vetor {
    //O trair Vetor eh criado para poder executar as mesmas funcoes nos dois tipos de vetor implementados
    fn novo() -> Self;
    fn ler(&self, i:i32) -> i32;
    fn colocar(&mut self, b:i32);
    fn alterar(&mut self, i:i32, b:i32);
    fn pegar(&mut self) ->i32;
    fn e_maior(self:&Self, other: &Self) -> bool;
}

fn _teste_vetor<T:Vetor>(_vetor:T) {
    let mut v: T = T::novo();
    for i in 0..10 {
        v.colocar(10*i);
        for j in 0..i {
            assert!(v.ler(j)==10*j);
        }
    }
    for k in 9..=1 {
        assert!(v.pegar()==10*k)
    }
    let u: T = T::novo();
    assert!(v.e_maior(&u));
    println!("Testes OK")
}

#[derive(Debug)]
pub struct VetorOn {
    n: i32,
    c:usize,
    p: *mut i32,
}

impl Vetor for VetorOn {
    fn novo() -> Self {
        //Cria um vetor vazio
        let c:usize = 16;
        let layout: Layout = match Layout::array::<i32>(c) {
            Ok(lt) =>lt,
            Err(e) => panic!("{}",e),
        };
        let p_u8: *mut u8 = unsafe {alloc(layout)};
        let p:*mut i32 = p_u8 as *mut i32;

        VetorOn {
            n: 0,
            c,
            p:p,
        }
    }
    fn ler(self:&Self, i:i32) -> i32 {
        //essa funcao le a i-esima coordenada do vetor (indexado a partir do 0)
        //se o vetor tiver comprimento <=i, causa um erro (panico)
        assert!(i<self.n);
        unsafe {
            let ponteiro: *mut i32= self.p.add(i as usize);
            ponteiro.read()
        }
    }
    fn colocar(self:&mut Self, b:i32) {
        if self.n>=self.c as i32 {self.redimensionar(2*self.c);}
        unsafe {self.p.add(self.n as usize).write(b);}
        self.n+=1;
    }
    fn alterar(self:&mut Self, i:i32, b:i32) {
        assert!(i<self.n);
        unsafe {
            let ponteiro: *mut i32 = self.p.add(i as usize);
            ponteiro.write(b);
        }
    }
    fn pegar(self:&mut Self) ->i32 {
        let valor:i32 = self.ler(self.n-1);
        self.n -= 1;
        valor
    }
    fn e_maior(self:&Self, other:&VetorOn) ->bool {
        self.n>=other.n
    }
}

impl VetorOn {
    //vetor implementado manualmente (usa unsafe), contendo entradas i32
    //tamanho dinamico

    fn redimensionar(self:&mut Self, tamanho:usize) {
        //Alocando um novo local de memoria com mais capacidade:
        let layout:Layout = Layout::array::<i32>(tamanho).expect("Erro no layout de alocacao") ;
        let q: *mut i32 = unsafe {alloc(layout)} as *mut i32;
        //Copiando os dados para o novo local de memoria:
        unsafe {
            for i in 0..self.n {
                let valor: i32 = self.p.add(i as usize).read();
                q.add(i as usize).write(valor);
            }
        }
        //Desalocando a memoria anterior:
        let layout_velho: Layout = Layout::array::<i32>(self.c).expect("Erro no layout de desalocacao");
        unsafe {dealloc(self.p as *mut u8, layout_velho)};
        self.p=q;
        self.c=tamanho;
    }
}



#[derive(Debug)]
pub struct VetorO1 {
    n: i32,
    n_copiados: i32,
    c:usize,
    p: *mut i32,
    q: *mut i32,
}

impl Vetor for VetorO1 {
    fn novo() ->Self {
        //Cria um vetor vazio
        let c:usize = 16;
        //layout principal do vetor
        let layout: Layout = match Layout::array::<i32>(c) {
            Ok(lt) =>lt,
            Err(e) => panic!("{}",e),
        };
        let p_u8: *mut u8 = unsafe {alloc(layout)};
        let p:*mut i32 = p_u8 as *mut i32;
        //layout secundario com o dobro da capacidade
        let layout_2: Layout = match Layout::array::<i32>(2*c) {
            Ok(lt) =>lt,
            Err(e) => panic!("{}",e),
        };
        let q_u8: *mut u8 = unsafe {alloc(layout_2)};
        let q:*mut i32 = q_u8 as *mut i32;

        VetorO1 {
            n: 0,
            n_copiados:0,
            c,
            p:p,
            q:q,
        }
    }
    fn ler(self:&Self, i:i32) -> i32 {
        //essa funcao le a i-esima coordenada do vetor (indexado a partir do 0)
        //se o vetor tiver comprimento <=i, causa um erro (panico)
        assert!(i<self.n);
        unsafe {
            let ponteiro: *mut i32= self.p.add(i as usize);
            ponteiro.read()
        }
    }
    fn colocar(self:&mut Self, b:i32) {
        if self.n>=self.c as i32 {self.trocar_local_memoria();}
        unsafe {self.p.add(self.n as usize).write(b);}
        self.n+=1;
        for _i in 1..=2 {
            self.copiar_para_secundario();
        }
    }
    fn alterar(self:&mut Self, i:i32, b:i32) {
        assert!(i<self.n);
        unsafe {
            let ponteiro: *mut i32 = self.p.add(i as usize);
            ponteiro.write(b);
        }
        if i>=self.n_copiados {
            //Se o vetor secundario ja estiver copiado ate o valor de i, precisamos alterar o valor no secundario tambem
            unsafe {
                let ponteiro: *mut i32 = self.q.add(i as usize);
                ponteiro.write(b);
            }
        }
    }
    fn pegar(self:&mut Self) ->i32 {
        let valor:i32 = self.ler(self.n-1);
        self.n -= 1;
        if self.n_copiados==self.n {self.n_copiados-=1;}
        valor
    }
    fn e_maior(self:&Self, other:&VetorO1) ->bool {
        self.n>=other.n
    }
}

impl VetorO1 {
    //vetor implementado manualmente (usa unsafe), contendo entradas i32
    //tamanho dinamico
    fn trocar_local_memoria(self:&mut Self) {
        //Troca o local primario de armazenamento pelo secundario, que tem o dobro da capacidade, em O(1)
        assert!(self.n_copiados==self.n);
        /*//Se nao tivermos terminado de copiar os dados do armazenamento primario para o secundario, devemos faze-lo agora:
        while self.n_copiados<self.n {self.copiar_para_secundario()}*/
        //Desalocando a memoria anterior:
        {
            let layout_velho: Layout = Layout::array::<i32>(self.c).expect("Erro no layout de desalocacao");
            unsafe { dealloc(self.p as *mut u8, layout_velho) };
            self.p = self.q.clone();
            self.c *= 2;
        }
        //Criando um novo local para armazenamento do vetor secundario:
        {
            let layout: Layout = Layout::array::<i32>(2*self.c).expect("Erro no layout para alocacao da memoria secundaria");
            let q_u8: *mut u8 = unsafe {alloc(layout)};
            self.q = q_u8 as *mut i32;
            self.n_copiados=0;
        }


    }
    fn copiar_para_secundario(self:&mut Self) {
        //Copia um elemento do local primario de armazenamento para o local secundario
        if self.n_copiados>=self.n {return}
        else {
            let valor_copiar:i32 = self.ler(self.n_copiados);
            unsafe {self.q.add(self.n_copiados as usize).write(valor_copiar);}
            self.n_copiados+=1;
        }
    }

}


fn _tempo_operacoes_vetor<T:Vetor>(n:i32, _vetor:T) {
    //Funcao que mede o tempo necessario para realizar as principais operacoes com o objeto Vetor
    let mut u:T = T::novo();
    // let mut f: fs::File = File::create(format!("saida/tempo n= {n}.txt")).expect("Erro na leitura do arquivo");
    let mut t_ler_soma:u128 =0;
    let mut t_alterar_soma:u128 = 0;
    let mut t_pegar_soma:u128 = 0;
    let mut t_colocar_soma:u128 = 0;
    let mut t_ler_max:u128 =0;
    let mut t_alterar_max:u128 = 0;
    let mut t_pegar_max:u128 = 0;
    let mut t_colocar_max:u128 = 0;
    // f.write(b"tamanho;t_ler;t_alterar;t_pegar;t_colocar\n").expect("Erro escrevendo no arquivo");
    let b: i32=rand::thread_rng().gen_range(1..=100);
    u.colocar(b);
    for i in 1..n {
        let b: i32=rand::thread_rng().gen_range(1..=100);
        let b_escrever: i32= rand::thread_rng().gen_range(1..=100);
        let posicao_ler: i32 = rand::thread_rng().gen_range(0..i);
        let posicao_escrever: i32 = rand::thread_rng().gen_range(0..i);
        let t0: Instant = Instant::now();
        u.ler(posicao_ler);
        let t1: Instant = Instant::now();
        u.alterar(posicao_escrever, b_escrever);
        let t2: Instant = Instant::now();
        u.colocar(b);
        let t3: Instant = Instant::now();
        u.pegar();
        let t4: Instant = Instant::now();
        u.colocar(b); //apenas colocando de volta o valor que foi retirado
        let t_ler = t1.duration_since(t0).as_nanos();
        let t_alterar = t2.duration_since(t1).as_nanos();
        let t_pegar = t4.duration_since(t3).as_nanos();
        let t_colocar = t3.duration_since(t2).as_nanos();
        // f.write(format!("{i};{t_ler};{t_alterar};{t_pegar};{t_colocar}\n").as_ref()).expect("Erro escrevendo no arquivo");
        //Alterando o registro da media e da maxima do tempo de cada operacao
        t_ler_soma+=t_ler;
        if t_ler>t_ler_max {t_ler_max=t_ler};
        t_pegar_soma+=t_pegar;
        if t_pegar>t_pegar_max {t_pegar_max=t_pegar};
        t_alterar_soma+=t_alterar;
        if t_alterar>t_alterar_max {t_alterar_max=t_alterar};
        t_colocar_soma+=t_colocar;
        if t_colocar>t_colocar_max {t_colocar_max=t_colocar};
    }
    let t_ler_media:f32 = (t_ler_soma as f32)/(n as f32);
    let t_alterar_media:f32 = (t_alterar_soma as f32)/(n as f32);
    let t_pegar_media:f32 = (t_pegar_soma as f32)/(n as f32);
    let t_colocar_media:f32 = (t_colocar_soma as f32)/(n as f32);
    println!("Vetor de {n} elementos");
    println!("Tempo \t\tMédia \t\tMáxima");
    println!("Leitura:\t{t_ler_media:.1} ns \t{t_ler_max} ns");
    println!("Alterar:\t{t_alterar_media:.1} ns \t{t_alterar_max} ns");
    println!("Pegar:  \t{t_pegar_media:.1} ns \t{t_pegar_max} ns");
    println!("Colocar:\t{t_colocar_media:.1} ns \t{t_colocar_max} ns");

}


pub fn vetor_main() {
    println!("Começo");
    _teste_vetor(VetorO1::novo());
    _teste_vetor(VetorOn::novo());
    println!("\nVetor O(n)");
    _tempo_operacoes_vetor(10000, VetorOn::novo());
    _tempo_operacoes_vetor(100000, VetorOn::novo());
    _tempo_operacoes_vetor(1000000, VetorOn::novo());
    println!("\nVetor O(1)");
    _tempo_operacoes_vetor(10000, VetorO1::novo());
    _tempo_operacoes_vetor(100000, VetorO1::novo());
    _tempo_operacoes_vetor(1000000, VetorO1::novo());
    println!("Fim!")
}