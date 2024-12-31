use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct IntermediateProducts {
    pub warns                   : Vec<String>,
    pub errors                  : Vec<String>,
    pub test_list               : Vec<String>,
}