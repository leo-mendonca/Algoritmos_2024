use std::hash::{Hasher, Hash, RandomState, BuildHasher};
use crate::lista_encadeada::{CelulaDupla, ListaDupla};

pub struct TabelaCV<Tc: Hash+Clone+Copy+Eq,Tv:Clone> {
    //Tabela chave-valor com chaves do tipo Tc (tem que ser espalhavel/hashable) e valores do tipo Tv
    d:u64,
    v:Vec<ListaDupla<(Tc,Tv)>>, //Vetor com a referencia de cada lista encadeada correspondente a um valor do hash,
    gerador_hash: RandomState, //objeto que cria as funcoes de espalhamento/hashes dessa tabela
}

impl<Tc:Hash+Clone+Copy+Eq,Tv:Clone> TabelaCV<Tc,Tv> {
    pub fn novo(d:u64) -> Self {
        let gerador_hash = RandomState::new();
        let mut v = Vec::<ListaDupla<(Tc,Tv)>>::new();
        for _i in 1..=d {
            v.push(ListaDupla::<(Tc,Tv)>::novo())
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
    pub fn acrescentar(self:&mut Self, chave: Tc, valor:Tv) {
        let hash_chave = self.calcular_hash(chave);
        // let mut lista = &mut self.v[hash_chave as usize];
        match self.procurar_chave_hash(chave, hash_chave) {
            Some((end, _valor)) =>{
                self.v[hash_chave as usize].alterar(end, (chave.clone(), valor.clone()))
            }
            None => {
                self.v[hash_chave as usize].colocar((chave.clone(),valor.clone()))
            }
        }
    }
    pub fn remover(self: &mut Self, chave:Tc) {
        //Procura a chave na tabela e deleta a entrada correspondente (se houver)
        let hash_chave = self.calcular_hash(chave);
        if let Some((end,_valor)) = self.procurar_chave_hash(chave, hash_chave) {
            self.v[hash_chave as usize].deletar(end)
        }
    }

    pub fn ler(self: &Self, chave: Tc) -> Option<Tv> {
        let hash_chave = self.calcular_hash(chave);
        match self.procurar_chave_hash(chave,hash_chave) {
            Some((_endereco, valor)) => Some(valor),
            None => None
        }
    }

    fn procurar_chave_hash(self: &Self, chave: Tc, hash_chave:u64) -> Option<(*mut CelulaDupla<(Tc, Tv)>, Tv)> {
        //Procura o valor e o endereco, na lista associada ao valor do hash dado, da celula correspondente à chave
        //Se a chave nao for encontrada, retorna None
        let lista: &ListaDupla<(Tc, Tv)> = self.v.get(hash_chave as usize).expect("O vetor de listas encadeadas deve ter um espaço para cada possível valor do hash (de 0 a d-1)");
        let mut iterador_lista = lista.into_iter();
        // for (c,v) in iterador_lista {
        //     if c==*chave {return Some((iterador_lista.endereco_atual as *mut CelulaDupla<(Tc,Tv)>, v))};
        // }
        while let Some((c,v)) = iterador_lista.next() {
            if c==chave {
                return Some((iterador_lista.endereco_atual as *mut CelulaDupla<(Tc,Tv)>, v))
            }
        }
        None //Se nao encontrar a chave ate o final da lista
    }
}

#[test]
fn teste_tabela() {
    let mut t = TabelaCV::<char,i32>::novo(5);
    assert_eq!(t.ler('a'),None);
    t.acrescentar('a',1);
    t.acrescentar('b',2);
    t.acrescentar('h',8);
    assert_eq!(t.ler('a'),Some(1));
    assert_eq!(t.ler('b'),Some(2));
    t.acrescentar('b',80);
    assert_eq!(t.ler('b'),Some(80));
    t.remover('b');
    assert_eq!(t.ler('b'),None);
}

pub fn main_tabela() {
    let mut t = TabelaCV::<char,i32>::novo(5);
    let res = t.ler('a');
    println!("{:?}",res);
    t.acrescentar('a',1);
    t.acrescentar('b',2);
    t.acrescentar('h',8);
    println!("a : {:?}",t.ler('a'));
    println!("b : {:?}",t.ler('b'));
    t.acrescentar('b',80);
    println!("b : {:?}",t.ler('b'));
    t.remover('b');
    println!("b : {:?}",t.ler('b'));


}