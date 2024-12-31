mod testcheck;
mod test;
mod types;

use crate::vm;

pub fn test_intermediate_products(products: crate::compiler::types::IntermediateProducts) -> types::IntermediateProducts {
    let mut test_products = types::IntermediateProducts { warns: Vec::new(), errors: Vec::new(), test_list: Vec::new() };
    use testcheck::*;
    // 1, テスト定義の一覧を作成
        // test_products.test_list = collect_tests(&products.ast);
    // 2, テストの名前に重複がないかを確認 -> err
    // 2, テストの名前に不足がないかを確認 -> warn
    // 3, テストのゲートのIOの数に問題がないかを確認
        // match  check_test_gates(&products.ast,&products.module_type_list) {
        //     Ok(()) => {},
        //     Err(msg) => {products.errors.extend(msg);return test_products;},
        // };
    test_products
}