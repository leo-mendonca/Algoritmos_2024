use std::alloc::{alloc, dealloc, Layout};
use std::fmt::{Display, Formatter};
use std::ptr::write;

struct CelulaSimples<T> {
    conteudo: T,
    proximo: *mut CelulaSimples<T>,
}
struct CelulaDupla<T> {
    conteudo: T,
    proximo: *mut CelulaDupla<T>,
    anterior: *mut CelulaDupla<T>,
}

pub struct ListaEncadeada<T> {
    n: u32,
    cabeca: *mut CelulaSimples<T>,
    ponta: *mut CelulaSimples<T>,
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
            proximo: self.cabeca,
        };
        if self.n==0 {
            unsafe {self.cabeca.write(nova_celula)};
            self.n+=1;
        }
        else {
            let layout:Layout = Layout::new::<CelulaSimples<T>>();
            let ponteiro: *mut CelulaSimples<T>  = unsafe {alloc(layout) as *mut CelulaSimples<T>};
            let mut penultima_celula = unsafe { self.ponta.read() };
            penultima_celula.proximo = ponteiro;
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
            celula_anterior.proximo = apontador_novo;
            endereco.write(celula_anterior);
        };
        self.n+=1;
    }
    fn proxima_mut(self:&mut Self, endereco: *mut CelulaSimples<T>) -> (T, *mut CelulaSimples<T>) {
        if endereco==self.ponta {
            panic!()
        }
        unsafe {
            let celula_atual: CelulaSimples<T> =  {endereco.read()};
            let end_prox:*mut CelulaSimples<T> = celula_atual.proximo;
            let proxima_celula: CelulaSimples<T> =  {end_prox.read()};
            (proxima_celula.conteudo, (end_prox))
        }
    }
    fn proxima(self:&Self, endereco: *const CelulaSimples<T>) -> (T, *const CelulaSimples<T>) {
        if endereco==self.ponta {
            panic!()
        }
        unsafe {
            let celula_atual: CelulaSimples<T> =  {endereco.read()};
            let end_prox:*const CelulaSimples<T> = celula_atual.proximo;
            let proxima_celula: CelulaSimples<T> =  {end_prox.read()};
            (proxima_celula.conteudo, (end_prox))
        }
    }
    fn ler_cabeca(self: &Self) ->(T,*const CelulaSimples<T>) {
        let celula: CelulaSimples<T> = unsafe {self.cabeca.read()};
        let conteudo: T = celula.conteudo;
        let apontador: *const CelulaSimples<T> = celula.proximo;
        (conteudo, apontador)
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
        let ponteiro_remover:*mut CelulaSimples<T> = celula_anterior.proximo;
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
        let cabeca_atual=self.cabeca;
        let celula_cabeca = unsafe {cabeca_atual.read()};
        self.cabeca = celula_cabeca.proximo.clone();
        let layout_remover: Layout = Layout::new::<CelulaSimples<T>>();
        unsafe { dealloc(cabeca_atual as *mut u8, layout_remover) };
        self.n-=1;
    }


}

pub struct ListaDupla<T> {
    n: u32,
    cabeca: *mut CelulaDupla<T>,
    ponta: *mut CelulaDupla<T>,
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
        let nova_celula: CelulaDupla<T> = CelulaDupla {
            conteudo: elemento,
            proximo: self.cabeca.clone(),
            anterior: self.ponta.clone(),
        };
        if self.n==0 {
            unsafe {self.cabeca.write(nova_celula)};
        }
        else {
            let layout:Layout = Layout::new::<CelulaDupla<T>>();
            let ponteiro: *mut CelulaDupla<T>  = unsafe {alloc(layout) as *mut CelulaDupla<T>};
            let mut penultima_celula = unsafe { self.ponta.read() };
            penultima_celula.proximo = ponteiro;
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
        let end_seguinte: *mut CelulaDupla<T> = celula_anterior.proximo;
        let mut celula_seguinte: CelulaDupla<T> = unsafe { end_seguinte.read()};
        let celula_nova: CelulaDupla<T> = CelulaDupla {
            conteudo: conteudo,
            proximo: end_seguinte.clone(),
            anterior: endereco.clone(),
        };
        let layout:Layout = Layout::new::<CelulaDupla<T>>();
        unsafe { //modificando os apontadores da celula anterior e da seguinte para apontar para a celula inserida
            let apontador_novo:*mut CelulaDupla<T> =alloc(layout) as *mut CelulaDupla<T>;
            apontador_novo.write(celula_nova);
            celula_anterior.proximo = apontador_novo;
            celula_seguinte.anterior = apontador_novo;
            endereco.write(celula_anterior);
            end_seguinte.write(celula_seguinte);
        };
        if endereco==self.ponta {self.ponta = end_seguinte}
        self.n+=1;
    }
    pub fn inserir_antes(self: &mut Self,endereco: *mut CelulaDupla<T>, conteudo: T) {
        match endereco==self.cabeca {
            true => {self.inserir_cabeca(conteudo)},
            false => {
                let celula: CelulaDupla<T> = unsafe {endereco.read()};
                self.inserir_apos(celula.anterior, conteudo);
            }
        }
    }
    fn inserir_cabeca(self: &mut Self, conteudo: T) {
        let cabeca_atual: *mut CelulaDupla<T> = self.cabeca.clone();
        let mut celula_atual = unsafe {cabeca_atual.read()};
        let layout: Layout = Layout::new::<CelulaDupla<T>>();
        let nova_cabeca: *mut CelulaDupla<T> = unsafe {alloc(layout) as *mut CelulaDupla<T>};
        let nova_celula: CelulaDupla<T> = CelulaDupla {
            conteudo: conteudo,
            anterior: nova_cabeca,
            proximo: cabeca_atual,
        };
        celula_atual.anterior=nova_cabeca;
        unsafe {
            cabeca_atual.write(celula_atual);
            nova_cabeca.write(nova_celula);
        }
        self.cabeca=nova_cabeca;
        self.n+=1;
    }
    fn proxima_mut(self:&mut Self, endereco: *mut CelulaDupla<T>) -> (T, *mut CelulaDupla<T>) {
        if endereco==self.ponta {
            panic!()
        }
        unsafe {
            let celula_atual: CelulaDupla<T> =  {endereco.read()};
            let end_prox:*mut CelulaDupla<T> = celula_atual.proximo;
            let proxima_celula: CelulaDupla<T> =  {end_prox.read()};
            (proxima_celula.conteudo, (end_prox))
        }
    }
    fn proxima(self:&Self, endereco: *const CelulaDupla<T>) -> (T, *const CelulaDupla<T>) {
        if endereco==self.ponta {
            panic!()
        }
        unsafe {
            let celula_atual: CelulaDupla<T> =  {endereco.read()};
            let end_prox:*const CelulaDupla<T> = celula_atual.proximo;
            let proxima_celula: CelulaDupla<T> =  {end_prox.read()};
            (proxima_celula.conteudo, (end_prox))
        }
    }
    fn ler_cabeca(self: &Self) ->(T,*const CelulaDupla<T>) {
        let celula: CelulaDupla<T> = unsafe {self.cabeca.read()};
        let conteudo: T = celula.conteudo;
        let apontador: *const CelulaDupla<T> = celula.proximo;
        (conteudo, apontador)
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
        let ponteiro_remover:*mut CelulaDupla<T> = celula_anterior.proximo;
        let celula_a_remover: CelulaDupla<T> = unsafe {ponteiro_remover.read()};
        //Alterando o apontador da célula anterior para "pular" a célula deletada
        celula_anterior.proximo = celula_a_remover.proximo.clone();

        unsafe { endereco.write(celula_anterior) };
        self.n-=1;

        //Se a célula removida for a ponta da lista, a célula anterior vira a nova ponta:
        if ponteiro_remover==self.ponta {self.ponta = endereco}
        else  {
            let mut celula_seguinte: CelulaDupla<T> = unsafe { celula_a_remover.proximo.read() };
            celula_seguinte.anterior=endereco.clone();
            unsafe {celula_a_remover.proximo.write(celula_seguinte)};
        }
        //Desalocando a memoria:
        let layout_remover: Layout = Layout::new::<CelulaDupla<T>>();
        unsafe { dealloc(ponteiro_remover as *mut u8, layout_remover) };
    }
    pub fn deletar(self: &mut Self, endereco: *mut CelulaDupla<T>) {
        //deleta a célula no endereço dado
        match endereco==self.cabeca {
            true=> self.deletar_cabeca(),
            false => {
                let celula = unsafe {endereco.read()};
                self.deletar_apos(celula.anterior)
            },
        }
    }
    pub fn deletar_cabeca(self: &mut Self) {
        assert!(self.n>0);
        let cabeca_atual=self.cabeca;
        let celula_cabeca = unsafe {cabeca_atual.read()};
        self.cabeca = celula_cabeca.proximo.clone();
        let layout_remover: Layout = Layout::new::<CelulaDupla<T>>();
        unsafe { dealloc(cabeca_atual as *mut u8, layout_remover) };
        self.n-=1;
    }

    fn anterior(self:&mut Self, endereco: *mut CelulaDupla<T>) -> (T,*mut CelulaDupla<T>) {
        if endereco==self.cabeca {panic!()}
        let celula_atual: CelulaDupla<T> = unsafe {endereco.read()};
        let celula_anterior: CelulaDupla<T> = unsafe{celula_atual.anterior.read()};
        (celula_anterior.conteudo, celula_atual.anterior)
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
        if self.contagem==self.lista.n {None}
        else if self.contagem==0 {
            let (conteudo, _proximo_endereco) : (T, *const CelulaSimples<T>) = self.lista.ler_cabeca();
            self.contagem+=1;
            Some(conteudo)
        }
        else {
            let (conteudo, proximo_endereco): (T, *const CelulaSimples<T>) = self.lista.proxima(self.endereco_atual);
            self.endereco_atual=proximo_endereco;
            self.contagem+=1;
            Some(conteudo)
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
    endereco_atual: *const CelulaDupla<T>,
    contagem: u32,
}
impl<'b,T> Iterator for IteradorListaDupla<'b, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.contagem==self.lista.n {None}
        else if self.contagem==0 {
            let (conteudo, _proximo_endereco) : (T, *const CelulaDupla<T>) = self.lista.ler_cabeca();
            self.contagem+=1;
            Some(conteudo)
        }
        else {
            let (conteudo, proximo_endereco): (T, *const CelulaDupla<T>) = self.lista.proxima(self.endereco_atual);
            self.endereco_atual=proximo_endereco;
            self.contagem+=1;
            Some(conteudo)
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
                write!(f, "{}", s);
            })
        }
        else {
            Ok(for s in self.into_iter() {
                write!(f,"{},",s);
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
                write!(f, "{}", s);
            })
        }
        else {
            Ok(for s in self.into_iter() {
                write!(f,"{},",s);
            })
        }
    }
}



fn _teste_bom_dia() {
    let mut lista:ListaEncadeada<char>=ListaEncadeada::novo();
    println!("Início");
    let mensagem: &str ="Bom dia!";
    for letra in mensagem.chars() {
        lista.colocar(letra);
        println!("{}",letra);
    }
    println!("Escrevi\nLendo:");
    let mut endereco: *mut CelulaSimples<char> = lista.cabeca;
    let conteudo: char;
    let mut pos_inserir: *mut CelulaSimples<char> = lista.cabeca;
    unsafe { conteudo = lista.cabeca.read().conteudo; }
    print!("{}",conteudo);
    for _i in 1..mensagem.len() {
        let ( conteudo, end) = lista.proxima_mut(endereco);
        endereco=end;
        print!("{}",conteudo);
        if conteudo=='m' { pos_inserir =endereco }
    }
    // endereco=lista.cabeca;
    lista.inserir_apos(pos_inserir, 's');
    println!("\nTerminei de inserir");
    lista.imprimir_lista();
    println!("Inserindo exclamação!");
    let (_c, pos_apos) = lista.proxima_mut(pos_inserir);
    lista.alterar(pos_apos, '!');
    lista.imprimir_lista();
    println!("Removendo exclamação!");
    lista.deletar_apos(pos_inserir);
    lista.imprimir_lista();
    println!("Removendo a primeira palavra");
    for _i in 1..=4 {
        lista.deletar_cabeca();
    }
    lista.imprimir_lista();
}

fn _teste_numerico() {
    println!("Iniciando teste com dados numéricos e iterador");
    let numeros: [i32; 5] = [10, 20, 30, 40, 50];
    let mut lista: ListaEncadeada<i32> = ListaEncadeada::novo();
    for n in numeros {
        lista.colocar(n) }
    for elem in lista.into_iter() {
        println!("{}",elem);
    }
    lista.imprimir_lista();
}
fn _teste_lista_dupla(){
    let mut lista: ListaDupla<char> = ListaDupla::novo();
    let mensagem: &str ="Palavra";
    for letra in mensagem.chars() {
        lista.colocar(letra);
        print!("{}",letra);
    }
    let mut endereco: *mut CelulaDupla<char> = lista.ponta;
    println!("\nInserindo exclamações entre as letras");
    for _i in 1..mensagem.len() {
        lista.inserir_antes(endereco, '!');
        let (_c, end) = lista.anterior(endereco);
        let (_c, end) = lista.anterior(end);
        endereco=end;
    }
    lista.imprimir_lista();
}
fn _main() {
    _teste_bom_dia();
    _teste_numerico();
    _teste_lista_dupla();
}