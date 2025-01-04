const esbuild = require('esbuild');

esbuild.build({
  entryPoints: ['./src/web/index.ts'], // エントリーポイント
  tsconfig: './tsconfig.json', // tsconfig.jsonのパス
  outfile: './dist/index.js',    // 出力先
  bundle: true,                   // 依存関係をバンドル
  minify: true,                   // 圧縮
  sourcemap: false,                // ソースマップ生成
  target: ['esnext'],             // トランスパイルのターゲット
  loader: { '.ts': 'ts' },        // TypeScriptを処理
  format: 'esm',  // 出力形式をESモジュールにする
}).then(() => {
    console.log('Build succeeded!');
}).catch(() => process.exit(1));