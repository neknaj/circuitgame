mod parser;
mod modulecheck;
mod compile;
mod types;

pub fn compile(input: &str) -> types::IntermediateProducts {
    use modulecheck::*;
    use compile::*;
    let mut products = types::IntermediateProducts { warns: Vec::new(), errors: Vec::new(), ast: types::File { components: Vec::new() } , module_type_list: Vec::new(), module_dependency: Vec::new(), module_dependency_sorted: Vec::new() };
    // 0, パース
    products.ast = match parser::parser(input).map_err(|err| format!("[Parser error]\n{}", err)) {
        Ok(ast) => ast,
        Err(msg) => {products.errors.push(msg);return products;},
    };
    // 1, モジュール定義の一覧を作成
    products.module_type_list = collect_modules(&products.ast);
    // 2, モジュールの名前に重複がないかを確認
    match check_module_name_duplicates(&products.module_type_list) {
        Ok(()) => {},
        Err(msg) => {products.errors.extend(msg);return products;},
    };
    // 3, モジュールのゲートの使い方に問題がないかを確認
    match  check_module_gates(&products.ast,&products.module_type_list) {
        Ok(()) => {},
        Err(msg) => {products.errors.extend(msg);return products;},
    };
    // 4, モジュールの依存関係を作成
    products.module_dependency = module_dependency(&products.ast);
    // 5, トポロジカルソート (循環がないかを確認)
    products.module_dependency_sorted = match sort_dependency(&products.module_dependency,&products.module_type_list) {
        Ok(res) => {products.warns.extend(res.1);res.0},
        Err(res) => {products.errors.extend(res.0);products.warns.extend(res.1);return products;},
    };
    // 6, 依存関係の先端から順にモジュールを展開 (全てのmoduleがnorのみで構成される)
    match module_expansion(&products.ast, &products.module_dependency_sorted) {
        Ok(()) => {},
        Err(msg) => {products.errors.extend(msg);return products;},
    };
    // 7, 各モジュールの遅延を計算
    // 8, testを実行
    products
}