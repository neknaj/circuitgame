import init, { greet } from './tsrust.js';

async function run() {
  await init(); // Wasmモジュールの初期化
  console.log(greet('World')); // Rustで書いた関数を呼び出し
}

run();


console.log("hello world")