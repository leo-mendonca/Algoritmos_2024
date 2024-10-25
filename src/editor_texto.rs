use crate::lista_encadeada::{CelulaDupla, ListaDupla};
use std::fs;
use std::io::{BufRead};
use std::io;


const N_LER: usize = 10;

struct Editor {
    path: String, //local do arquivo de texto
    pos_cursor: u32, //posicao do cursor no texto (entre 0 e tamanho do arquivo)
    ponteiro_cursor: *mut CelulaDupla<char>, //apontador para a celula da lista onde o cursor esta
    ponteiro_inicio_visivel: *const CelulaDupla<char>, //apontador para o inicio da parte visivel do texto na lista_total
    ponteiro_fim_visivel: *const CelulaDupla<char>, //apontador para o ultimo caractere da parte visivel do texto na lista_total
    tamanho: usize, //tamanho do arquivo
    lista_total: ListaDupla<char>, //lista encadeada contendo o texto inteiro do arquivo
    visivel_antes: ListaDupla<char>, //lista contendo os N_leitura caracteres antes do cursor
    visivel_depois: ListaDupla<char>, //lista contendo os N_leitura caracteres a partir do cursor (inclusive)
}

fn ler_arquivo(arquivo:fs::File) -> ListaDupla<char> {
    // let f = fs::File::open(path).expect("O path deve estar correto");
    let leitor = io::BufReader::new(arquivo);
    let mut lista :ListaDupla<char> = ListaDupla::<char>::novo();
    'varrendo_linhas: for linha in leitor.lines() {
        match linha {
            Ok(conteudo_linha) => {
                for letra in conteudo_linha.chars() {
                    lista.colocar(letra);
                }
                lista.colocar('\n');
                //todo() tratar o caso em que a ultima linha nao termina em \n
            }
            Err(_) => {break 'varrendo_linhas}
        }
    }
    return lista
}

impl Editor {
    fn novo(path: &str) -> Result<Editor, io::Error> {
        let arquivo = match fs::File::open(path) {
            Ok(f) => {f}
            Err(e) => {return Err(e)}
        };
        let lista_total: ListaDupla<char> = ler_arquivo(arquivo);
        let mut visivel_depois: ListaDupla<char> = ListaDupla::<char>::novo();
        let mut it = lista_total.into_iter();
        let mut ponteiro_fim_visivel: *const CelulaDupla<char> = lista_total.cabeca.clone();
        'lista_depois: for _i in 0..N_LER {
            match it.next() {
                Some(letra) => {
                    visivel_depois.colocar(letra);
                    ponteiro_fim_visivel = it.endereco_atual;
                },
                None => {
                    ponteiro_fim_visivel = it.endereco_atual;
                    break 'lista_depois; },
            }

        }
        Ok(Editor {
            path: String::from(path),
            pos_cursor: 0,
            ponteiro_cursor: lista_total.cabeca.clone(),
            ponteiro_inicio_visivel: lista_total.cabeca.clone(),
            ponteiro_fim_visivel: ponteiro_fim_visivel,
            tamanho: lista_total.n,
            lista_total: lista_total,
            visivel_antes: ListaDupla::<char>::novo(),
            visivel_depois: visivel_depois,
        })

    }

    fn exibir(self:&Self) {
        println!("{}\u{2038}{}",self.visivel_antes,self.visivel_depois); //\u{2038} e o caractere unicode do cursor
    }
    fn passo_direita(self:&mut Self) {
        if self.pos_cursor==self.tamanho as u32 {
            return
        }
        let celula_cursor = unsafe {self.ponteiro_cursor.read()};
        self.pos_cursor+=1;
        self.ponteiro_cursor = celula_cursor.proximo.expect("Se estivéssemos no final do arquivo, a função já teria retornado");
        let conteudo_cursor = celula_cursor.conteudo;
        self.visivel_antes.colocar(conteudo_cursor);
        assert!(self.visivel_antes.n<=N_LER+1);

        self.visivel_depois.deletar_cabeca();
        //alterando os apontadores de inicio e fim da parte visivel:
        let celula_inicio = unsafe {self.ponteiro_inicio_visivel.read()};
        if self.visivel_antes.n==N_LER+1 {
            self.visivel_antes.deletar_cabeca();
            self.ponteiro_inicio_visivel = celula_inicio.proximo.expect("O início da parte visível nunca vai ser o fim do arquivo, a menos que o cursor esteja no final");
        }
        let celula_fim = unsafe {self.ponteiro_fim_visivel.read()};
        match celula_fim.proximo {
            None => {}
            Some(apontador) => {
                let celula = unsafe {apontador.read()};
                self.visivel_depois.colocar(celula.conteudo);
                self.ponteiro_fim_visivel = apontador;
            }
        }
    }
    fn passo_esquerda(self:&mut Self) {
        if self.pos_cursor==0 {
            return
        }
        let celula_velho_cursor = unsafe {self.ponteiro_cursor.read()};
        self.pos_cursor-=1;
        self.ponteiro_cursor = celula_velho_cursor.anterior.expect("Se estivéssemos no início do arquivo, a função já teria retornado");
        let celula_novo_cursor = unsafe {self.ponteiro_cursor.read()};
        let novo_conteudo_cursor = celula_novo_cursor.conteudo;
        self.visivel_antes.deletar(self.visivel_antes.ponta);

        //alterando os apontadores de inicio e fim da parte visivel:
        let celula_fim = unsafe {self.ponteiro_fim_visivel.read()};
        if self.visivel_depois.n==N_LER {
            self.visivel_depois.deletar(self.visivel_depois.ponta);
            self.ponteiro_fim_visivel = celula_fim.anterior.expect("O fim da parte visível não é o primeiro caractere do arquivo, a menos que estejamos na extremidade (já tratado) e que o arquivo tenha uma só letra");

        }
        self.visivel_depois.inserir_antes(self.visivel_depois.cabeca, novo_conteudo_cursor);
        let celula_inicio = unsafe {self.ponteiro_inicio_visivel.read()};
        match celula_inicio.anterior {
            None => {}
            Some(apontador) => {
                let celula = unsafe {apontador.read()};
                self.visivel_antes.inserir_antes(self.visivel_antes.cabeca, celula.conteudo);
                self.ponteiro_inicio_visivel = apontador;
            }
        }
    }

    fn andar_multiplos(self:&mut Self, passos: i32) {
        if passos>=0 {
            for _i in 0..passos {
                self.passo_direita();
            }
        }
        else {
            for _i in 0..-passos {
                self.passo_esquerda();
            }
        }
    }

    //todo() Implementar funcoes para inserir caracteres e salvar a lista
}

pub fn main_editor() {
    let mut ed=Editor::novo("Input/texto_teste.txt").expect("Erro na leitura do texto");
    println!("Lista visivel: \n{}",ed.visivel_depois);
    println!("Texto completo: \n{}", ed.lista_total);
    ed.exibir();
    ed.passo_direita();
    ed.exibir();
    ed.passo_esquerda();
    ed.exibir();
    ed.andar_multiplos(-1);
    ed.exibir();
    ed.andar_multiplos(10);
    ed.exibir();
    ed.andar_multiplos(-5);
    ed.exibir();
}