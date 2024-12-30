#[cfg(not(feature = "web"))]
mod compiler;
#[cfg(not(feature = "web"))]
fn main() {
    println!("Hello World");
    let input = "
// Example usage with comments
using nor:2->1;

module not (x)->(a) {
    a: nor <- x x;
}

module or (x y)->(b) {
    a: nor <- x y;
    b: not <- a  ;
}

module and (x y)->(c) {
    a: not <- x  ;
    b: not <- y  ;
    c: nor <- a b;
}

module xor (x y)->(e) {
    a: not <- x  ;
    b: not <- y  ;
    c: nor <- a b;
    d: nor <- x y;
    e: nor <- c d;
}

module hAddr (x y)->(c s) {
    c: and <- x y;
    s: xor <- x y;
}

module fAdr (x y z)->(c s2) {
    c1 s1: hAddr <- x y  ;
    c2 s2: hAddr <- s1 z ;
    c    : or    <- c1 c2;
}

test not:1->1 {
    t -> f;
    f -> t;
}

test or:2->1 {
    t t -> t;
    t f -> t;
    f t -> t;
    f f -> f;
}

test and:2->1 {
    t t -> t;
    t f -> f;
    f t -> f;
    f f -> f;
}
";
    let result = compiler::compile(input);
    // println!("[result] {:#?}",result);
    println!("[warns] {:#?}",result.warns);
    println!("[errors] {:#?}",result.errors);
    println!("[sortedDependency] {:#?}",result.module_dependency_sorted);
}


#[cfg(feature = "web")]
fn main() {
}