use serde::Serialize;

/// Errのレベル: 一つ目は停止 二つ目は警告
/// `Ok(( result, warn[] ))` `Err(( error[], warn[] ))`
pub type Warns = Vec<String>;
pub type Errs = Vec<String>;
pub type ResultwithWarn<T> = Result<(T,Warns),(Errs,Warns)>;


#[derive(Debug, Clone, Serialize)]
pub struct TestPattern {
    pub accept: bool,
    pub input : Vec<bool>,
    pub expect: Vec<bool>,
    pub output: Vec<bool>,
}


#[derive(Debug, Clone, Serialize)]
pub struct IntermediateProducts {
    pub warns      : Vec<String>,
    pub errors     : Vec<String>,
    pub test_list  : Vec<String>,
    pub test_result: std::collections::HashMap<String,Vec<TestPattern>>,
}