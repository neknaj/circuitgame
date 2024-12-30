use modulecheck::sort_dependency;

pub mod parser;
pub mod modulecheck;
mod types;


pub fn compile(input: &str) -> Result<(),String> {
    let mut warns: Vec<String> = Vec::new();
    // 0, パース
    let ast = parser::parser(input).map_err(|err| format!("[Parser error]\n{}", err))?;
    // println!("{:#?}",ast);
    // 1, モジュール定義の一覧を作成
    let module_type_list = modulecheck::collect_modules(&ast);
    // 2, モジュールの名前に重複がないかを確認
    modulecheck::check_module_name_duplicates(&module_type_list)?;
    // 3, モジュールのゲートの使い方に問題がないかを確認
    modulecheck::check_module_gates(&ast,&module_type_list)?;
    // 4, モジュールの依存関係を作成
    let module_dependency = modulecheck::module_dependency(&ast);
    // 5, トポロジカルソート (循環がないかを確認)
    let sort_res = sort_dependency(&module_dependency,&module_type_list);
    warns.push(sort_res.warn);
    let module_dependency_sorted = sort_res.res?;
    // 6, 依存関係の先端から順にモジュールを展開 (全てのmoduleがnorのみで構成される)
    // 7, 各モジュールの遅延を計算
    // 8, testを実行
    Ok(())
}