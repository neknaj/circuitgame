use compile::serialize_to_vec;
use types::IntermediateProducts;

mod parser;
mod modulecheck;
mod compile;
mod types;

#[cfg(feature = "web")]
pub fn compile(input: &str,module: &str) -> Result<Vec<u32>,String> {
    serialize(intermediate_products(input), module)
}

pub fn serialize(products: IntermediateProducts,module: &str) -> Result<Vec<u32>,String> {
    if products.errors.len()>0 {
        return Err(products.errors.join("\n"));
    }
    let expanded_module = match products.expanded_modules.get(module) {
        Some(v) => v.clone(),
        None => {return Err(format!("An undefined module was specified: {}",module));}
    };
    let serialized = serialize_to_vec(expanded_module);
    Ok(serialized)
}

pub fn intermediate_products(input: &str) -> types::IntermediateProducts {
    use modulecheck::*;
    use compile::*;
    let mut products = types::IntermediateProducts { warns: Vec::new(), errors: Vec::new(), ast: types::File { components: Vec::new() } , module_type_list: Vec::new(), module_dependency: Vec::new(), module_dependency_sorted: Vec::new(), expanded_modules: std::collections::HashMap::new() };
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
    products.expanded_modules = match module_expansion(&products.ast, &products.module_dependency_sorted) {
        Ok(v) => {v},
        Err(msg) => {products.errors.extend(msg);return products;},
    };
    // 7, 各モジュールの遅延を計算
    // 8, testを実行
    products
}