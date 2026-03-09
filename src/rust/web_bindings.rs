use crate::resourcemanager::ResourceManager;
use crate::vm::types::Module;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

lazy_static::lazy_static! {
    static ref VM_RESOURCE: Mutex<ResourceManager<Module>> = Mutex::new(ResourceManager::new());
}

#[wasm_bindgen(js_name=Module)]
pub fn export_module(data: Vec<u32>) -> Result<u32, String> {
    let mut vmres = match VM_RESOURCE.lock() {
        Ok(v) => v,
        Err(_) => return Err("Mutex error".to_string()),
    };
    match Module::new(data) {
        Ok(v) => Ok(vmres.add_resource(v)),
        Err(v) => Err(v),
    }
}

#[wasm_bindgen(js_name=CompilerIntermediateProducts)]
pub fn export_compiler_intermediate_products(input: &str) -> String {
    let result = crate::compiler::intermediate_products(input);
    match serde_json::to_string_pretty(&result) {
        Ok(text) => text,
        Err(err) => format!("serializing error {err}"),
    }
}

#[wasm_bindgen(js_name=Test)]
pub fn export_test(input: &str) -> String {
    let result = crate::compiler::intermediate_products(input);
    if !result.errors.is_empty() {
        return "compiling error".to_string();
    }
    let test_result = crate::test::test(result);
    match serde_json::to_string_pretty(&test_result) {
        Ok(text) => text,
        Err(err) => format!("serializing error {err}"),
    }
}

#[wasm_bindgen(js_name=Compile)]
pub fn export_compile(input: &str, module: &str) -> Vec<u32> {
    match crate::compiler::compile(input, module) {
        Ok(v) => v,
        Err(_) => Vec::new(),
    }
}

#[wasm_bindgen(js_name=TranspileTS)]
pub fn export_transpile(input: &str, output_modules_pattern: &str) -> String {
    let result = crate::compiler::intermediate_products(input);
    if !result.errors.is_empty() {
        return format!("// Error:\n{}", result.errors.join("\n"));
    }
    let mut output_modules = Vec::new();
    let regex_pattern = regex::Regex::new(&format!("^({output_modules_pattern})$")).unwrap();
    for module_name in result.defined_func_module_list.clone() {
        if regex_pattern.is_match(&module_name) {
            output_modules.push(module_name);
        }
    }
    for module_name in result.defined_non_func_module_list.clone() {
        if regex_pattern.is_match(&module_name) {
            output_modules.push(module_name);
        }
    }
    let mut modules = Vec::new();
    for module_name in &output_modules {
        let binary = match crate::compiler::serialize(result.clone(), module_name.as_str()) {
            Ok(v) => v,
            Err(err) => return format!("// Error: {err}"),
        };
        modules.push(Module::new(binary).unwrap());
    }
    match crate::transpiler::ts_transpiler::transpile(modules, false) {
        Ok(v) => v,
        Err(err) => format!("// Error: {err}"),
    }
}

#[wasm_bindgen(js_name=TranspileTSresId)]
pub fn export_ts_transpile_res_id(modules_res_id: Vec<u32>) -> String {
    let mut modules = Vec::new();
    for res_id in modules_res_id {
        let mut vmres = match VM_RESOURCE.lock() {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(module) = vmres.get_resource(res_id) {
            modules.push(module.clone());
        }
    }
    match crate::transpiler::ts_transpiler::transpile(modules, false) {
        Ok(v) => v,
        Err(err) => format!("// Error: {err}"),
    }
}

#[wasm_bindgen(js_name=VMreset)]
pub fn export_vmreset(resource_id: u32) -> Result<(), String> {
    let mut vmres = match VM_RESOURCE.lock() {
        Ok(v) => v,
        Err(_) => return Err("Mutex error".to_string()),
    };
    match vmres.get_resource(resource_id) {
        Some(module) => {
            module.reset();
            Ok(())
        }
        None => Err(format!("Resource not found: {resource_id}")),
    }
}

#[wasm_bindgen(js_name=VMset)]
pub fn export_vmset(resource_id: u32, index: u32, value: bool) -> Result<(), String> {
    let mut vmres = match VM_RESOURCE.lock() {
        Ok(v) => v,
        Err(_) => return Err("Mutex error".to_string()),
    };
    match vmres.get_resource(resource_id) {
        Some(module) => module.set(index, value),
        None => Err(format!("Resource not found: {resource_id}")),
    }
}

#[wasm_bindgen(js_name=VMgetOutput)]
pub fn export_vmget_output(resource_id: u32) -> Vec<u32> {
    let mut vmres = match VM_RESOURCE.lock() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    match vmres.get_resource(resource_id) {
        Some(module) => match module.get_output() {
            Ok(v) => v.into_iter().map(|b| if b { 1 } else { 0 }).collect(),
            Err(_) => Vec::new(),
        },
        None => Vec::new(),
    }
}

#[wasm_bindgen(js_name=VMgetGates)]
pub fn export_vmget_gates(resource_id: u32) -> Vec<u32> {
    let mut vmres = match VM_RESOURCE.lock() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    match vmres.get_resource(resource_id) {
        Some(module) => module
            .get_gates()
            .into_iter()
            .map(|b| if b { 1 } else { 0 })
            .collect(),
        None => Vec::new(),
    }
}

#[wasm_bindgen(js_name=VMgetTick)]
pub fn export_vmget_tick(resource_id: u32) -> u128 {
    let mut vmres = match VM_RESOURCE.lock() {
        Ok(v) => v,
        Err(_) => return 0,
    };
    match vmres.get_resource(resource_id) {
        Some(module) => module.get_tick(),
        None => 0,
    }
}

#[wasm_bindgen(js_name=VMgetTickAsStr)]
pub fn export_vmget_tick_as_str(resource_id: u32) -> String {
    let mut vmres = match VM_RESOURCE.lock() {
        Ok(v) => v,
        Err(_) => return "Mutex error".to_string(),
    };
    match vmres.get_resource(resource_id) {
        Some(module) => module.get_tick().to_string(),
        None => format!("Resource not found: {resource_id}"),
    }
}

#[wasm_bindgen(js_name=VMnext)]
pub fn export_vmnext(resource_id: u32, n: u32) -> Result<u128, String> {
    let mut vmres = match VM_RESOURCE.lock() {
        Ok(v) => v,
        Err(_) => return Err("Mutex error".to_string()),
    };
    match vmres.get_resource(resource_id) {
        Some(module) => module.next(n),
        None => Err(format!("Resource not found: {resource_id}")),
    }
}

#[wasm_bindgen(js_name=VMnextAsStr)]
pub fn export_vmnext_as_str(resource_id: u32, n: u32) -> Result<String, String> {
    let mut vmres = match VM_RESOURCE.lock() {
        Ok(v) => v,
        Err(_) => return Err("Mutex error".to_string()),
    };
    match vmres.get_resource(resource_id) {
        Some(module) => module.next(n).map(|tick| tick.to_string()),
        None => Err(format!("Resource not found: {resource_id}")),
    }
}
