use super::common::process_input;

pub async fn main(input_path: String, output_path: Vec<String>, doc_output_path: Option<String>, module: Option<String>) {
    let _ = process_input(&input_path, module, output_path, doc_output_path);
}