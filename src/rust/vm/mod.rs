pub mod types;
pub mod deserializer;

use types::*;

// use std::sync::Mutex;


impl Module {
    pub fn new(data: Vec<u32>) -> Result<Self,String> {
        deserializer::deserialize_from_vec(&data)
    }
    pub fn reset(&mut self) {
        self.cond.fill(false);
        self.tick=0;
    }
    /// inputを指定した値にする
    pub fn set(&mut self,index: u32,value: bool) -> Result<(),String> {
        match self.cond.get_mut(index as usize + &self.gates_sequential.len() + &self.gates_symmetry.len()) {
            Some(v)=>{
                *v = value;
            },
            None =>{return Err(format!("Index out of bounds"));}
        };
        Ok(())
    }
    /// inputを反転する
    pub fn inv(&mut self,index: u32) -> Result<(),String> {
        match self.cond.get_mut(index as usize + &self.gates_sequential.len() + &self.gates_symmetry.len()) {
            Some(v)=>{
                *v = !*v;
            },
            None =>{return Err(format!("Index out of bounds"));}
        };
        Ok(())
    }
    /// 全てのoutputを取得する
    pub fn get_output(&self) -> Result<GatesCond,String> {
        let mut outputs = Vec::new();
        for i in &self.outputs {
            match self.cond.get(*i as usize) {
                Some(v) => {
                    outputs.push(v.clone());
                }
                None => {return Err(format!("Index out of bounds"));}
            }
        }
        Ok(outputs)
    }
    /// 全てのinputを取得する
    pub fn get_input(&self) -> Result<GatesCond,String> {
        let mut inputs = Vec::new();
        let gates = self.gates_sequential.len() + &self.gates_symmetry.len();
        for i in 0..self.inputs {
            match self.cond.get(i as usize + gates) {
                Some(v) => {
                    inputs.push(v.clone());
                }
                None => {return Err(format!("Index out of bounds"));}
            }
        }
        Ok(inputs)
    }
    /// 全てのgatesを取得する input付き
    pub fn get_gates(&self) -> GatesCond {
        self.cond.clone()
    }
    /// 現在のtickを取得する
    pub fn get_tick(&self) -> u128 {
        self.tick
    }
    /// gatesをn周更新する
    pub fn next(&mut self,n: u32) -> Result<u128, String> {
        for _ in 0..n {
            let mut gate_index = 0;
            { // sequential
                for gate in &self.gates_sequential {
                    let input1 = *self.cond.get(gate.0 as usize).ok_or("gates access error")?;
                    let input2 = *self.cond.get(gate.1 as usize).ok_or("gates access error")?;
                    let output = self.cond.get_mut(gate_index).ok_or("gates access error")?;
                    *output = !(input1||input2); // input1,input2のNORを計算する
                    gate_index+=1;
                }
            }
            let mut gate_index = 0;
            { // symmetry
                let before_cond = self.cond.clone();
                for gate in &self.gates_symmetry {
                    let input1 = *before_cond.get(gate.0 as usize).ok_or("gates access error")?;
                    let input2 = *before_cond.get(gate.1 as usize).ok_or("gates access error")?;
                    let output = self.cond.get_mut(gate_index).ok_or("gates access error")?;
                    *output = !(input1||input2); // input1,input2のNORを計算する
                    gate_index+=1;
                }
            }
            self.tick+=1;
        }
        Ok(self.tick)
    }
}
