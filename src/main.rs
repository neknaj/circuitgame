#[cfg(not(feature = "web"))]
mod compiler;
#[cfg(not(feature = "web"))]
mod test;
#[cfg(not(feature = "web"))]
mod vm;
#[cfg(not(feature = "web"))]
use clap::Parser;

// コマンドライン引数
#[derive(Parser, Debug)]
struct Opt {
    /// Input file
    #[arg(short = 'i', long = "input", value_name = "Input File Path")]
    input: Option<String>,
    #[arg(short = 'o', long = "output", value_name = "Output NCGb to File")]
    output: Option<String>,
}

#[cfg(not(feature = "web"))]
fn main() {
    println!("Neknaj Circuit Game");
    // 引数を処理
    let opt = Opt::parse();
    let input_path = match opt.input {
        Some(v) => v,
        None => {
            println!("No input path specified in command line arguments");
            return;
        },
    };
    let output_path = opt.output;
    // inputを読み込み
    let input = match std::fs::read_to_string(input_path) {
        Ok(v) => v,
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => println!("File not found"),
                std::io::ErrorKind::PermissionDenied => println!("Permission denied"),
                _ => println!("Other error occurred: {}", e),
            };
            return;
        }
    };

    // inputを処理
    let result = compiler::intermediate_products(&input);
    // println!("[result] {:#?}",result);
    println!("[warns] {:#?}",&result.warns);
    println!("[errors] {:#?}",&result.errors);
    println!("[sortedDependency] {:#?}",&result.module_dependency_sorted);
    if result.errors.len()>0 {return;}
    let module = match result.module_dependency_sorted.get(0) {
        Some(v) => v.clone(),
        None => {return;}
    };
    let binary = match compiler::serialize(result.clone(), module.as_str()) {
        Ok(v)=>v,
        Err(v)=>{
            println!("[error] {:#?}",v);
            return;
        }
    };
    let test_result = test::test(result);
    println!("[test warns] {:#?}",&test_result.warns);
    println!("[test errors] {:#?}",&test_result.errors);
    // コンパイル結果をoutput
    match output_path {
        Some(v)=> {
            if let Err(e) = write_binary_file(v.as_str(), binary) {
                eprintln!("[error]:output {}", e);
            } else {
                println!("Output completed");
            }
        },
        None => {
            println!("[info] No output path specified in command line arguments");
            println!("{:?}",binary);
        }
    }
    return;
}

#[cfg(not(feature = "web"))]
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

#[cfg(feature = "web")]
fn main() {
}