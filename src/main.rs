#[cfg(not(feature = "web"))]
mod compiler;
#[cfg(not(feature = "web"))]
mod test;
#[cfg(not(feature = "web"))]
mod vm;
#[cfg(not(feature = "web"))]
fn main() -> Result<(),()> {
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
    let result = compiler::intermediate_products(input);
    // println!("[result] {:#?}",result);
    println!("[warns] {:#?}",&result.warns);
    println!("[errors] {:#?}",&result.errors);
    println!("[sortedDependency] {:#?}",&result.module_dependency_sorted);
    if result.errors.len()>0 {
        return Err(());
    }
    let module = match result.module_dependency_sorted.get(0) {
        Some(v) => v.clone(),
        None => {return Err(());}
    };
    let binary = match compiler::serialize(result.clone(), module.as_str()) {
        Ok(v)=>v,
        Err(v)=>{
            println!("[error] {:#?}",v);
            return Err(());
        }
    };
    println!("{:?}",binary);
    let test_result = test::test_intermediate_products(result);
    println!("[test warns] {:#?}",&test_result.warns);
    println!("[test errors] {:#?}",&test_result.errors);
    let _ = vm::init(binary);
    match vm::next() {Ok(())=>{},Err(v)=>{println!("[error] {:#?}",v);}};
    match vm::set_input(0, true) {Ok(())=>{},Err(v)=>{println!("[error] {:#?}",v);}};
    match vm::set_input(1, true) {Ok(())=>{},Err(v)=>{println!("[error] {:#?}",v);}};
    match vm::set_input(2, true) {Ok(())=>{},Err(v)=>{println!("[error] {:#?}",v);}};
    match vm::next() {Ok(())=>{},Err(v)=>{println!("[error] {:#?}",v);}};
    match vm::next() {Ok(())=>{},Err(v)=>{println!("[error] {:#?}",v);}};
    match vm::next() {Ok(())=>{},Err(v)=>{println!("[error] {:#?}",v);}};
    match vm::next() {Ok(())=>{},Err(v)=>{println!("[error] {:#?}",v);}};
    match vm::get_output() {Ok(v)=>{println!("{:#?}",v);},Err(v)=>{println!("[error] {:#?}",v);}};
    return Ok(());
}


#[cfg(feature = "web")]
fn main() {
}