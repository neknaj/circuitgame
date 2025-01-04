const esbuild = require('esbuild');
const { exec } = require('child_process');
const fs = require('fs');
const util = require('util');
const path = require('path');
const https = require('https');

const execPromise = util.promisify(exec);

function makedir() {
    const destPath = './dist';
    const destDir = path.resolve(destPath);
    if (!fs.existsSync(destDir)) {
        fs.mkdirSync(destDir, { recursive: true });
    }
    console.log('Directory created:', destDir);
}

function buildTS() {
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
        console.log('TS Build succeeded!');
    }).catch(() => process.exit(1));
}

async function buildRust() {
    try {
        // wasm-packコマンドを実行
        const { stdout, stderr } = await execPromise('wasm-pack build --target web --features web');
        process.stdout.write(stdout);
        if (stderr) {
            process.stderr.write(stderr);
        }
        console.log('Wasm build complete!');
        return true;
    } catch (error) {
        // エラーオブジェクトから終了コードを取得
        process.stderr.write(error.message);
        console.error('Exit Code:', error.code); // 終了コードを表示
        return false;
    }
}


function copyFiles(files) {
    // 各ファイルをコピーするプロミスの配列を作成
    const copyPromises = files.map((file) => {
        const sourcePath = path.resolve(file[0]);
        const destinationPath = path.resolve(file[1]);
        return fs.promises.copyFile(sourcePath, destinationPath);
    });
    return Promise.all(copyPromises);
}

async function getCdomjs() {
    const url = 'https://raw.githubusercontent.com/neknaj/cDom/8d6fa798db021d3ce7e4e4a4c3073f5fb2e71237/cdom_module.ts';
    const savePath = './src/web/cdom.ts';
    if (fs.existsSync(savePath)) return; // 既に存在するならダウンロードしない
    await new Promise((resolve, reject) => {
        https.get(url, (res) => {
            if (res.statusCode === 200) {
                const file = fs.createWriteStream(savePath);
                res.pipe(file);

                file.on('finish', () => {
                    resolve('File downloaded and saved');
                });

                file.on('error', (err) => {
                    reject(`Error writing to file: ${err.message}`);
                });
            } else {
                reject(`Failed to download file. Status code: ${res.statusCode}`);
            }
        }).on('error', (err) => {
            reject(`Error: ${err.message}`);
        });
    });
}

async function main() {
    makedir();
    if (! await buildRust()) return;
    await copyFiles([
        [
            "./pkg/circuitgame_lib.js",
            "./src/web/circuitgame_lib.js"
        ],
        [
            "./pkg/circuitgame_lib.d.ts",
            "./src/web/circuitgame_lib.d.ts"
        ],
        [
            "./pkg/circuitgame_lib_bg.wasm",
            "./dist/circuitgame_lib_bg.wasm"
        ],
    ]);
    await getCdomjs();
    await buildTS();
    await copyFiles([
        [
            "./src/web/index.html",
            "./dist/index.html"
        ],
        [
            "./spec/sample.ncg",
            "./dist/sample.ncg"
        ],
    ]);
}

main()