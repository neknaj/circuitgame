mod compiler;
mod test;
mod vm;
mod transpiler;
use clap::Parser;
mod native;

// コマンドライン引数
#[derive(Parser, Debug)]
struct Opt {
    /// Input file
    #[arg(short = 'i', long = "input", value_name = "Input File Path")]
    input: Option<String>,
    #[arg(short = 'o', long = "output", value_name = "Output File Path")]
    output: Vec<String>,
    #[arg(short = 'd', long = "docOut", value_name = "Document output File Path")]
    doc_output: Option<String>,
    #[arg(short = 'm', long = "module", value_name = "Name of module to compile")]
    module: Option<String>,
    #[arg(short = 's', long = "server", value_name = "Open server for API")]
    server: Option<bool>,
    #[arg(long = "vm", value_name = "Run VM")]
    run_vm: Option<bool>,
}

#[cfg(not(feature = "web"))]
#[tokio::main]
async fn main() {
    use colored::*;
    // 引数を処理
    let opt = Opt::parse();
    let input_path = match opt.input {
        Some(v) => v,
        None => {
            println!("{}:{} No input path specified in command line arguments","[error]".red(),"arguments".cyan());
            return;
        },
    };
    let server_launch = opt.server.unwrap_or(false);
    if !server_launch {
        native::withoutwatch::main(input_path,opt.output, opt.doc_output, opt.module).await;
    }
    else {
        native::watch::main(input_path, opt.output, opt.doc_output, opt.module, opt.run_vm.unwrap_or(false)).await;
    }
    return;
}

#[cfg(not(feature = "web"))]
#[cfg(feature = "web")]
fn main() {
}