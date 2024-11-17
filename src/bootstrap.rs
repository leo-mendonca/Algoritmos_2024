use rand::random;
use ndarray::{Array2};
use ndarray;



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

struct FamiliaUpdate {
    //Conjunto de conjuntos X \in Z^2 tal que, se y+X está infectado, então y se torna infectado
    v: Vec<Vec<[i32;2]>>
}

struct ProcessoBootstrap {
    // tabuleiro: Vec<bool>,
    tabuleiro: Array2<bool>,
    fam_update: FamiliaUpdate,
    n:usize,
}

impl ProcessoBootstrap {
    fn novo(n: usize, p:f64, fam_update: FamiliaUpdate) -> Self {
        let tabuleiro = gerar_estado_inicial(p,n,n);
        ProcessoBootstrap{
            tabuleiro,
            fam_update,
            n
        }
    }
    fn atualiza_passo(self: &mut Self) {
        //Versão incial "burra", sem paralelização
        let mut tab_novo = self.tabuleiro.clone();
        for i in 0..self.n {
            for j in 0..self.n {
               if !self.tabuleiro[[i,j]] {
                   tab_novo[[i,j]] = checar_propagacao(&self.tabuleiro, &self.fam_update, i, j)
               }
            }
        }
        self.tabuleiro = tab_novo;
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
    // let A = gerar_estado_inicial(0.5,2,2);
    let mut boot = ProcessoBootstrap::novo(3,0.5,FAMILIA_UPDATE);
    println!("Estado inicial:\n{:?}",boot.tabuleiro);
    boot.atualiza_passo();
    println!("Atualizando o tabuleiro uma vez:\n{:?}",boot.tabuleiro);
    boot.atualiza_passo();
    println!("Atualizando o tabuleiro de novo:\n{:?}",boot.tabuleiro);
}
