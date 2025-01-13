#![cfg(feature = "web")]

mod compiler;
mod transpiler;
mod test;
mod vm;
mod resourcemanager;

use wasm_bindgen::prelude::*;


// resource manager
use std::sync::Mutex;
use crate::resourcemanager::*;
use crate::vm::types::Module;

lazy_static::lazy_static! {
    static ref VM_resource: Mutex<ResourceManager<Module>> = Mutex::new( ResourceManager::new() );
}

#[wasm_bindgen(js_name=Module)]
pub fn export_Module(data: Vec<u32>) -> Result<u32,String> {
    let mut vmres = match VM_resource.lock() {
        Ok(v)=>v,
        Err(_)=> return Err(format!("Mutex error"))
    };
    match Module::new(data) {
        Ok(v)=>{
            Ok( vmres.add_resource(v) )
        }
        Err(v) => Err(v)
    }
}

// Compile, Test

#[wasm_bindgen(js_name=CompilerIntermediateProducts)]
pub fn export_compiler_intermediate_products(input: &str) -> String {
    let result = compiler::intermediate_products(input);
    match serde_json::to_string_pretty(&result) {
        Ok(str) => str,
        Err(_) => return format!("serializing error"),
    }
}

#[wasm_bindgen(js_name=Test)]
pub fn export_test(input: &str) -> String {
    let result = compiler::intermediate_products(input);
    if result.errors.len()>0 {
        return format!("compiling error");
    }
    let test_result = test::test(result);
    match serde_json::to_string_pretty(&test_result) {
        Ok(str) => str,
        Err(_) => return format!("serializing error"),
    }
}

#[wasm_bindgen(js_name=Compile)]
pub fn export_compile(input: &str,module: &str) -> Vec<u32> {
    match compiler::compile(input,module) {
        Ok(v) => v,
        Err(_) => Vec::new(),
    }
}




#[wasm_bindgen(js_name=TranspileTS)]
pub fn export_transpile(input: &str,output_modules_pattern: &str) -> String {
    let result = compiler::intermediate_products(&input);
    if result.errors.len()>0 {
        return format!("// Error:\n{}",result.errors.join("\n"));
    }
    let mut output_modules = Vec::new();
    let regex_pattern = regex::Regex::new(&format!("^({})$",output_modules_pattern)).unwrap();
    for test_str in result.defined_func_module_list.clone() {
        if regex_pattern.is_match(&test_str) {
            output_modules.push(test_str.clone());
        }
    }
    for test_str in result.defined_non_func_module_list.clone() {
        if regex_pattern.is_match(&test_str) {
            output_modules.push(test_str.clone());
        }
    }
    let mut modules = Vec::new();
    for module_name in &output_modules {
        let binary = match compiler::serialize(result.clone(), module_name.as_str()) {
            Ok(v) => v,
            Err(v) => { return format!("// Error: {}",v); }
        };
        modules.push(Module::new(binary).unwrap());
    }
    match transpiler::ts_transpiler::transpile(modules,false) {
        Ok(v) => v,
        Err(v) => format!("// Error: {}",v)
    }
}

#[wasm_bindgen(js_name=TranspileTSresId)]
pub fn export_ts_transpile_res_id(modules_res_id: Vec<u32>) -> String {
    let mut modules = Vec::new();
    for res_id in modules_res_id {
        let mut vmres = match VM_resource.lock() {
            Ok(v)=>v,
            Err(_)=> {continue;}
        };
        match vmres.get_resource(res_id) {
            Some(module) => {
                modules.push(module.clone());
            },
            None => {},
        }
    }
    match transpiler::ts_transpiler::transpile(modules,false) {
        Ok(v) => v,
        Err(v) => format!("// Error: {}",v)
    }
}



// VM



#[wasm_bindgen(js_name=VMreset)]
pub fn export_VMreset(resource_id: u32) -> Result<(),String> {
    let mut vmres = match VM_resource.lock() {
        Ok(v)=>v,
        Err(_)=> return Err(format!("Mutex error"))
    };
    match vmres.get_resource(resource_id) {
        Some(module)=> {
            Ok( module.reset() )
        },
        None=> return  Err(format!("Resource not found: {}",resource_id))
    }
}

#[wasm_bindgen(js_name=VMset)]
pub fn export_VMset(resource_id: u32,index: u32,value: bool) -> Result<(),String> {
    let mut vmres = match VM_resource.lock() {
        Ok(v)=>v,
        Err(_)=> return Err(format!("Mutex error"))
    };
    match vmres.get_resource(resource_id) {
        Some(module)=> {
            module.set(index,value)
        },
        None=> return  Err(format!("Resource not found: {}",resource_id))
    }
}
#[wasm_bindgen(js_name=VMgetOutput)]
pub fn export_VMgetOutput(resource_id: u32) -> Vec<u32> {
    let mut vmres = match VM_resource.lock() {
        Ok(v)=>v,
        Err(_)=> return Vec::new()
    };
    match vmres.get_resource(resource_id) {
        Some(module)=> {
            match module.get_output() {
                Ok(v)=>v.into_iter().map(|b| if b { 1 } else { 0 }).collect(),
                Err(_)=>Vec::new()
            }
        },
        None=> Vec::new()
    }
}
#[wasm_bindgen(js_name=VMgetGates)]
pub fn export_VMgetGates(resource_id: u32) -> Vec<u32> {
    let mut vmres = match VM_resource.lock() {
        Ok(v)=>v,
        Err(_)=> return Vec::new()
    };
    match vmres.get_resource(resource_id) {
        Some(module)=> {
            module.get_gates().into_iter().map(|b| if b { 1 } else { 0 }).collect()
        },
        None=> return Vec::new()
    }
}
#[wasm_bindgen(js_name=VMgetTick)]
pub fn export_VMgetTick(resource_id: u32) -> u128 {
    let mut vmres = match VM_resource.lock() {
        Ok(v)=>v,
        Err(_)=> return 0
    };
    match vmres.get_resource(resource_id) {
        Some(module)=> {
            module.get_tick()
        },
        None=> return 0
    }
}
#[wasm_bindgen(js_name=VMgetTickAsStr)]
pub fn export_VMgetTick_as_str(resource_id: u32) -> String {
    let mut vmres = match VM_resource.lock() {
        Ok(v)=>v,
        Err(_)=> return format!("Mutex error")
    };
    match vmres.get_resource(resource_id) {
        Some(module)=> {
            module.get_tick().to_string()
        },
        None=> return  format!("Resource not found: {}",resource_id)
    }
}
#[wasm_bindgen(js_name=VMnext)]
pub fn export_VMnext(resource_id: u32,n: u32) -> Result<u128,String> {
    let mut vmres = match VM_resource.lock() {
        Ok(v)=>v,
        Err(_)=> return Err(format!("Mutex error"))
    };
    match vmres.get_resource(resource_id) {
        Some(module)=> {
            module.next(n)
        },
        None=> return  Err(format!("Resource not found: {}",resource_id))
    }
}
#[wasm_bindgen(js_name=VMnextAsStr)]
pub fn export_VMnext_as_str(resource_id: u32,n: u32) -> Result<String,String> {
    let mut vmres = match VM_resource.lock() {
        Ok(v)=>v,
        Err(_)=> return Err(format!("Mutex error"))
    };
    match vmres.get_resource(resource_id) {
        Some(module)=> {
            match module.next(n) {
                Ok(v)=> Ok(v.to_string()),
                Err(v)=> Err(v),
            }
        },
        None=> return  Err(format!("Resource not found: {}",resource_id))
    }
}