pub type NORGate = (u32,u32);

#[derive(Clone)]
pub struct Module {
    pub func: bool,
    pub name: String,
    pub inputs: u32,
    pub outputs: Vec<u32>,
    pub gates_sequential: Vec<NORGate>,
    pub gates_symmetry: Vec<NORGate>,
    pub cond: GatesCond,
    pub tick: u128,
}

pub type GatesCond = Vec<bool>;
