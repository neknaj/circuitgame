mod compiler;

fn main() {
    println!("Hello World");
    let input = "USEnor:2>1;DEFnot(x)>(a){a:nor<x,x;}DEFor(x,y)>(b){a:nor<x,y;b:not<a;}DEFand(x,y)>(c){a:not<x;b:not<y;c:nor<a,b;}DEFxor(x,y)>(e){a:not<x;b:not<y;c:nor<a,b;d:nor<x,y;e:nor<c,d;}DEFhAddr(x,y)>(c,s){c:and<x,y;s:xor<x,y;}DEFfAdr(x,y,z)>(c,s2){c1,s1:hAddr<x,y;c2,s2:hAddr<s1,z;c:or<c1,c2;}TESTnot:1>1{t>f;f>t;}TESTor:2>1{t,t>t;t,f>t;f,t>t;f,f>f;}TESTand:2>1{t,t>t;t,f>f;f,t>f;f,f>f;}";
    match compiler::compile(input) {
        Ok(()) => {},
        Err(error) => { println!("{}",error); }
    }
}