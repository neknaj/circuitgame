import init, { Parse } from './circuitgame_lib.js';

async function run() {
    await init();
    const input = `
// Example usage with comments
using nor:2->1;

// This is a NOT gate module
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
    `;
    const input_without_space = `USEnor:2>1;DEFnot(x)>(a){a:nor<x,x;}DEFor(x,y)>(b){a:nor<x,y;b:not<a;}DEFand(x,y)>(c){a:not<x;b:not<y;c:nor<a,b;}DEFxor(x,y)>(e){a:not<x;b:not<y;c:nor<a,b;d:nor<x,y;e:nor<c,d;}DEFhAddr(x,y)>(c,s){c:and<x,y;s:xor<x,y;}DEFfAdr(x,y,z)>(c,s2){c1,s1:hAddr<x,y;c2,s2:hAddr<s1,z;c:or<c1,c2;}TESTnot:1>1{t>f;f>t;}TESTor:2>1{t,t>t;t,f>t;f,t>t;f,f>f;}TESTand:2>1{t,t>t;t,f>f;f,t>f;f,f>f;}`;
    try {
        console.log("< Input >")
        const result = Parse(input);
        console.log(result);
    } catch (e) {
        console.error('Parse error:', e);
    }
    try {
        console.log("< Input without Space >")
        const result = Parse(input_without_space);
        console.log(result);
    } catch (e) {
        console.error('Parse error:', e);
    }
}

run();