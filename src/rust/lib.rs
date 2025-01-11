#![cfg(feature = "web")]

mod compiler;
mod test;
mod vm;
mod resourcemanager;
mod transpiler;

use rand::seq::index;
use wasm_bindgen::prelude::*;
use std::str;

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

// VM


use std::sync::Mutex;
use crate::resourcemanager::*;
use crate::vm::types::Module;

lazy_static::lazy_static! {
    static ref VM_resource: Mutex<ResourceManager<Module>> = Mutex::new( ResourceManager::new() );
}

#[wasm_bindgen(js_name=VMinit)]
pub fn export_VMinit(data: Vec<u32>) -> Result<u32,String> {
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
pub fn export_VMgetTick(resource_id: u32) -> u32 {
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
#[wasm_bindgen(js_name=VMnext)]
pub fn export_VMnext(resource_id: u32,n: u32) -> Result<u32,String> {
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