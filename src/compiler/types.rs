use serde::Serialize;

/// Errのレベル: 一つ目は停止 二つ目は警告
/// `Ok(( result, warn[] ))` `Err(( error[], warn[] ))`
pub type Warns = Vec<String>;
pub type Errs = Vec<String>;
pub type ResultwithWarn<T> = Result<(T,Warns),(Errs,Warns)>;

// パーサー系

#[derive(Debug, Clone, Serialize)]
pub struct File {
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Component {
    Using(Using),
    Module(Module),
    Test(Test),
}

#[derive(Debug, Clone, Serialize)]
pub struct Using {
    pub type_sig: MType,
}

#[derive(Debug, Clone, Serialize)]
pub struct Module {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub gates: Vec<Gate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Test {
    pub name: String,
    pub type_sig: MType,
    pub patterns: Vec<TestPattern>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Gate {
    pub outputs: Vec<String>,
    pub module_name: String,
    pub inputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MType {
    pub input_count: usize,
    pub output_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestPattern {
    pub inputs: Vec<bool>,
    pub outputs: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleType {
    pub name: String,
    pub mtype: MType,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeDepends {
    pub node: String,
    pub depends: String,
}

// norに展開

// 入力のgateのインデックスを保存
// 入力がmoduleのinputの場合は、gates.len()+indexにする
pub type CompiledGate = (u32,u32);

#[derive(Debug, Clone, Serialize)]
pub struct CompiledModule {
    pub inputs: u32,
    pub outputs: Vec<u32>,
    pub gates: Vec<CompiledGate>,
}

// compileの返り値

#[derive(Debug, Clone, Serialize)]
pub struct IntermediateProducts {
    pub warns                   : Vec<String>,
    pub errors                  : Vec<String>,
    pub ast                     : File,
    pub module_type_list        : Vec<ModuleType>,
    pub module_dependency       : Vec<NodeDepends>,
    pub module_dependency_sorted: Vec<String>,
    pub expanded_modules        : std::collections::HashMap<String,CompiledModule>,
}