use super::test;
use super::vm;
use super::compiler;
use colored::*;

pub fn main(input_path: String, output_path: Option<String>) {
    // inputを読み込み
    println!("{}:{} input file: {}","[info]".green(),"input".cyan(),input_path.clone());
    let input = match std::fs::read_to_string(input_path) {
        Ok(v) => v,
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => println!("{}:{} File not found","[error]".red(),"arguments".cyan()),
                std::io::ErrorKind::PermissionDenied => println!("{}:{} Permission denied","[error]".red(),"arguments".cyan()),
                _ => println!("{}:{} {}","[error]".red(),"arguments".cyan(),e),
            };
            return;
        }
    };
    // inputを処理
    let result = compiler::intermediate_products(&input);
    // println!("[result] {:#?}",result);
    for i in &result.warns {
        println!("{}:{} {}","[warn]".yellow(),"compile".cyan(),i);
    }
    for i in &result.errors {
        println!("{}:{} {}","[error]".red(),"compile".cyan(),i);
    }
    println!("sortedDependency {:?}",&result.module_dependency_sorted);
    if result.errors.len()>0 {return;}
    let module = match result.module_dependency_sorted.get(0) {
        Some(v) => v.clone(),
        None => {return;}
    };
    let binary = match compiler::serialize(result.clone(), module.as_str()) {
        Ok(v)=>v,
        Err(v)=>{
            println!("{}:{} {}","[error]".red(),"serialize".cyan(),v);
            return;
        }
    };
    let test_result = test::test(result);
    for i in &test_result.warns {
        println!("{}:{} {}","[warn]".yellow(),"test".cyan(),i);
    }
    for i in &test_result.errors {
        println!("{}:{} {}","[error]".red(),"test".cyan(),i);
    }
    // コンパイル結果をoutput
    match output_path {
        Some(v)=> {
            if let Err(e) = write_binary_file(v.as_str(), binary) {
                println!("{}:{} {}","[error]".red(),"output".cyan(),e);
            } else {
                println!("{}:{} Output completed","[info]".green(),"output".cyan());
            }
        },
        None => {
            println!("{}:{} No output path specified in command line arguments","[info]".green(),"output".cyan());
            println!("{:?}",binary);
        }
    }
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
