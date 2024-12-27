# Rust+TypeScript Webアプリ テンプレート
![image](https://img.shields.io/badge/-TypeScript-103040.svg?logo=typescript&style=popout)
![image](https://img.shields.io/badge/-Rust-403540.svg?logo=rust&style=popout)


このプロジェクトは、Rust と TypeScript を使用して WebAssembly (Wasm) を利用した Web アプリケーションを構築するためのテンプレートです。  

## 始め方

このテンプレートを使用するための手順は以下の通りです。  

### 必要な環境

- Node.js (>=14.0.0)  
- Rust (>=1.60.0)  
    - `wasm-pack` (Rust コードを WebAssembly にビルドするためのツール)  

(Linuxの場合は、このリポジトリで$`npm run install-rust`を実行すればRustとwasm-packがインストールできます)  

### 開発

1. このリポジトリをクローンします  

2. buildします  
    $`npm run build`を実行すれば、`/dist`に必要なファイルが作成されます  

3. ローカルのサーバーで動かします  
    `/dist`をルートにしてサーバーを起動してください  
    $`npm run server`を実行すれば、http-serverを使ってローカルサーバーを起動できます  
    表示されるURLをWebブラウザで開いて下さい  

4. ファイルを編集します  
    `/src`内のファイルを編集して、Webアプリの機能を作成してください  
    編集したら`2`~`3`を再度行ってください  

5. アプリを公開します  
    - Github Pages  
        `.github\workflows\deploy.yml`に、Github Actionsを使って公開するための設定があります  
        Githubにcommitすると、`gh-pages`ブランチが作られ、`/dist`の中身がその中に入ります  
        Githubリポジトリの`Settings > Pages`からGithub Pagesの設定を行ってください  
    - その他  
        GithubPagesと同様、`/dist`の中身をWebサーバーに配置すれば行えます  

