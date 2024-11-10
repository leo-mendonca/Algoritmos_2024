use std::collections::VecDeque;
use infinitable::Infinitable;
use Algoritmos::tabela_chave_valor::{ConjuntoIteravel, TabelaCV};
use infinitable::Infinitable::{Finite,Infinity};


struct Grafo {
    tabela: TabelaCV<u32, ConjuntoIteravel<u32>>,
    d: u64,
    lista_nos: ConjuntoIteravel<u32>,
}
impl Grafo {

    fn novo(d: u64) -> Self {
        let tabela = TabelaCV::<u32, ConjuntoIteravel::<u32>>::novo(d);
        let lista_nos = ConjuntoIteravel::<u32>::novo(d);
        Grafo {
            tabela,
            d,
            lista_nos
        }
    }
    fn inserir_vertice(self: &mut Self, x:u32) {
        //Se o vertice ja existir, nada acontece
        self.tabela.inserir(&x, ConjuntoIteravel::<u32>::novo(self.d));
        self.lista_nos.inserir(&x);
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
        self.lista_nos.contem(&i)
        // match self.tabela.ler(&i) {
        //     None => {false}
        //     Some(_) => {true}
        // }
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
        self.lista_nos.remover(&x);
    }
}
#[derive(Debug)]
enum Cor {
    Branco,
    Verde,
    Laranja
}
#[derive(Debug)]
struct Decoracao {
    //estrutura para listar caracteristicas de cada nó durante a busca em largura
    cor: Cor,
    profundidade: Infinitable<u32>, //A profundidade pode ser infinita
    pai: Option<u32>
}

fn busca_largura(g: &Grafo, s: u32) -> TabelaCV<u32,Decoracao> {
    //colore os nós do grafo g a partir do nó s, para encontrar a componente conexa
    let mut fila = VecDeque::<u32>::new();
    let mut tab = TabelaCV::<u32,Decoracao>::novo(100);
    let dec_s = Decoracao {
        cor: Cor::Verde,
        profundidade: Finite(0),
        pai:None,
    };
    fila.push_back(s);
    tab.inserir(&s, dec_s);
    while let Some(u) = fila.pop_front() { //enquanto a fila não estiver vazia
        let dec_u = tab.ler(&u).expect("O nó já foi inserido na tabela");
        let prof_u = dec_u.profundidade;
        //Varrendo os vizinhos de u:
        for x in g.tabela.ler(&u).expect("O nó pertence ao grafo").into_iter() {
            if ! tab.contem(&x) { //se x não está na tabela ainda, temos que acrescenta-lo
                let dec_x = Decoracao {
                    cor: Cor::Verde,
                    profundidade: prof_u+Finite(1),
                    pai: Some(u),
                };
                tab.inserir(&x, dec_x);
                fila.push_back(x);
            }
        }
        //Terminados os vizinhos de u, ele deve ser colorido de laranja
        let dec_u_mut = tab.ler_mut(&u).expect("O nó u está na tabela");
        dec_u_mut.cor = Cor::Laranja;
    }
    //Os nós remanescentes devem ser pintados de branco e ter profundidade infinita
    for y in g.lista_nos.into_iter() {
        if ! tab.contem(&y) {
            let dec_y = Decoracao {
                cor: Cor::Branco,
                profundidade: Infinity,
                pai: None,
            };
            tab.inserir(&y, dec_y);
        }
    }
    tab
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
    let mut g = Grafo::novo(5);
    for i in 1..=5 {
        g.inserir_vertice(i);
    }
    g.inserir_elo(1,2);
    g.inserir_elo(1,3);
    g.inserir_elo(4,5);
    g.inserir_elo(3,4);
    println!("Fazendo a busca com o grafo conexo a partir do nó 1");
    let t = busca_largura(&g, 1);
    for i in g.lista_nos.into_iter() {
        println!("{i} : {:?}",t.ler(&i).unwrap());
    }
    g.remover_vertice(3);
    println!("Quando eliminamos o nó 3, passam a haver duas componentes conexas");
    let t2 = busca_largura(&g,1);
    for i in g.lista_nos.into_iter() {
        println!("{i} : {:?}",t2.ler(&i).unwrap());
    }

}
