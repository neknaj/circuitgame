mod compiler;
mod test;
mod vm;
use clap::Parser;
mod native;

// コマンドライン引数
#[derive(Parser, Debug)]
struct Opt {
    /// Input file
    #[arg(short = 'i', long = "input", value_name = "Input File Path")]
    input: Option<String>,
    #[arg(short = 'o', long = "output", value_name = "Output NCGb to File")]
    output: Option<String>,
    #[arg(short = 's', long = "server", value_name = "Open server for API")]
    server: Option<bool>,
}

#[cfg(not(feature = "web"))]
#[tokio::main]
async fn main() {
    use colored::*;
    println!("< {} >","Neknaj Circuit Game".bold());
    // 引数を処理
    let opt = Opt::parse();
    let input_path = match opt.input {
        Some(v) => v,
        None => {
            println!("{}:{} No input path specified in command line arguments","[error]".red(),"arguments".cyan());
            return;
        },
    };
    let output_path = opt.output;
    let server_launch = opt.server.unwrap_or(false);
    if !server_launch {
        native::withoutserver::main(input_path, output_path);
    }
    else {
        native::server::main().await;
    }
    return;
}

#[cfg(not(feature = "web"))]
#[cfg(feature = "web")]
fn main() {
}