use std::{fs, io};
use std::hash::{Hasher, Hash, RandomState, BuildHasher};
use std::io::BufRead;
use std::ptr::hash;
use std::vec::IntoIter;
use crate::lista_encadeada::{CelulaDupla, IteradorListaDupla, ListaDupla};



pub struct TabelaCV<Tc: Hash+Eq+Clone,Tv> {
    //todo() implementar usando vetores em vez de listas encadeadas para evitar os erros encontrados ao usar String na lista
    //Tabela chave-valor com chaves do tipo Tc (tem que ser espalhavel/hashable) e valores do tipo Tv
    d:u64,
    // v:Vec<ListaDupla<(Tc,Tv)>>, //Vetor com a referencia de cada lista encadeada correspondente a um valor do hash,
    v:Vec<Vec<(Tc,Tv)>>, //Vetor com a referencia de cada subvetor correspondente a um valor do hash,
    gerador_hash: RandomState, //objeto que cria as funcoes de espalhamento/hashes dessa tabela
}

impl<Tc:Hash+Eq+Clone,Tv> TabelaCV<Tc,Tv> {
    pub fn novo(d:u64) -> Self {
        let gerador_hash = RandomState::new();
        let mut v = Vec::<Vec<(Tc,Tv)>>::new();
        for _i in 1..=d {
            v.push(Vec::<(Tc,Tv)>::new());
        }
        TabelaCV::<Tc,Tv> {
            d,
            v,
            gerador_hash,
        }
    }

    fn calcular_hash(self:&Self, chave:Tc) -> u64 {
        let mut s = self.gerador_hash.build_hasher();
        chave.hash(&mut s);
        s.finish() % self.d
    }
    pub fn inserir(self:&mut Self, chave: &Tc, valor:Tv) {
        let c = (*chave).clone();
        let hash_chave = self.calcular_hash(c.clone());
        // let mut lista = &mut self.v[hash_chave as usize];
        match self.procurar_chave_hash(&c, hash_chave) {
            Some((indice, _valor)) =>{
                // self.v[hash_chave as usize].alterar(end, (chave, valor))
                // self.v[hash_chave as usize].insert(indice, (c, valor))
                self.v[hash_chave as usize][indice] =(c, valor)
            }
            None => {
                // self.v[hash_chave as usize].colocar((chave,valor))
                self.v[hash_chave as usize].push((c, valor))
            }
        }
    }
    pub fn remover(self: &mut Self, chave:&Tc) {
        //Procura a chave na tabela e deleta a entrada correspondente (se houver)
        let hash_chave = self.calcular_hash((*chave).clone());
        if let Some((indice,_valor)) = self.procurar_chave_hash(chave, hash_chave) {
            self.v[hash_chave as usize].remove(indice);
        }
    }

    pub fn ler(self: &Self, chave: &Tc) -> Option<&Tv> {
        let hash_chave = self.calcular_hash((*chave).clone());
        match self.procurar_chave_hash(chave, hash_chave) {
            Some((_endereco, valor)) => Some(valor),
            None => None
        }
    }

    fn procurar_chave_hash(self: &Self, chave: &Tc, hash_chave: u64) -> Option<(usize, &Tv)> {
        let v = self.v.get(hash_chave as usize).expect("O vetor deve conter um subvetor na posicao correspondente a cada valor do hash");
        for (i, (c, val)) in v.iter().enumerate() {
            if c==chave {
                return Some((i,val))
            }
        }
        None
    }

    // fn _procurar_chave_hash_lista(self: &Self, chave: Tc, hash_chave:u64) -> Option<(*mut CelulaDupla<(Tc, Tv)>, Tv)> {
    //     //Procura o valor e o endereco, na lista associada ao valor do hash dado, da celula correspondente à chave
    //     //Se a chave nao for encontrada, retorna None
    //     let lista: &ListaDupla<(Tc, Tv)> = self.v.get(hash_chave as usize).expect("O vetor de listas encadeadas deve ter um espaço para cada possível valor do hash (de 0 a d-1)");
    //     let mut iterador_lista = lista.into_iter();
    //     while let Some((c,v)) = iterador_lista.next() {
    //         if c==chave {
    //             return Some((iterador_lista.endereco_atual as *mut CelulaDupla<(Tc,Tv)>, v))
    //         }
    //     }
    //     None //Se nao encontrar a chave ate o final da lista
    // }
}
pub struct Conjunto<Tc:Hash+Clone+Eq> {
    //Estrutura de dados que, dado um elemento(chave), diz se ele pertence ou não ao conjunto
    //Aproveitamos a estrutura da tabela chave-valor, mas usando uma tupla vazia como valor, para não ocupar espaço na memória
    //Construido como sugerido no Rustonomicon
    // tabela: TabelaCV<Tc,()>
    tabela: TabelaCV<Tc,()>
}
impl<Tc:Hash+Clone+Eq> Conjunto<Tc> {
    pub fn novo(d:u64) -> Self {
        // let tabela = TabelaCV::<Tc,()>::novo(d);
        let tabela = TabelaCV::<Tc,()>::novo(d);
        Conjunto::<Tc> {
            tabela
        }
    }
    pub fn contem(self: &Self, elemento: &Tc) ->bool {
        //avalia se um dado elemento esta contido no conjunto
        match self.tabela.ler(elemento) {
            Some(_item) => true,
            None => false,
        }
    }
    pub fn inserir(self: &mut Self, elemento: &Tc) {
        //acrescenta um elemento no conjunto
        // self.tabela.acrescentar(elemento, ())
        self.tabela.inserir(elemento, ())
    }
    pub fn remover(self: &mut Self, elemento: &Tc) {
        self.tabela.remover(elemento)
    }
}

pub struct ConjuntoIteravel<Tc:Hash+Clone+Eq> {
    tabela: TabelaCV<Tc, *mut CelulaDupla<Tc>>,
    lista: ListaDupla<Tc>,
}

impl<Tc:Hash+Clone+Eq> ConjuntoIteravel<Tc> {
    pub fn novo(d: u64) -> Self {
        let tabela = TabelaCV::<Tc, *mut CelulaDupla<Tc>>::novo(d);
        let lista = ListaDupla::<Tc>::novo();
        ConjuntoIteravel::<Tc> {
            tabela,
            lista
        }
    }
    pub fn inserir(self: &mut Self, elemento: &Tc) {
        match self.tabela.ler(&elemento) {
            None => {
                self.lista.colocar(elemento.clone()); //insere na ponta da lista
                self.tabela.inserir(elemento, self.lista.ponta.clone());
                return
            }
            Some(endereco) => {
                return //elemento ja esta no conjunto
            }
        }
    }

    pub fn remover(self: &mut Self, elemento: &Tc) {
        if let Some(endereco) = self.tabela.ler(elemento) {
            self.lista.deletar(*endereco);
            self.tabela.remover(elemento);
        }
    }

    pub fn contem(self: &Self, elemento: &Tc) ->bool {
        match self.tabela.ler(elemento) {
            None => false,
            Some(_item) => true,
        }
    }
}
impl<'a,T:Hash+Clone+Eq> IntoIterator for &'a ConjuntoIteravel<T> {
    type Item = T;
    type IntoIter = IteradorListaDupla<'a,T>;

    fn into_iter(self) -> Self::IntoIter {
        self.lista.into_iter()
    }
}

fn ler_dicionario(path: &str) ->Conjunto<String> {
    //Essa funcao nao funcionava porque a lista encadeada tem erro de heap corruption quando usamos uma string como entrada
    //Por isso foi necessario modificar a tabela chave-valor para usar vetores em vez de listas
    let arquivo = fs::File::open(path).expect("O arquivo deve existir e estar acessível");
    let mut conj = Conjunto::<String>::novo(10000);
    let leitor = io::BufReader::new(arquivo);
    for linha in leitor.lines() {
        if let Ok(palavra) = linha {
            conj.inserir(&palavra);
            assert!(conj.contem(&palavra))
        }
    }
    conj
}

#[test]
fn teste_tabela() {
    let mut t = TabelaCV::<char,i32>::novo(5);
    assert_eq!(t.ler(&'a'),None);
    t.inserir(&'a', 1);
    t.inserir(&'b', 2);
    t.inserir(&'h', 8);
    assert_eq!(t.ler(&'a'),Some(&1));
    assert_eq!(t.ler(&'b'),Some(&2));
    t.inserir(&'b', 80);
    assert_eq!(t.ler(&'b'),Some(&80));
    t.remover(&'b');
    assert_eq!(t.ler(&'b'),None);
    t.remover(&'b');
    assert_eq!(t.ler(&'b'),None);
}

// #[test]
// fn teste_tabela_string() {
//     let mut t = TabelaCV::<String, i32>::novo(2);
//     for s in ["a", "Aarao", "aba", "abacate", "abacateiro", "abacateiros", "abacates", "casa", "lagartixa"] {
//         t.acrescentar(s.to_string(), 1);
//     }
//     assert_eq!(t.ler("a".to_string()), Some(1));
//     //todo() temos um erro quando usamos String ou str na tabela chave-valor, porque na verdade a lista encadeada parece ser incompatível com tipos de tamanho dinâmico
// }

#[test]
pub fn teste_ortografia() {
    let mut conj = ler_dicionario("Input/Dicionário.txt");
    // let mut conj = Conjunto::<String>::novo(2);
    for s in ["a","Aarao","aba","abacate","abacateiro","abacateiros","abacates", "casa","lagartixa"] {
        conj.inserir(&s.to_string());
    }
    //Testando palavras que estão no dicionário
    assert!(conj.contem(&"casa".to_string()));
    assert!(conj.contem(&"lagartixa".to_string()));
    //E se cometermos um erro ortográfico?
    assert!(! conj.contem(&"largatixa".to_string()));
    //Podemos retirar palavras do dicionário
    assert!(conj.contem(&"abacate".to_string()));
    conj.remover(&"abacate".to_string());
    assert!(! conj.contem(&"abacate".to_string()));
    //podemos acerescentar palavras
    assert!(! conj.contem(&"Lagrange".to_string()));
    conj.inserir(&"Lagrange".to_string());
    assert!(conj.contem(&"Lagrange".to_string()));
}
#[test]
pub fn teste_conjunto() {
    let mut conj = Conjunto::<i32>::novo(1000);
    for i in 1..=10000 {
        conj.inserir(&i)
    }
    for i in 1..=10000 {
        assert!(conj.contem(&i));
    }
    conj.remover(&7);
    assert!(! conj.contem(&7));
}

#[test]
fn teste_conjunto_iteravel() {
    let mut conj = ConjuntoIteravel::<i32>::novo(100);
    for i in 1..=10 {
        conj.inserir(&i)
    }
    for i in 1..=10 {
        assert!(conj.contem(&i));
    }
    conj.remover(&7);
    assert!(! conj.contem(&7));
    let mut v = Vec::<i32>::new();
    for item in conj.into_iter() {
        println!("{}",item);
        v.push(item);
    }
    assert!(v==vec![1,2,3,4,5,6,8,9,10]);
}

pub fn main_tabela() {
    let mut t = TabelaCV::<char,i32>::novo(1);
    let res = t.ler(&'a');
    println!("{:?}",res);
    t.inserir(&'a', 1);
    t.inserir(&'b', 2);
    t.inserir(&'h', 8);
    println!("a : {:?}",t.ler(&'a'));
    println!("b : {:?}",t.ler(&'b'));
    t.inserir(&'b', 80);
    println!("b : {:?}",t.ler(&'b'));
    t.remover(&'b');
    println!("b : {:?}",t.ler(&'b'));

    // teste_ortografia();

}