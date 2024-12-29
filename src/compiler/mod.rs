pub mod parser;
pub mod modulecheck;
mod types;


pub fn compile(input: &str) -> Result<(),String> {
    // 0, パース
    let ast = parser::parser(input).map_err(|err| format!("[Parser error]\n{}", err))?;
    // println!("{:#?}",ast);
    // 1, モジュール定義の一覧を作成
    let module_type_list = modulecheck::collect_modules(&ast);
    // 2, モジュールの名前に重複がないかを確認
    modulecheck::check_module_name_duplicates(&module_type_list)?;
    // 3, モジュールのゲートの使い方に問題がないかを確認
    modulecheck::check_module_gates(&ast,&module_type_list)?;
    // 4, モジュールの依存関係を作成し、循環がないかを確認
    // 5, 依存関係の先端から順にモジュールを展開 (全てのmoduleがnorのみで構成される)
    // 6, 各モジュールの遅延を計算
    // 7, testを実行
    Ok(())
}