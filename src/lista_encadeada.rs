use std::io::Write;
use std::alloc::{alloc, dealloc, Layout};
use std::fmt::{Display, Formatter};
use std::{fs, io};
use std::io::BufRead;
// use std::ptr::write;

pub struct CelulaSimples<T> {
    pub conteudo: T,
    pub proximo: Option<*mut CelulaSimples<T>>,
}
pub struct CelulaDupla<T> {
    pub conteudo: T,
    pub proximo: Option<*mut CelulaDupla<T>>,
    pub anterior: Option<*mut CelulaDupla<T>>,
}

pub struct ListaEncadeada<T> {
    pub n: usize,
    pub cabeca: *mut CelulaSimples<T>,
    pub ponta: *mut CelulaSimples<T>,
    // tamanho_unidade: usize,
}
impl<T> ListaEncadeada<T> {
    pub fn novo() ->Self {

        let layout:Layout = Layout::new::<CelulaSimples<T>>();

        let cabeca: *mut CelulaSimples<T> = unsafe {alloc(layout)} as *mut CelulaSimples<T>;
        ListaEncadeada {
            n: 0,
            cabeca: cabeca,
            ponta: cabeca.clone(),
        }
    }
    pub fn colocar(self: &mut Self,elemento: T) {
        let nova_celula: CelulaSimples<T> = CelulaSimples {
            conteudo: elemento,
            proximo: None,
        };
        if self.n==0 {
            unsafe {self.cabeca.write(nova_celula)};
            self.n+=1;
        }
        else {
            let layout:Layout = Layout::new::<CelulaSimples<T>>();
            let ponteiro: *mut CelulaSimples<T>  = unsafe {alloc(layout) as *mut CelulaSimples<T>};
            let mut penultima_celula = unsafe { self.ponta.read() };
            penultima_celula.proximo = Some(ponteiro);
            unsafe { self.ponta.write(penultima_celula) };
            unsafe { ponteiro.write(nova_celula) };
            self.ponta = ponteiro;
            self.n += 1;
        }
    }
    pub fn inserir_apos(self:&mut Self, endereco: *mut CelulaSimples<T>, conteudo: T) {
        //Insere o caractere 'conteudo' na celula imediatamente apos a celula que esta em 'endereco'
        //Identificando a celula atual (que sera a anterior à nova):
        let mut celula_anterior:CelulaSimples<T> = unsafe {endereco.read()};
        let celula_nova: CelulaSimples<T> = CelulaSimples {
            conteudo: conteudo,
            proximo: celula_anterior.proximo,
        };
        let layout:Layout = Layout::new::<CelulaSimples<T>>();
        unsafe { //modificando o apontador da celula anterior para apontar para a celula inserida
            let apontador_novo:*mut CelulaSimples<T> =alloc(layout) as *mut CelulaSimples<T>;
            apontador_novo.write(celula_nova);
            celula_anterior.proximo = Some(apontador_novo);
            endereco.write(celula_anterior);
        };
        self.n+=1;
    }
    fn proxima_mut(self:&mut Self, endereco: *mut CelulaSimples<T>) -> Option<(T, *mut CelulaSimples<T>)> {
        if endereco==self.ponta {
            panic!()
        }
        unsafe {
            let celula_atual: CelulaSimples<T> =  {endereco.read()};
            match celula_atual.proximo {
                None => {None}
                Some(ponteiro) => {
                    let proxima_celula: CelulaSimples<T> =  {ponteiro.read()};
                    Some((proxima_celula.conteudo, ponteiro))
                }
            }

        }
    }
    fn proxima(self:&Self, endereco: *const CelulaSimples<T>) -> Option<(T, *const CelulaSimples<T>)> {
        if endereco == self.ponta {
            panic!()
        }
        unsafe {
            let celula_atual: CelulaSimples<T> = { endereco.read() };
            match celula_atual.proximo {
                None => { None }
                Some(ponteiro) => {
                    let proxima_celula: CelulaSimples<T> = { ponteiro.read() };
                    Some((proxima_celula.conteudo, ponteiro))
                }
            }
        }
    }
    fn ler_cabeca(self: &Self) ->(T, Option<*const CelulaSimples<T>>) {
        //A lista nao pode estar vazia, ou seja, a cabeca tem que ter conteudo
        let celula: CelulaSimples<T> = unsafe {self.cabeca.read()};
        let conteudo: T = celula.conteudo;
        match celula.proximo {
            Some(apontador) => (conteudo, Some(apontador)),
            None => (conteudo, None),
        }
    }

    pub fn alterar(self: &Self, endereco: *mut CelulaSimples<T>, conteudo: T) {
        let mut celula = unsafe {endereco.read()};
        celula.conteudo=conteudo;
        unsafe {endereco.write(celula)};
    }
    pub fn deletar_apos(self: &mut Self, endereco: *mut CelulaSimples<T>) {
        assert!(endereco!=self.ponta);
        //Deleta a celula seguinte àquela do endereço fornecido
        let mut celula_anterior: CelulaSimples<T> = unsafe {endereco.read()};
        let ponteiro_remover:*mut CelulaSimples<T> = celula_anterior.proximo.expect("O ponteiro da celula não deve ser None, pois ela não pode ser a ponta!");
        let celula_a_remover: CelulaSimples<T> = unsafe {ponteiro_remover.read()};
        //Alterando o apontador da célula anterior para "pular" a célula deletada
        celula_anterior.proximo = celula_a_remover.proximo.clone();
        unsafe { endereco.write(celula_anterior) };
        self.n-=1;
        //Se a célula removida for a ponta da lista, a célula anterior vira a nova ponta:
        if ponteiro_remover==self.ponta {self.ponta = endereco}
        //Desalocando a memoria:
        let layout_remover: Layout = Layout::new::<CelulaSimples<T>>();
        unsafe { dealloc(ponteiro_remover as *mut u8, layout_remover) };
    }
    pub fn deletar_cabeca(self: &mut Self) {
        assert!(self.n>0);
        if self.n>1 {
            let cabeca_atual = self.cabeca;
            let celula_cabeca = unsafe { cabeca_atual.read() };
            self.cabeca = celula_cabeca.proximo.expect("O apontador da nova cabeça não pode ser None, pois a lista não é vazia");
            let layout_remover: Layout = Layout::new::<CelulaSimples<T>>();
            unsafe { dealloc(cabeca_atual as *mut u8, layout_remover) };
        }
        self.n-=1;
    }


}
#[derive(Debug)]
pub struct ListaDupla<T> {
    pub n: usize,
    pub cabeca: *mut CelulaDupla<T>,
    pub ponta: *mut CelulaDupla<T>,
}
impl<T> ListaDupla<T> {
    pub fn novo() ->Self {

        let layout:Layout = Layout::new::<CelulaDupla<T>>();

        let cabeca: *mut CelulaDupla<T> = unsafe {alloc(layout)} as *mut CelulaDupla<T>;
        ListaDupla {
            n: 0,
            cabeca: cabeca,
            ponta: cabeca.clone(),
        }
    }
    pub fn colocar(self: &mut Self,elemento: T) {
        //Insere um elemento na ponta da lista

        if self.n==0 {
            let nova_celula = CelulaDupla {
                conteudo: elemento,
                proximo: None,
                anterior: None,
            };
            unsafe {self.cabeca.write(nova_celula)};
        }
        else {
            let nova_celula: CelulaDupla<T> = CelulaDupla {
                conteudo: elemento,
                proximo: None,
                anterior: Some(self.ponta.clone()),
            };
            let layout:Layout = Layout::new::<CelulaDupla<T>>();
            let ponteiro: *mut CelulaDupla<T>  = unsafe {alloc(layout) as *mut CelulaDupla<T>};
            let mut penultima_celula = unsafe { self.ponta.read() };
            penultima_celula.proximo = Some(ponteiro);
            unsafe { self.ponta.write(penultima_celula) };
            unsafe { ponteiro.write(nova_celula) };
            self.ponta = ponteiro;
        }
        self.n += 1;
    }
    pub fn inserir_apos(self:&mut Self, endereco: *mut CelulaDupla<T>, conteudo: T) {
        //Insere o caractere 'conteudo' na celula imediatamente apos a celula que esta em 'endereco'
        //Identificando a celula atual (que sera a anterior à nova):
        let mut celula_anterior:CelulaDupla<T> = unsafe {endereco.read()};
        let end_seguinte: Option<*mut CelulaDupla<T>> = celula_anterior.proximo;

        let celula_seguinte: Option<CelulaDupla<T>> = match end_seguinte{
            Some(ponteiro) =>Some(unsafe { ponteiro.read() }),
            None => None,
        };
        let celula_nova: CelulaDupla<T> = CelulaDupla {
            conteudo: conteudo,
            proximo: end_seguinte.clone(),
            anterior: Some(endereco.clone()),
        };
        let layout:Layout = Layout::new::<CelulaDupla<T>>();
        let apontador_novo = unsafe { //modificando os apontadores da celula anterior e da seguinte para apontar para a celula inserida
            let apontador_novo:*mut CelulaDupla<T> =alloc(layout) as *mut CelulaDupla<T>;
            apontador_novo.write(celula_nova);
            celula_anterior.proximo = Some(apontador_novo);
            endereco.write(celula_anterior);
            match celula_seguinte {
                Some(mut celula) => {
                    celula.anterior = Some(apontador_novo);
                    end_seguinte.expect("Sabemos que é um ponteiro").write(celula);
                },
                None => {},
            }
            apontador_novo
        };
        if endereco==self.ponta {self.ponta = apontador_novo}
        self.n+=1;
    }
    pub fn inserir_antes(self: &mut Self,endereco: *mut CelulaDupla<T>, conteudo: T) {
        if endereco==self.cabeca  {
            self.inserir_cabeca(conteudo)
        }
        else {
            let celula: CelulaDupla<T> = unsafe {endereco.read()};
            self.inserir_apos(celula.anterior.expect("A célula anterior não é vazia, pois o endereço que foi passado não é a cabeça"), conteudo);
        }

    }
    fn inserir_cabeca(self: &mut Self, conteudo: T) {
        let cabeca_atual: *mut CelulaDupla<T> = self.cabeca.clone();
        let mut celula_atual = unsafe {cabeca_atual.read()};
        let layout: Layout = Layout::new::<CelulaDupla<T>>();
        let nova_cabeca: *mut CelulaDupla<T> = unsafe {alloc(layout) as *mut CelulaDupla<T>};
        let nova_celula: CelulaDupla<T> = CelulaDupla {
            conteudo: conteudo,
            anterior: None,
            proximo: Some(cabeca_atual),
        };
        celula_atual.anterior=Some(nova_cabeca);
        unsafe {
            cabeca_atual.write(celula_atual);
            nova_cabeca.write(nova_celula);
        }
        self.cabeca=nova_cabeca;
        self.n+=1;
    }
    pub fn proxima_mut(self:&mut Self, endereco: *mut CelulaDupla<T>) -> Option<(T, *mut CelulaDupla<T>)>{
        match self.proxima(endereco as *const CelulaDupla<T>) {
            None =>None,
            Some((conteudo, ponteiro)) => Some((conteudo, ponteiro as *mut CelulaDupla<T>))
        }
    }
    pub fn proxima(self:&Self, endereco: *const CelulaDupla<T>) -> Option<(T, *const CelulaDupla<T>)> {
        unsafe {
            let celula_atual: CelulaDupla<T> = { endereco.read() };
            match celula_atual.proximo {
                None => { None }
                Some(ponteiro) => {
                    let proxima_celula: CelulaDupla<T> = { ponteiro.read() };
                    Some((proxima_celula.conteudo, ponteiro))
                }
            }
        }
    }

    fn ler_cabeca(self: &Self) ->(T, Option<*const CelulaDupla<T>>) {
        //A lista nao pode estar vazia, ou seja, a cabeca tem que ter conteudo
        let celula: CelulaDupla<T> = unsafe {self.cabeca.read()};
        let conteudo: T = celula.conteudo;
        match celula.proximo {
            Some(apontador) => (conteudo, Some(apontador)),
            None => (conteudo, None),
        }
    }
    pub fn ler(self: &Self, endereco: *const CelulaDupla<T>) -> T {
        let celula = unsafe {endereco.read()};
        celula.conteudo
    }
    pub fn alterar(self: &Self, endereco: *mut CelulaDupla<T>, conteudo: T) {
        let mut celula = unsafe {endereco.read()};
        celula.conteudo=conteudo;
        unsafe {endereco.write(celula)};
    }
    pub fn deletar_apos(self: &mut Self, endereco: *mut CelulaDupla<T>) {
        assert!(endereco!=self.ponta);
        //Deleta a celula seguinte àquela do endereço fornecido
        let mut celula_anterior: CelulaDupla<T> = unsafe {endereco.read()};
        let ponteiro_remover:*mut CelulaDupla<T> = celula_anterior.proximo.expect("A célula passada não deve ser a ponta!");
        let celula_a_remover: CelulaDupla<T> = unsafe {ponteiro_remover.read()};
        //Alterando o apontador da célula anterior para "pular" a célula deletada
        celula_anterior.proximo = celula_a_remover.proximo.clone();

        unsafe { endereco.write(celula_anterior) };
        self.n-=1;

        //Se a célula removida for a ponta da lista, a célula anterior vira a nova ponta:
        if ponteiro_remover==self.ponta {self.ponta = endereco}
        else  {
            let prox = celula_a_remover.proximo.expect("Não deve ser None, pois não estamos na ponta");
            let mut celula_seguinte: CelulaDupla<T> = unsafe { prox.read() };
            celula_seguinte.anterior=Some(endereco.clone());
            unsafe {prox.write(celula_seguinte)};
        }
        //Desalocando a memoria:
        let layout_remover: Layout = Layout::new::<CelulaDupla<T>>();
        unsafe { dealloc(ponteiro_remover as *mut u8, layout_remover) };
    }
    pub fn deletar(self: &mut Self, endereco: *mut CelulaDupla<T>) {
        //deleta a célula no endereço dado
        if endereco==self.cabeca {self.deletar_cabeca()}
        else {
            let celula = unsafe {endereco.read()};
            self.deletar_apos(celula.anterior.expect("Deve haver uma célula anterior, pois não estamos na cabeça"))
        }
    }
    pub fn deletar_cabeca(self: &mut Self) {
        assert!(self.n>0);
        let cabeca_atual=self.cabeca;
        let celula_cabeca = unsafe {cabeca_atual.read()};
        if self.n>1 {
            self.cabeca = celula_cabeca.proximo.expect("Deve haver alguma célula depois da cabeça");
            let layout_remover: Layout = Layout::new::<CelulaDupla<T>>();
            unsafe { dealloc(cabeca_atual as *mut u8, layout_remover) };
            let mut celula_seguinte = unsafe {self.cabeca.read()};
            celula_seguinte.anterior = None;
            unsafe { self.cabeca.write(celula_seguinte)};
        }
        self.n-=1;
    }

    pub fn anterior_mut(self:&Self, endereco: *mut CelulaDupla<T>) -> Option<(T, *mut CelulaDupla<T>)> {
        if endereco==self.cabeca {return None}
        let celula_atual: CelulaDupla<T> = unsafe {endereco.read()};
        let anterior = celula_atual.anterior.expect("Não estamos na cabeça");
        let celula_anterior: CelulaDupla<T> = unsafe{anterior.read()};
        Some((celula_anterior.conteudo, anterior))
    }
    pub fn anterior(self:&Self, endereco: *const CelulaDupla<T>) -> Option<(T, *const CelulaDupla<T>)> {
        match self.anterior_mut(endereco as *mut CelulaDupla<T>) {
            None => None,
            Some((conteudo, endereco)) => Some((conteudo, endereco as *const CelulaDupla<T>)),
        }
    }
}

pub struct IteradorLista<'a, T> {
    lista: &'a ListaEncadeada<T>,
    endereco_atual: *const CelulaSimples<T>,
    contagem: u32,
}
impl<'b,T> Iterator for IteradorLista<'b,T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.contagem==self.lista.n as u32 {None}
        else if self.contagem==0 {
            self.contagem+=1;
            Some(self.lista.ler_cabeca().0)
        }
        else {
            match self.lista.proxima(self.endereco_atual) {
                Some((conteudo, proximo_endereco)) => {
                    self.endereco_atual=proximo_endereco;
                    self.contagem+=1;
                    Some(conteudo)
                },
                None => None,
            }
        }
    }
}

impl<'a,T> IntoIterator for &'a ListaEncadeada<T> where T:'a {
    //Implementando um iterador para poder usar loops do tipo for com a lista encadeada
    type Item = T;

    type IntoIter = IteradorLista<'a,T> where T:'a;

    fn into_iter(self) -> Self::IntoIter {
        IteradorLista {
            lista: &self,
            endereco_atual: self.cabeca.clone(),
            contagem: 0,
        }
    }
}

pub struct IteradorListaDupla<'a, T> {
    lista: &'a ListaDupla<T>,
    pub endereco_atual: *const CelulaDupla<T>,
    contagem: u32,
}
impl<'b,T> Iterator for IteradorListaDupla<'b, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.contagem==self.lista.n as u32 {None}
        else if self.contagem==0 {
            self.contagem+=1;
            Some(self.lista.ler_cabeca().0)
        }
        else {
            match self.lista.proxima(self.endereco_atual) {
                Some((conteudo, proximo_endereco)) => {
                    self.endereco_atual=proximo_endereco;
                    self.contagem+=1;
                    Some(conteudo)
                },
                None => None,
            }
        }
    }
}

impl<'a,T> IntoIterator for &'a ListaDupla<T> where T:'a {
    //Implementando um iterador para poder usar loops do tipo for com a lista encadeada
    type Item = T;

    type IntoIter = IteradorListaDupla<'a, T>
    where T:'a;

    fn into_iter(self) -> Self::IntoIter {
        IteradorListaDupla {
            lista: &self,
            endereco_atual: self.cabeca.clone(),
            contagem: 0,
        }
    }
}



impl<T:Display> ListaEncadeada<T> {
    fn imprimir_lista(self: &Self) {
        for s in self.into_iter() {
            print!("{}",s);
        }
        print!("\n");
    }
}
impl<T:Display> Display for ListaEncadeada<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if std::any::type_name::<T>() =="char" {
            Ok(for s in self.into_iter() {
                match write!(f, "{}", s) {
                    Ok(_) => {},
                    Err(e) => {return Err(e)},
                };
            })
        }
        else {
            Ok(for s in self.into_iter() {
                match write!(f,"{},",s) {
                    Ok(_) => {}
                    Err(e) => {return Err(e)}
                };
            })
        }
    }
}

impl<T:Display> ListaDupla<T> {
    fn imprimir_lista(self: &Self) {
        for s in self.into_iter() {
            print!("{}",s);
        }
        print!("\n");
    }
}
impl<T:Display> Display for ListaDupla<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if std::any::type_name::<T>() =="char" {
            Ok(for s in self.into_iter() {
                match write!(f, "{}", s) {
                    Ok(_) => {}
                    Err(e) => {return Err(e)}
                };
            })
        }
        else {
            Ok(for s in self.into_iter() {
                match write!(f,"{},",s) {
                    Ok(_) => {}
                    Err(e) => {return Err(e)}
                };
            })
        }
    }
}

pub fn ler_arquivo(arquivo:fs::File) -> ListaDupla<char> {
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

pub fn escrever_arquivo(mut arquivo:fs::File, lista:&ListaDupla<char>) {
    // let mut escritor = io::BufWriter::new(arquivo);
    for letra in lista.into_iter() {
        write!(&mut arquivo, "{}",letra).expect("Erro ao salvar o arquivo");
    }
}

#[test]
fn _teste_bom_dia() {
    let mut lista:ListaEncadeada<char>=ListaEncadeada::novo();
    println!("Início");
    let mensagem: &str ="Bom dia!";
    let mut mensagem_fila = mensagem.chars();
    let mut letra_correta:char;
    lista.colocar('A');
    assert_eq!("A",format!("{}",lista));
    lista.deletar_cabeca();
    for letra in mensagem.chars() {
        lista.colocar(letra);
        println!("{}",letra);
        letra_correta=mensagem_fila.next().unwrap();
        assert_eq!(letra_correta, letra);
    }
    assert_eq!("Bom dia!", format!("{}",lista));
    println!("Escrevi\nLendo:");
    let mut endereco: *mut CelulaSimples<char> = lista.cabeca;
    let conteudo: char;
    let mut pos_inserir: *mut CelulaSimples<char> = lista.cabeca;
    unsafe { conteudo = lista.cabeca.read().conteudo; }
    print!("{}",conteudo);
    for _i in 1..mensagem.len() {
        let ( conteudo, end) = lista.proxima_mut(endereco).expect("Não chegamos ao fim da lista");
        endereco=end;
        print!("{}",conteudo);
        if conteudo=='m' { pos_inserir =endereco }
    }
    lista.inserir_apos(pos_inserir, 's');
    println!("\nTerminei de inserir");
    println!("{}",lista);
    assert_eq!("Boms dia!", format!("{}",lista));
    println!("Inserindo exclamação!");
    let (_c, pos_apos) = lista.proxima_mut(pos_inserir).expect("Não chegamos ao fim da lista");
    lista.alterar(pos_apos, '!');
    println!("{}",lista);
    assert_eq!("Bom! dia!", format!("{}",lista));
    println!("Removendo exclamação!");
    lista.deletar_apos(pos_inserir);
    println!("{}",lista);
    assert_eq!("Bom dia!", format!("{}",lista));
    println!("Removendo a primeira palavra");
    for _i in 1..=4 {
        lista.deletar_cabeca();
    }
    println!("{}",lista);
    assert_eq!("dia!", format!("{}",lista));
}
#[test]
fn _teste_numerico() {
    println!("Iniciando teste com dados numéricos e iterador");
    let numeros: [i32; 5] = [10, 20, 30, 40, 50];
    let mut lista: ListaEncadeada<i32> = ListaEncadeada::novo();
    for n in numeros {
        lista.colocar(n) }
    for elem in lista.into_iter() {
        println!("{},",elem);
    }
    println!("{}",lista);
    let s = String::from("10,20,30,40,50,");
    // let str_lista = String::from(lista);
    assert_eq!(s, format!("{}",lista));
}
#[test]
fn _teste_lista_dupla(){
    let mut lista: ListaDupla<char> = ListaDupla::novo();
    let mensagem: &str ="Palavra";
    for letra in mensagem.chars() {
        lista.colocar(letra);
        print!("{}",letra);
    }
    assert_eq!(format!("{}",lista), "Palavra");
    let mut endereco: *mut CelulaDupla<char> = lista.ponta;
    println!("\nInserindo exclamações entre as letras");
    for _i in 1..mensagem.len() {
        lista.inserir_antes(endereco, '!');
        let (_c, end) = lista.anterior_mut(endereco).expect("Não estamos no início da lista");
        let (_c, end) = lista.anterior_mut(end).expect("Não estamos no início da lista");
        endereco=end;
    }
    println!("{}",lista);
    assert_eq!(format!("{}",lista), "P!a!l!a!v!r!a");
}
