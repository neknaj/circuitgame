use colored::*;
use crate::native::document::document;
use crate::vm::deserializer::deserialize_from_vec;

use super::super::test;
use super::super::vm;
use super::super::compiler;

// 入力処理を別関数として分離
pub fn process_input(input_path: &str,output_modules_pattern: String, output_path: Vec<String>, doc_output_path: Option<String>) -> Vec<Vec<u32>> {
    println!("< {} >\n","Neknaj Circuit Game".bold());

    println!("{}:{} input  file: {}","[info]".green(),"input ".cyan(),input_path);
    println!("{}:{} output file: {:?}","[info]".green(),"output".cyan(),output_path);
    // inputを読み込み
    let input = match std::fs::read_to_string(input_path) {
        Ok(v) => v,
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => println!("{}:{} File not found","[error]".red(),"arguments".cyan()),
                std::io::ErrorKind::PermissionDenied => println!("{}:{} Permission denied","[error]".red(),"arguments".cyan()),
                _ => println!("{}:{} {}","[error]".red(),"arguments".cyan(),e),
            };
            return Vec::new();
        }
    };

    // inputを処理
    let result = compiler::intermediate_products(&input);

    for i in &result.warns {
        println!("{}:{} {}","[warn]".yellow(),"compile".cyan(),i);
    }
    for i in &result.errors {
        println!("{}:{} {}","[error]".red(),"compile".cyan(),i);
    }
    println!("sortedDependency {:?}",&result.module_dependency_sorted);

    if result.errors.len() > 0 {
        return Vec::new();
    }

    let test_result = test::test(result.clone());
    for i in &test_result.warns {
        println!("{}:{} {}","[warn]".yellow(),"test".cyan(),i);
    }
    for i in &test_result.errors {
        println!("{}:{} {}","[error]".red(),"test".cyan(),i);
    }

    let mut output_modules = Vec::new();
    let regex_pattern = regex::Regex::new(&format!("^{}$",output_modules_pattern)).unwrap();
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
    println!("{}:{} {}","[info]".green(),"modules".cyan(),format!("Module exports: {}",output_modules.join(", ")));

    for output in output_path {
        // outputのtypeを決定する
        let out_type = match output.split(":").nth(1) {
            // 明示されている場合
            Some(t) => t,
            // 拡張子から推定
            None => match &output {
                name if name.ends_with(".bin")  => "ncgb",
                name if name.ends_with(".ncgb") => "ncgb",
                name if name.ends_with(".c")    => "c",
                name if name.ends_with(".h")    => "cheader",
                name if name.ends_with(".d.ts") => "dts",
                name if name.ends_with(".ts")   => "ts",
                name if name.ends_with(".js")   => "js",
                // output_typeの推定に失敗
                _ => {
                    println!("{}:{} {}","[error]".red(),"output".cyan(),format!("Could not infer output type for {}",output));
                    continue;
                },
            },
        };
        // typeに基づいてoutput
        match out_type {
            "ncgb" => {
                if output_modules.len()>1 {
                    println!("{}:{} {}","[error]".red(),"output".cyan(),format!("NCGb output doesn't support multiple modules: {}",output));
                    println!("{}:{} {}","[info]".green(),"output".cyan(),format!("Only the first module was exported {}",output));
                }
                if output_modules.len()==0 {
                    println!("{}:{} {}","[warn]".green(),"output".cyan(),format!("No module is specified to output: {}",output));
                }
                if let Some(module_name) = output_modules.get(0) {
                    let binary = match compiler::serialize(result.clone(), module_name.as_str()) {
                        Ok(v) => v,
                        Err(v) => {
                            println!("{}:{} {}","[error]".red(),"serialize".cyan(),v);
                            return Vec::new()
                        }
                    };
                    if let Err(e) = write_binary_file(output.as_str(), binary.clone()) {
                        println!("{}:{} {}","[error]".red(),"output".cyan(),e);
                    } else {
                        println!("{}:{} Output completed: {}","[info]".green(),"output".cyan(),output);
                    }
                }
            },
            "c"|"cheader" => {
                if output_modules.len()>1 {
                    println!("{}:{} {}","[error]".red(),"transpile".cyan(),format!("NCGb output doesn't support multiple modules: {}",output));
                    println!("{}:{} {}","[info]".green(),"transpile".cyan(),format!("Only the first module was exported to {}",output));
                }
                if output_modules.len()==0 {
                    println!("{}:{} {}","[warn]".green(),"transpile".cyan(),format!("No module is specified to output: {}",output));
                }
                if let Some(module_name) = output_modules.get(0) {
                    let binary = match compiler::serialize(result.clone(), module_name.as_str()) {
                        Ok(v) => v,
                        Err(v) => {
                            println!("{}:{} {}","[error]".red(),"serialize".cyan(),v);
                            return Vec::new()
                        }
                    };
                    match crate::transpiler::c_transpiler::transpile(deserialize_from_vec(&binary).unwrap(),out_type=="cheader") {
                        Ok(data) => {
                            if let Err(e) = write_text_file(output.as_str(), &data) {
                                println!("{}:{} {}","[error]".red(),"output".cyan(),e);
                            } else {
                                println!("{}:{} Output completed: {}","[info]".green(),"transpile".cyan(),output);
                            }
                        },
                        Err(err) => {
                            println!("{}:{} {}","[error]".red(),"transpile".cyan(),err);
                        }
                    }
                }
            },
            "ts"|"dts" => {
                let mut modules = Vec::new();
                for module_name in &output_modules {
                    let binary = match compiler::serialize(result.clone(), module_name.as_str()) {
                        Ok(v) => v,
                        Err(v) => {
                            println!("{}:{} {}","[error]".red(),"serialize".cyan(),v);
                            return Vec::new()
                        }
                    };
                    modules.push(deserialize_from_vec(&binary).unwrap());
                }
                match crate::transpiler::ts_transpiler::transpile(modules,out_type=="dts") {
                    Ok(data) => {
                        if let Err(e) = write_text_file(output.as_str(), &data) {
                            println!("{}:{} {}","[error]".red(),"output".cyan(),e);
                        } else {
                            println!("{}:{} Output completed: {}","[info]".green(),"transpile".cyan(),output);
                        }
                    },
                    Err(err) => {
                        println!("{}:{} {}","[error]".red(),"transpile".cyan(),err);
                    }
                }
            },
            "js" => {
                if output_modules.len()>1 {
                    println!("{}:{} {}","[error]".red(),"transpile".cyan(),format!("NCGb output doesn't support multiple modules: {}",output));
                    println!("{}:{} {}","[info]".green(),"transpile".cyan(),format!("Only the first module was exported to {}",output));
                }
                if output_modules.len()==0 {
                    println!("{}:{} {}","[warn]".green(),"transpile".cyan(),format!("No module is specified to output: {}",output));
                }
                if let Some(module_name) = output_modules.get(0) {
                    let binary = match compiler::serialize(result.clone(), module_name.as_str()) {
                        Ok(v) => v,
                        Err(v) => {
                            println!("{}:{} {}","[error]".red(),"serialize".cyan(),v);
                            return Vec::new()
                        }
                    };
                    match crate::transpiler::js_transpiler::transpile(deserialize_from_vec(&binary).unwrap()) {
                        Ok(data) => {
                            if let Err(e) = write_text_file(output.as_str(), &data) {
                                println!("{}:{} {}","[error]".red(),"output".cyan(),e);
                            } else {
                                println!("{}:{} Output completed: {}","[info]".green(),"transpile".cyan(),output);
                            }
                        },
                        Err(err) => {
                            println!("{}:{} {}","[error]".red(),"transpile".cyan(),err);
                        }
                    }
                }
            },
            _ => {
                println!("{}:{} {}","[error]".red(),"output".cyan(),format!("Unsupported output type was specified: {} for {}",out_type,output));
            },
        };
    }
    match document(result.clone()) {
        Ok(doc_str)=>{
            match doc_output_path {
                Some(v)=> {
                    if let Err(e) = write_text_file(v.as_str(), &doc_str) {
                        println!("{}:{} {}","[error]".red(),"output".cyan(),e);
                    } else {
                        println!("{}:{} document output completed","[info]".green(),"output".cyan());
                    }
                },
                None => {
                    println!("{}:{} No document output path specified in command line arguments","[info]".green(),"output".cyan());
                }
            }
        },
        Err(v)=>{
            println!("{}:{} {}","[error]".red(),"document".cyan(),v);
        }
    };

    let mut binaries = Vec::new();
    for module_name in &output_modules {
        let binary = match compiler::serialize(result.clone(), module_name.as_str()) {
            Ok(v) => v,
            Err(v) => {
                println!("{}:{} {}","[error]".red(),"serialize".cyan(),v);
                return Vec::new()
            }
        };
        binaries.push(binary);
    }
    binaries
}

fn write_binary_file(filename: &str, data: Vec<u32>) -> std::io::Result<()> {use std::fs::File;
    use byteorder::{LittleEndian, WriteBytesExt};
    // ファイルの作成
    let mut file = File::create(filename)
        .map_err(|e| std::io::Error::new(e.kind(), format!("ファイル作成に失敗しました: {}", e)))?;

    // データの書き込み
    for &value in &data {
        file.write_u32::<LittleEndian>(value)
            .map_err(|e| std::io::Error::new(e.kind(), format!("データ書き込みに失敗しました: {}", e)))?;
    }

    Ok(())
}

fn write_text_file(file_path: &str, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::File::create(file_path)
        .map_err(|e| std::io::Error::new(e.kind(), format!("ファイル作成に失敗しました: {}", e)))?;
    file.write_all(content.as_bytes())
        .map_err(|e| std::io::Error::new(e.kind(), format!("データ書き込みに失敗しました: {}", e)))?;
    Ok(())
}




use tokio::time::{sleep, Duration, self, Instant};
use tokio::sync::broadcast;
pub async fn runVM(data: Vec<u32>, vmset_tx: broadcast::Sender<u32>, ws_tx: broadcast::Sender<String>) -> Result<(),String> {
    let mut rx = vmset_tx.subscribe();
    use crate::vm::types::Module;
    let mut vm_module = match Module::new(data) {
        Ok(v) => v,
        Err(_) => {
            println!("failed to init VM");
            return Err(format!("failed to init VM"));}
    };

    println!("");
    println!("VM start");
    println!("Press the number key in the index to switch inputs");
    println!("\n\n");
    loop {
        // まずVMを1ステップ実行
        let _ = vm_module.next(1);
        // outputをプリント
        println!("\x1B[4A\x1B[2K");
        println!("tick   {}",vm_module.get_tick());
        println!("input  {}",vm_module.get_input().unwrap().iter().map(|&b| if b {"t"}else{"f"}).collect::<Vec<_>>().join(" "));
        println!("output {}",vm_module.get_output().unwrap().iter().map(|&b| if b {"t"}else{"f"}).collect::<Vec<_>>().join(" "));
        // let _ = ws_tx.send(format!("tick:{},input:{:?},output:{:?}",
        //     vm_module.get_tick(),
        //     vm_module.get_input().unwrap().iter().map(|&b| if b {"t"}else{"f"}).collect::<Vec<_>>().join(""),
        //     vm_module.get_output().unwrap().iter().map(|&b| if b {"t"}else{"f"}).collect::<Vec<_>>().join(""),
        // ));
        // メッセージの確認（ノンブロッキング）
        if let Ok(index) = rx.try_recv() {
            // println!("Received VM setting: index={}",index);
            // ここでメッセージに基づく処理を実装
            // 例：VMの設定を更新するなど
            let _ = vm_module.inv(index);
        }
        // 必要に応じて短い待機を入れる
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}
