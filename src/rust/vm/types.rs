pub type NORGate = (u32,u32);

pub struct Module {
    pub func: bool,
    pub inputs: u32,
    pub outputs: Vec<u32>,
    pub gates: Vec<NORGate>,
    pub cond: GatesCond,
    pub tick: u32,
}

pub type GatesCond = Vec<bool>;
