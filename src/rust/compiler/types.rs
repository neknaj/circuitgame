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
    Graphical(Graphical),
    Test(Test),
    Include(Include),
}

#[derive(Debug, Clone, Serialize)]
pub struct Include {
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Using {
    pub type_sig: MType,
}

#[derive(Debug, Clone, Serialize)]
pub struct Module {
    pub func: bool,
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub gates: Vec<Gate>,
}


#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ImgSize {
    Size {width: u32, height: u32},
    Auto(()),
}

#[derive(Debug, Clone, Serialize)]
pub struct Graphical {
    pub name: String,
    pub size: ImgSize,
    pub pixels: Vec<Pixel>,
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
pub struct PreOutputs {
    pub arr_name: String,
    pub arr_size: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreInputs {
    pub arr_name: String,
    pub arr_slice: ArrSlice,
}
#[derive(Debug, Clone, Serialize)]
pub struct ArrSlice {
    pub all: bool,
    // all=trueなら以下の内容を無視
    pub start: usize,
    pub end: usize,
    pub step: usize,
    pub lower_inclusive: bool,
    pub upper_inclusive: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreGate {
    pub outputs: Vec<PreOutputs>,
    pub module_name: String,
    pub inputs: Vec<PreInputs>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Pixel {
    pub coord: (u32,u32),
    pub io_index: IoIndex,
    pub color: PixelColor,
}

#[derive(Debug, Clone, Serialize)]
pub struct PixelColor {
    pub on: (u8,u8,u8),
    pub off: (u8,u8,u8),
}

#[derive(Debug, Clone, Serialize)]
pub struct IoIndex {
    pub io_type: String,
    pub index: u32,
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

#[derive(Debug, Clone, Serialize)]
pub enum CompiledGateInput {
    NorGate(u32), // gateのn番目
    Input(u32), // inputのn番目
}

// 入力のgateのインデックスを保存
pub type CompiledGate = (CompiledGateInput,CompiledGateInput);

#[derive(Debug, Clone, Serialize)]
pub struct CompiledModule {
    pub func: bool,
    pub name: String,
    pub inputs: u32,
    pub outputs: Vec<u32>,
    pub gates_sequential: Vec<CompiledGate>,
    pub gates_symmetry: Vec<CompiledGate>,
}

// compileの返り値

#[derive(Debug, Clone, Serialize)]
pub struct IntermediateProducts {
    pub source                      : String,
    pub warns                       : Vec<String>,
    pub errors                      : Vec<String>,
    pub ast                         : File,
    pub defined_non_func_module_list: Vec<String>,
    pub defined_func_module_list    : Vec<String>,
    pub module_type_list            : Vec<ModuleType>,
    pub module_dependency           : Vec<NodeDepends>,
    pub module_dependency_sorted    : Vec<String>,
    pub expanded_modules            : std::collections::HashMap<String,CompiledModule>,
}