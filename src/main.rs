mod vetores;
mod lista_encadeada;

use crate::lista_encadeada::{ListaDupla, ListaEncadeada};
use crate::vetores::Vetor;

struct Editor<'a> {
    path: &'a str,
    pos_cursor: u32,
    tamanho: u32,
}

impl<'a> Editor<'a> {
    //todo()
}

#[test]
fn teste_erro() {
    assert!(true)
}


fn main() {
    // let mut u: vetores::VetorOn = vetores::VetorOn::novo();
    // u.colocar(7);
    // u.colocar(3);
    // println!("{:?}",u);
    // vetores::vetor_main();
    // let mut lista:ListaEncadeada<char>=ListaEncadeada::novo();
    // let mut lista_2:ListaDupla<i32> = ListaDupla::novo();
    // for letra in "abcde".chars() {lista.colocar(letra)};
    // for n in [10,9,8,7,6,5].into_iter() {lista_2.colocar(n)}
    // println!("{}",lista);
    // println!("{}",lista_2);
    // lista_encadeada::lista_main();
}
