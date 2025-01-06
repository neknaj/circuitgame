pub fn collect_tests(ast: &crate::compiler::types::File) -> Vec<String> {
    use crate::compiler::types::*;
    let mut tests = Vec::new();
    for component in &ast.components {
        match component {
            Component::Test(test)=>{
                tests.push(test.name.clone());
            },
            _ => {} // テストでなければ何もしない
        }
    }
    tests
}

pub fn check_test_name_duplicates(modules: &Vec<String>) -> Result<(),Vec<String>> {
    let mut module_names = std::collections::HashSet::new();
    let mut errors = Vec::new();
    for module in modules {
        if !module_names.insert(module) {
            errors.push(format!("Multiple tests are provided for one module: {}",module));
        }
    }
    if errors.len()==0 { Ok(()) }
    else { Err(errors) }
}

pub fn check_test_missing(provided_tests: &Vec<String>,defined_modules: &Vec<String>) -> Vec<String> {
    let mut warns = Vec::new();
    for module in defined_modules {
        if !provided_tests.contains(module)&&module!="nor" {
            warns.push(format!("No test provided for module: {}",module));
        }
    }
    warns
}