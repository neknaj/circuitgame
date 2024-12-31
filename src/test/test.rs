use super::types::*;
use crate::vm;

pub fn test_gates(
    product: &crate::compiler::types::IntermediateProducts,
    module_type_list: &Vec<crate::compiler::types::ModuleType>,
)-> ResultwithWarn<std::collections::HashMap<String,Vec<TestPattern>>> {
    let mut errors = Vec::new();
    let warns = Vec::new();
    let mut result_map = std::collections::HashMap::<String,Vec<TestPattern>>::new();
    // let mut errors = Vec::new();
    for component in &product.ast.components {
        match component {
            crate::compiler::types::Component::Test(test)=>{ // Testのみ処理
                // println!("< {:#?} >",&test.name);
                // moduleの型を取得
                let module_type = match module_type_list.iter().find(|m| m.name==test.name).map(|m| &m.mtype) {
                    Some(mtype) => mtype.clone(),
                    None => { errors.push(format!("Undefined module used: {}",test.name));break; },
                };
                // patternがtypeに合致するかを確認
                for pattern in &test.patterns {
                    // println!("{:#?}",pattern);
                    if pattern.inputs.len()!=module_type.input_count||pattern.outputs.len()!=module_type.output_count {
                        errors.push(format!("Used module with unmatched type: {} expected {}->{} but got {}->{}",&test.name,module_type.input_count,module_type.output_count,pattern.inputs.len(),pattern.outputs.len()));
                    }
                }
                // vmに入れて出力を確認する
                let binary = match crate::compiler::serialize(product.clone(), &test.name.as_str()) {
                    Ok(v)=>v,
                    Err(v)=>{ errors.push(v);break; }
                };
                let _ = vm::init(binary);
                let mut test_result = Vec::new();
                // それぞれのpatternを試す
                for pattern in &test.patterns {
                    // 入力を設定する
                    let mut input_index = 0;
                    for input in &pattern.inputs {
                        let _ = vm::set_input(input_index, *input);
                        input_index+=1;
                    }
                    // vmを数ステップ進める <- todo: 指定できるようにする
                    for _ in 0..1000 {
                        let _ = vm::next();
                    }
                    // 出力を取得する
                    let output = match vm::get_output() {
                        Ok(v) => v,
                        Err(v)=>{ errors.push(v);break; }
                    };
                    // 出力の一致を確認する
                    let mut test_failed = false;
                    let mut out_index = 0;
                    for out in &output {
                        let expect = match pattern.outputs.get(out_index) {
                            Some(v) => *v,
                            None => { errors.push(format!("Index out of bounds"));break; }
                        };
                        if *out!=expect {
                            test_failed = true;
                        }
                        out_index+=1;
                    }
                    //
                    // println!("test accepted: {}",!test_failed);
                    let test_pattern = TestPattern {
                        input : pattern.inputs.clone(),
                        expect: pattern.outputs.clone(),
                        output: output.clone(),
                        accept: !test_failed,
                    };
                    test_result.push(test_pattern);
                }
                result_map.insert(test.name.clone(), test_result);
            },
            _ => {} // Testでなければ何もしない
        }
    }
    if errors.len()>0 {
        return Err((errors,warns));
    }
    Ok((result_map,warns))
}