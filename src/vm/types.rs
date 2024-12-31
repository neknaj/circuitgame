pub type NORGate = (u32,u32);

pub struct Module {
    pub inputs: u32,
    pub outputs: Vec<u32>,
    pub gates: Vec<NORGate>,
}