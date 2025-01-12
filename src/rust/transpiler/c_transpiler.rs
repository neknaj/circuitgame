use tokio_tungstenite::tungstenite::http::header;
use warp::filters::method::head;

use crate::vm::deserializer::deserialize_from_vec;
use crate::vm::types::*;

pub fn transpile(module: Module,header: bool) -> Result<String,String> {
    if !module.func {
        return Err(format!("C transpiler does not support nom-func module: {}",module.name));
    }
    let out_struct = format!("typedef struct {{\n    bool arr[{}]\n}} {}Result;",module.outputs.len(),module.name);
    let out_func = format!("{}Result {}({});",module.name,module.name,vec!["int";module.inputs as usize].join(","));
    let out_header = format!("#ifndef TRANSPILE_HEADER\n#define TRANSPILE_HEADER\n\n{}\n\n{}\n\n{}\n\n#endif","#include <stdbool.h>",out_struct,out_func);
    if header { return Ok(out_header); }
    Ok(out_header)
}