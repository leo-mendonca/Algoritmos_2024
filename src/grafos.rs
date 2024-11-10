use Algoritmos::tabela_chave_valor::{ConjuntoIteravel, TabelaCV};

struct Grafo {
    tabela: TabelaCV<u32, ConjuntoIteravel<u32>>,
    d: u64
}
impl Grafo {

    fn novo(d: u64) -> Self {
        let tabela = TabelaCV::<u32, ConjuntoIteravel::<u32>>::novo(d);
        Grafo {
            tabela,
            d
        }
    }
    fn inserir_vertice(self: &mut Self, x:u32) {
        self.tabela.inserir(&x, ConjuntoIteravel::<u32>::novo(self.d))
    }
    fn inserir_elo(self: &mut Self, x:u32, y:u32) {
        assert!(self.contem_vertice(x));
        assert!(self.contem_vertice(y));
        let viz_x = self.tabela.ler_mut(&x).expect("Garantimos que x está no grafo. Senão, a função leva a erro");
        viz_x.inserir(&y);
        let viz_y = self.tabela.ler_mut(&y).expect("Também garantimos que y está no grafo");
        viz_y.inserir(&x);
        //Pela forma como inserir é definida no Conjunto, se o elo já existir, nada é feito
    }
    fn contem_vertice(self: &Self, i: u32) -> bool {
        match self.tabela.ler(&i) {
            None => {false}
            Some(_) => {true}
        }
    }
    fn contem_elo(self: &Self, x: u32, y: u32) -> bool {
        //Retorna true se e somente se ambos os vértices existirem e tiverem um elo entre si
        match self.tabela.ler(&x) {
            None => false,
            Some(conjunto) => conjunto.contem(&y),
        }
    }
    fn remover_elo(self: &mut Self, x:u32, y:u32) {
        if !(self.contem_elo(x,y)) {return} //Se o elo não existir, nada precisa ser feito
        let viz_x: &mut ConjuntoIteravel<u32> = self.tabela.ler_mut(&x).expect("Garantimos acima que x e y estão no grafo");
        viz_x.remover(&y);
        let viz_y: &mut ConjuntoIteravel<u32> = self.tabela.ler_mut(&y).expect("Garantimos acima que x e y estão no grafo");
        viz_y.remover(&x);

    }
    fn remover_vertice(self: &mut Self, x:u32) {
        //Remove um vertice, eliminando todos os seus elos
        //Se o vertice nao existir, nada acontece
        if !(self.contem_vertice(x)) {return}
        {
            let lista_vizinhos = {
                let viz_x = self.tabela.ler(&x).expect("Sabemos que x pertence ao conjunto");
                Vec::from_iter(viz_x.into_iter())
            };
            for y in lista_vizinhos {
                self.remover_elo(x, y)
            }
        }
        self.tabela.remover(&x);
    }
}

#[test]
fn teste_grafo() {
    let mut g = Grafo::novo(5);
    for i in 1..=5 {
        g.inserir_vertice(i);
    }
    g.inserir_elo(1,2);
    g.inserir_elo(1,3);
    g.inserir_elo(4,5);
    for i in 1..=5 {
        assert!(g.contem_vertice(i));
    }
    assert!(g.contem_elo(1,2));
    assert!(g.contem_elo(3,1));
    assert!(!g.contem_elo(4,1));
    assert!(g.contem_elo(4,5));
    assert!(!g.contem_elo(5,3));
    assert!(!g.contem_vertice(6));
    g.remover_vertice(1);
    assert!(!g.contem_vertice(1));
    assert!(!g.contem_elo(1,2));

}

fn main() {
    println!("Grafos");
}
