use tokio_tungstenite::tungstenite::http::header;
use warp::filters::method::head;

use crate::vm::deserializer::deserialize_from_vec;
use crate::vm::types::*;

pub fn transpile(module: Module,header: bool) -> Result<String,String> {
    if !module.func {
        return Err(format!("C transpiler does not support nom-func module: {}",module.name));
    }
    // ヘッダーを作る
    let out_struct = format!("typedef struct {{\n    bool outputs[{}OutputsLen];\n}} {}Result;",module.name,module.name);
    let out_func_h = format!("{}Result {}({});",module.name,module.name,vec!["int";module.inputs as usize].join(","));
    let out_header = format!("#ifndef TRANSPILE_{}_HEADER\n#define TRANSPILE_{}_HEADER\n\n{}\n\n{}\n\n{}\n\n{}\n\n#endif",module.name,module.name,"#include <stdbool.h>",format!("#define {}OutputsLen {}",module.name,module.outputs.len()),out_struct,out_func_h);
    if header { return Ok(out_header); }
    // 本体の関数を作る
    let out_func_head = format!("{}Result {}({})",module.name,module.name,(0..module.inputs as usize).map(|i| format!("int b{}",i+module.gates.len())).collect::<Vec<String>>().join(", "));
    let out_func_gates = module.gates.iter().enumerate().map(|(index,value)| format!("    int b{} = !( b{} | b{} );",index,value.0,value.1)).collect::<Vec<String>>().join("\n");
    let out_func_return = format!("    {}Result result = {{{{ {} }}}};\n    return result;",module.name,module.outputs.iter().map(|value| format!("b{}",value)).collect::<Vec<String>>().join(", "));
    let out_func = format!("{} {{\n{}\n{}\n}}",out_func_head,out_func_gates,out_func_return);
    let out_code = format!("{}\n\n{}",out_header,out_func);
    Ok(out_code)
}