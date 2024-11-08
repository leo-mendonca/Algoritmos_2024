// pub mod lista_encadeada;


pub(crate) use Algoritmos::lista_encadeada::{CelulaDupla, ListaDupla, escrever_arquivo, ler_arquivo};
use std::fs;
use std::io::Write;
use std::io;
use crossterm::{event, execute, terminal};
use crossterm::event::KeyCode;


const N_LER: usize = 50;

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

fn aguardar_enter() {
    //aguarda o usuario apertar a tecla Enter para prosseguir
    loop {
        let leitura: event::Event = event::read().expect("Não deve haver erro na leitura do teclado");
        if let event::Event::Key(chave) = leitura {
            if (chave.code==event::KeyCode::Enter) & (chave.kind==event::KeyEventKind::Press)  {return}
        }
    }
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
        execute!(io::stdout(),
            crossterm::terminal::Clear(terminal::ClearType::All),
        ).expect("Não devemos ter erro ao escrever no terminal");
        println!("{}\u{2038}{}\n\n",self.visivel_antes,self.visivel_depois); //\u{2038} é o caractere unicode do cursor
    }
    fn passo_direita(self:&mut Self) {
        if self.pos_cursor==(self.tamanho-1) as u32 {
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

    fn salvar(self: &Self) {
        println!("Salvando em {}",self.path);
        match std::fs::File::create(self.path.clone()) {
            Err(e) => {println!("Não foi possível abrir o arquivo\n{}",e);
                return;
            }
            Ok(arquivo) => {
                escrever_arquivo(arquivo, &self.lista_total);
                println!("Arquivo salvo com sucesso")
            }
        }
    }

    fn escrever(self: &mut Self, c: char) {
        self.lista_total.inserir_antes(self.ponteiro_cursor, c);
        let celula_inicio = unsafe {self.ponteiro_inicio_visivel.read()};
        self.visivel_antes.colocar(c);
        if self.visivel_antes.n ==N_LER+1 {
            self.visivel_antes.deletar_cabeca();
            self.ponteiro_inicio_visivel = celula_inicio.proximo.expect("A lista visivel_antes nao é vazia");
        }
        self.pos_cursor+=1;
        self.tamanho+=1;
    }

    fn deletar(self: &mut Self) {
        //Deleta o caractere na posicao do cursor
        //modificar: ponteiro_cursor, ponteiro_fim_visivel, tamanho, lista_total, visivel_depois
        if self.pos_cursor==(self.tamanho-1) as u32 { //se estivermos na ultima posicao:
            let (letra, endereco_anterior) = self.lista_total.anterior_mut(self.ponteiro_cursor).expect("O texto não pode ficar vazio");
            self.pos_cursor-=1;
            self.ponteiro_cursor=endereco_anterior;
            self.lista_total.deletar(self.lista_total.ponta);
            self.visivel_depois.inserir_antes(self.visivel_depois.cabeca, letra);
            self.visivel_depois.deletar(self.visivel_depois.ponta);
            self.visivel_antes.deletar(self.visivel_antes.ponta);
            if self.visivel_antes.n ==N_LER-1  {
                if let Some((letra, endereco)) = self.lista_total.anterior(self.ponteiro_inicio_visivel) {
                    self.visivel_antes.inserir_antes(self.visivel_antes.cabeca, letra);
                    self.ponteiro_inicio_visivel = endereco;
                }
            }
            return
        };
        let ponteiro_cursor_atual =self.ponteiro_cursor.clone();
        match self.lista_total.proxima_mut(self.ponteiro_cursor) {
            None => {return} //Neste caso estamos na ultima posicao, o que nao deve acontecer
            Some((_letra, ponteiro)) => {self.ponteiro_cursor = ponteiro;}
        }
        self.lista_total.deletar(ponteiro_cursor_atual);

        let celula_fim = unsafe {self.ponteiro_fim_visivel.read()};
        self.visivel_depois.deletar_cabeca();
        if let Some(endereco) =  celula_fim.proximo {
            self.visivel_depois.colocar(self.lista_total.ler(endereco));
            self.ponteiro_fim_visivel = endereco;
        }
        self.tamanho-=1;
    }
    fn backspace(self: &mut Self) {
        if self.pos_cursor==0 {return}
        self.passo_esquerda();
        self.deletar();
    }

    fn ir_final(self:&mut Self) {
        loop {
            self.passo_direita();
            if self.pos_cursor+1==self.tamanho as u32 {return}
        }
    }
    fn ir_inicio(self:&mut Self) {
        loop {
            self.passo_esquerda();
            if self.pos_cursor==0 {return}
        }
    }


    //todo() Implementar funcao para selecionar arquivo de entrada
}

pub fn main() {
    //Testes estáticos com o prelúdio de Memórias Póstumas de Brás Cubas:
    let mut ed=Editor::novo("Input/texto_teste.txt").expect("Erro na leitura do texto");
    println!("Lista visivel: \n{}",ed.visivel_depois);
    println!("Texto completo: \n{}", ed.lista_total);
    println!("{}\u{2038}{}\n\n",ed.visivel_antes,ed.visivel_depois);
    ed.passo_direita();
    println!("{}\u{2038}{}\n\n",ed.visivel_antes,ed.visivel_depois);
    ed.passo_esquerda();
    println!("{}\u{2038}{}\n\n",ed.visivel_antes,ed.visivel_depois);
    ed.andar_multiplos(-1);
    println!("{}\u{2038}{}\n\n",ed.visivel_antes,ed.visivel_depois);
    ed.andar_multiplos(10);
    println!("{}\u{2038}{}\n\n",ed.visivel_antes,ed.visivel_depois);
    ed.andar_multiplos(-10);
    println!("{}\u{2038}{}\n\n",ed.visivel_antes,ed.visivel_depois);


    println!("Teste Interativo com Esaú e Jacó (livro completo)\nPressione Enter para ler");
    aguardar_enter();

    ed = Editor::novo("Input/Esau e Jaco.txt").expect("O path deve ser válido");
    ed.exibir();
    crossterm::terminal::enable_raw_mode().expect("Não devemos ter erros ao habilitar o modo raw");
    loop {
        let leitura: event::Event = event::read().expect("Não deve haver erro na leitura do teclado");
        if let event::Event::Key(evento) = leitura {
            if evento.kind == event::KeyEventKind::Press { //Temos que checar se a tecla foi pressionada, senao cada input eh lido duas vezes
                //Comandos do usuario segurando Ctrl (e.g. salvar: Ctrl+s)
                if evento.modifiers == event::KeyModifiers::CONTROL {
                    match evento.code {
                        KeyCode::Char('s') => { ed.salvar() }
                        _ => {}
                    }
                }
                else {
                    match evento.code {
                        KeyCode::Backspace => {
                            ed.backspace();
                        }
                        KeyCode::Delete => {
                            ed.deletar();
                        }
                        KeyCode::Enter => {
                            ed.escrever('\n');
                        }
                        KeyCode::Left => {
                            ed.passo_esquerda();
                        }
                        KeyCode::Right => {
                            ed.passo_direita();
                        }
                        KeyCode::Up => {}
                        KeyCode::Down => {}
                        KeyCode::Home => {
                            ed.ir_inicio();
                        }
                        KeyCode::End => {
                            ed.ir_final();
                        }
                        KeyCode::PageDown => {
                            ed.andar_multiplos(N_LER as i32);
                        }
                        KeyCode::PageUp => {
                            ed.andar_multiplos(-(N_LER as i32));
                        }
                        KeyCode::Tab => {}
                        KeyCode::BackTab => {}

                        KeyCode::Insert => {}
                        KeyCode::Char(c) => {
                            ed.escrever(c);
                        }
                        KeyCode::Null => {}
                        KeyCode::Esc => {
                            crossterm::terminal::disable_raw_mode().expect("Desabilitar o modo raw do console");
                            break
                        }
                        KeyCode::CapsLock => {}
                        KeyCode::ScrollLock => {}
                        KeyCode::NumLock => {}
                        KeyCode::PrintScreen => {}
                        KeyCode::Pause => {}
                        KeyCode::Menu => {}
                        KeyCode::KeypadBegin => {}
                        KeyCode::Media(_) => {}
                        KeyCode::Modifier(_) => {}
                        KeyCode::F(_) => {}
                    }
                    ed.exibir();

                }
            }
        }
    }
}