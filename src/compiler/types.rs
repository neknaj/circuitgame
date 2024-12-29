use serde::Serialize;

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