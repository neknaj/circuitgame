pub mod parser;
mod types;


pub fn compile(input: &str) -> Result<(),String> {
    // 0, パース
    let ast = parser::parser(input).map_err(|err| format!("[Parser error]\n{}", err))?;
    // 1, モジュール定義の一覧を作成
    // 残りの処理
    return Ok(());
}