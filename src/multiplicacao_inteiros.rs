use std::arch::x86_64::_mm_i32gather_pd;
use std::cmp::{max, Ordering};
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub, Neg, Mul};

//Estrutura de dados para representar inteiros grandes
#[derive(Debug)]
struct Int {
    sinal: bool,    //true=positivo, false=negativo
    num: Vec<bool>  //true=1, false=0, começando pela unidade e caminhando na direção da dezena e da centena
}

impl Int {
    fn abs(self: &Self) -> Self {
        Int{
            sinal:true,
            num:self.num.clone(),
        }
    }
    fn compara_valor_abs(self: &Self, other: &Self) ->Ordering {
        let N = max(self.num.len(), other.num.len());
        let x = preenche_vetor(&self.num, N);
        let y = preenche_vetor(&other.num, N);
        for i in (0..N).rev() { //Varre o vetor de bits de tras pra frente (milhares para centenas para dezenas para unidades e compara self e other
            match (self.num.get(i), other.num.get(i)) {
                (None,None) | (None, Some(false)) | (Some(false), None) => {}
                (Some(true),None) | (Some(true),Some(false)) => {return Ordering::Greater}
                (None, Some(true)) | (Some(false),Some(true)) => {return Ordering::Less}
                (Some(true),Some(true)) | (Some(false), Some(false)) => {}
            }
        }
        return Ordering::Equal
    }
}
impl Neg for Int {
    type Output = Self;
    fn neg(self) ->Self::Output {
        Self{
            sinal: !self.sinal,
            num: self.num,
        }
    }

}


impl Add for Int {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        let N = max(self.num.len(),other.num.len());    //numero de casas decimais do maior numero
        //Copiamos os vetores dos dois numeros e preenchemos com zeros

        let mut soma = Vec::<bool>::new();
        let mut excedente: i32 = 0; //valor excedente em cada soma de bits para fazer o "vai 1", no caso de a soma exceder a dezena binária
        if self.sinal==other.sinal {    //Soma de numeros naturais
            let x: Vec<bool> = preenche_vetor(&self.num, N+1);
            let y: Vec<bool> = preenche_vetor(&other.num, N+1);
            for i in 0..(N+1) {

                match (x[i], y[i]) {
                    (true, true) => { //1+1=10
                        if excedente ==0 {
                            excedente+= 1;
                            soma.push(false);
                        }
                        else {
                            // nesse caso a soma é 11: o excedente continua igual a 1, mas o bit atual vira 1
                            soma.push(true);
                        }
                    },
                    (true, false) | (false,true) => {
                        if excedente==0 {
                            soma.push(true)
                        }
                        else {
                            // Nesse caso a soma é 10: o bit atual é 0 e o excedente continua igual a 1
                            soma.push(false)
                        }
                        // else {
                        //     excedente-=1;
                        //     prod.push(true)
                        // }
                    },
                    (false,false) => {
                        if excedente==0 {
                            soma.push(false)
                        }
                        else {
                            excedente-=1;
                            soma.push(true)
                        }
                    },
                }
            }

            Int{
                sinal: self.sinal,
                num: soma,
            }
        }
        else{   //Subtração |x|-|y|
            let (x,y, sinal) = match self.compara_valor_abs(&other) {
                Ordering::Less => {
                    let x: Vec<bool> = preenche_vetor(&other.num, N+1);
                    let y: Vec<bool> = preenche_vetor(&self.num, N+1);
                    let sinal = other.sinal;
                    (x,y, sinal)
                }
                Ordering::Equal => {
                    return Int{sinal: true, num:Vec::<bool>::new()}
                }
                Ordering::Greater => {
                    let x: Vec<bool> = preenche_vetor(&self.num, N+1);
                    let y: Vec<bool> = preenche_vetor(&other.num, N+1);
                    let sinal = other.sinal;
                    (x,y, sinal)
                }
            };
            for i in 0..(N+1) {
                match (x[i],y[i]) {
                    (true,true) | (false,false) =>{
                        if excedente==0 {soma.push(false)}
                        else {  //Nesse caso, o excedente é -1, o bit atual fica 1 e continuamo "devendo" 1
                            soma.push(true)
                        }
                    }
                    (true, false) => {
                        if excedente==0 {soma.push(true)}
                        else {
                            excedente+=1;
                            soma.push(false)
                        }
                    }
                    (false,true) => {
                        if excedente==0 {   //0-1->1, e o excedente vira -1
                            excedente-=1;
                            soma.push(true);
                        }
                        else{   //Se o excedente é -1, então o bit vira 0 e o excedente permanece
                            soma.push(false)
                        }
                    }
                }
            }
            Int{
                sinal:sinal,
                num:soma
            }
        }

    }
}
impl Sub for Int {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        self + (-other)
    }
}
impl Mul for Int {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        // //Multiplicacao usando o algoritmo de Karatsuba de dividir para conquistar
        // let sinal = (self.sinal==other.sinal);  //O produto é positivo sse os fatores tiverem o mesmo sinal
        // let N = max(self.num.len(), other.num.len());
        // if N==1 {
        //     return self.num[0] & other.num[0]
        // }
        // let (k,pot_2): (u32, usize) = maior_pot_2(N as u32) as (u32, usize);
        // if N!=pot_2 {
        //     let x = Int{
        //         sinal:self.sinal,
        //         num:preenche_vetor(&self.num, 2*pot_2),
        //     } ;
        //     let y = Int{
        //         sinal:other.sinal,
        //         num:preenche_vetor(&other.num, 2*pot_2),
        //     } ;
        //     return x*y
        // }
        // //TODO()
        self

        // let sinal = (self.sinal==other.sinal);  //O produto é positivo sse os fatores tiverem o mesmo sinal
        // let N = max(self.num.len(), other.num.len());
        // if N==1 {
        //         return Int {
        //             sinal:sinal,
        //             num:vec![self.num[0] & other.num[0]] }
        //     }
        // let mut prod = Vec::<bool>::new();
        // let x = preenche_vetor(&self.num, N);
        // let y = preenche_vetor(&other.num, N);
        // for i in 0..N {
        //     let produto = x[i]*y[i];
        // }

    }
}

impl From<i32> for Int {
    fn from(value: i32) -> Self {
        let sinal: bool = value>=0;
        let val = value.abs();
        let mut num = Vec::<bool>::new();
        let mut n=0;
        while 2_i32.pow(n)<=val {
            let resto = val % 2_i32.pow(n+1);
            let bit:bool = resto>=2_i32.pow(n);
            num.push(bit);
            n+=1;
        }
        Int{
            sinal,
            num
        }
    }
}

impl Into<i32> for Int {
    fn into(self) -> i32 {
        let mut val: i32 = 0;
        for (i,bit) in self.num.into_iter().enumerate() {
            if bit {
                val+=2_i32.pow(i as u32);
            }
        }
        if self.sinal { val}
        else { -val}

    }
}
// impl Display for Int {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         Ok(
//             if ! self.sinal {
//                 write!(f,"-");
//             };
//             for
//         )
//     }
// }

fn preenche_vetor(v: &Vec<bool>, N: usize) -> Vec<bool> {
    //Cria uma copia do vetor v preenchido ate ter N casas, completando com zeros se necessario
    let mut u = Vec::<bool>::new();
    for i in 0..N {
        match v.get(i) {
            None => {u.push(false)},
            Some(valor) => {u.push(*valor)}
        }
    }
    u
}
fn maior_pot_2(n:u32) -> (u32,u32) {
    //Encontra a maior potencia de 2 tal que 2^k<=n
    //Retorna (k, 2^k)
    let mut k:u32 = 0;
    assert!(n>0);
    if n==1 {return (0, 1)}
    while 2_u32.pow(k+1)<=n {
        k+=1;
    }
    (k, 2_u32.pow(k))
}

#[test]
fn teste_preencher() {
    let u = vec![true,false,true];
    let v = preenche_vetor(&u, 5);
    println!("{:?}", v);
    assert_eq!(v, vec![true,false,true,false,false])
}

#[test]
fn teste_conversao_soma() {
    let dez = Int::from(-10);
    println!("{:?}",dez);
    let dez_reconvertido: i32 = dez.into();
    println!("{}",dez_reconvertido);
    assert_eq!(dez_reconvertido,-10);
    let x = Int::from(53);
    let y = Int::from(47);
    println!("x = {:?}",x);
    println!("y = {:?}",y);
    let soma = x+y;
    println!("x+y = {:?}",soma);
    let soma_i32: i32 = soma.into();
    println!("soma = {}",soma_i32);
    assert_eq!(soma_i32, 100);
    let z = Int::from(7);
    let w = Int::from(-11);
    println!("7 = z = {:?}", z);
    println!("-11 = w = {:?}", w);
    let soma = z + w;
    println!("z+w = {:?}",soma);
    let soma_i32: i32 = soma.into();
    println!("7-11 = {}",soma_i32);
    assert_eq!(soma_i32,-4);
}

fn main() {
    println!("Multiplicação")
}