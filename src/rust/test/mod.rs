mod testcheck;
mod test;
mod types;

pub fn test(products: crate::compiler::types::IntermediateProducts) -> types::TestProducts {
    let mut test_products = types::TestProducts { warns: Vec::new(), errors: Vec::new(), test_list: Vec::new(), test_result: std::collections::HashMap::new() };
    use testcheck::*;
    // 1, テスト定義の一覧を作成
    test_products.test_list = collect_tests(&products.ast);
    // 2, テストの名前に不足がないかを確認 -> warn
    test_products.warns.extend(check_test_missing(&test_products.test_list, &products.module_dependency_sorted));
    // 3, テストの名前に重複がないかを確認 -> err
    match check_test_name_duplicates(&test_products.test_list) {
        Ok(()) => {},
        Err(msg) => {test_products.errors.extend(msg);return test_products;},
    };
    // 4, テストを実行
    test_products.test_result = match test::test_gates(&products,&products.module_type_list) {
        Ok(res) => {test_products.warns.extend(res.1);res.0},
        Err(res) => {test_products.errors.extend(res.0);test_products.warns.extend(res.1);return test_products;},
    };
    test_products
}