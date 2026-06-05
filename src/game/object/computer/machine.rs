use cgmath::{Rotation, Vector3};

use crate::game::object::{computer::Instructions, internals::CommandBlockInfo};

const MAX_LINES_PER_TICK: usize = 100;

pub enum MachineError {
    Stack,
    Ip,
    Func,
    OpCode,
}

pub struct Machine {
    stack: Vec<f64>,
    ip: usize,
    pub interrupts: Vec<f64>,
    instructions: Instructions,
}

impl Machine {
    pub fn new(instructions: Instructions) -> Self {
        Self {
            stack: Vec::new(),
            ip: 0,
            instructions,
            interrupts: Vec::new(),
        }
    }

    /// Run until a call is encountered, or tick. Run the function call.
    fn run_to_call(&mut self) -> Result<Option<u64>,MachineError> {
        for _ in 0..MAX_LINES_PER_TICK {
            if self.ip > self.instructions.instructions.len() {
                return Err(MachineError::Ip)
            }
            match self.instructions.instructions[self.ip] {
                0 => (),// Nop
                1 => { // Push
                    self.stack.push(f64::from_le_bytes([
                        self.instructions.instructions[self.ip+1],
                        self.instructions.instructions[self.ip+2],
                        self.instructions.instructions[self.ip+3],
                        self.instructions.instructions[self.ip+4],
                        self.instructions.instructions[self.ip+5],
                        self.instructions.instructions[self.ip+6],
                        self.instructions.instructions[self.ip+7],
                        self.instructions.instructions[self.ip+8]
                    ]));
                    self.ip += 8;
                },
                2 => { self.stack.pop().ok_or(MachineError::Stack)?; }, // Pop
                3 => { self.stack.push(*self.stack.last().ok_or(MachineError::Stack)?); } // Dup
                4 => { self.stack.push(self.ip as f64); }, // Puship
                5 => { self.ip = self.stack.pop().ok_or(MachineError::Stack)?.round() as usize; }, // jpop
                
                6 => { // Jmp
                    self.ip = u64::from_le_bytes([
                        self.instructions.instructions[self.ip+1],
                        self.instructions.instructions[self.ip+2],
                        self.instructions.instructions[self.ip+3],
                        self.instructions.instructions[self.ip+4],
                        self.instructions.instructions[self.ip+5],
                        self.instructions.instructions[self.ip+6],
                        self.instructions.instructions[self.ip+7],
                        self.instructions.instructions[self.ip+8]
                    ]) as usize - 1;
                },
                7 => { // Jmp if not equal
                    if self.stack.pop().ok_or(MachineError::Stack)? != 0. {
                        self.ip = u64::from_le_bytes([
                            self.instructions.instructions[self.ip+1],
                            self.instructions.instructions[self.ip+2],
                            self.instructions.instructions[self.ip+3],
                            self.instructions.instructions[self.ip+4],
                            self.instructions.instructions[self.ip+5],
                            self.instructions.instructions[self.ip+6],
                            self.instructions.instructions[self.ip+7],
                            self.instructions.instructions[self.ip+8]
                        ]) as usize;
                    } else {
                        self.ip += 8;
                    }
                },
                8 => { // Less
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push((a < b) as i64 as f64)
                },
                9 => { // Greater
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push((a > b) as i64 as f64)
                }, 
                10 => { // Less equal
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push((a <= b) as i64 as f64)
                },
                11 => { // Greater equal
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push((a >= b) as i64 as f64)
                },
                12 => { // Equals
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push((a == b) as i64 as f64)
                },
                13 => { // Float add
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(a + b)
                },
                14 => { // Float sub
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(a - b)
                },
                15 => { // Float mul
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(a * b)
                },
                16 => { // Float div
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(a / b)
                },
                17 => { // Float negate
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(-a)
                },
                18 => { // Float power
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(a.powf(b))
                },
                19 => { // And
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(((a != 0.) && (a != 0.)) as i64 as f64);
                },
                20 => { // Or
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(((a != 0.) || (a != 0.)) as i64 as f64);
                },
                21 => { // Xor
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(((a != 0.) ^ (a != 0.)) as i64 as f64);
                },
                22 => { // not
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push((!(a != 0.)) as i64 as f64);
                },
                23 => { self.stack.remove(self.stack.len()-2); },// dupn
                24 => { self.stack.push(*self.stack.get(self.stack.len()-2).ok_or(MachineError::Stack)?); },// dupn

                25 => {// call
                    let function = u64::from_le_bytes([
                        self.instructions.instructions[self.ip+1],
                        self.instructions.instructions[self.ip+2],
                        self.instructions.instructions[self.ip+3],
                        self.instructions.instructions[self.ip+4],
                        self.instructions.instructions[self.ip+5],
                        self.instructions.instructions[self.ip+6],
                        self.instructions.instructions[self.ip+7],
                        self.instructions.instructions[self.ip+8]
                    ]);
                    self.ip += 8;
                    self.ip += 1;
                    return Ok(Some(function));
                },
                26 => {// tick
                    self.ip += 1;
                    return Ok(None);
                },
                27 => {// Query interrupt
                    self.stack.push(self.interrupts.pop().unwrap_or(0.));
                },
                28 => {// Swap
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(a);
                    self.stack.push(b);
                },
                _ => return Err(MachineError::OpCode)
            }
            self.ip += 1;
        }
        Ok(None)
    }

    pub fn tick(&mut self, block: &mut CommandBlockInfo) -> Result<(), MachineError> {
        let mut message_index = 0;
        while let Some(function) = self.run_to_call()? {
            match function {
                0 => {// Print
                    let arg = self.stack.pop().ok_or(MachineError::Stack)?;
                    println!("Breakpoint {}", arg);
                },
                1 => {// Add force
                    let mut force = Vector3::new(
                        self.stack.pop().ok_or(MachineError::Stack)?,
                        self.stack.pop().ok_or(MachineError::Stack)?,
                        self.stack.pop().ok_or(MachineError::Stack)?
                    );
                    force = block.quat.rotate_vector(force);
                    let torque = block.pos.cross(force);
                    let force = block.body.ori.rotate_vector(force);
                    block.body.add_force(force);
                    block.body.add_torque(torque);
                },
                2 => {// Add torque
                    let torque = Vector3::new(self.stack.pop().ok_or(MachineError::Stack)?, self.stack.pop().ok_or(MachineError::Stack)?, self.stack.pop().ok_or(MachineError::Stack)?);
                    block.body.add_torque(torque);
                },
                3 => {// Emit signal
                    let recv_block = self.stack.pop().ok_or(MachineError::Stack)?.round() as u8;
                    let n_send = self.stack.pop().ok_or(MachineError::Stack)?.round() as usize;
                    
                    let mut data = Vec::with_capacity(n_send);
                    for _ in 0..n_send {
                        data.push(self.stack.pop().ok_or(MachineError::Stack)?);
                    }
                    
                    match &mut block.circuit {
                        Some(c) => {
                            c.send(recv_block, data)
                        },
                        None => (),
                    }
                },
                4 => {// Receive signal
                    let arg = self.stack.pop().ok_or(MachineError::Stack)?;
                    let jtrue = u64::from_le_bytes(self.stack.pop().ok_or(MachineError::Stack)?.to_le_bytes()) as usize;
                    let jfalse = u64::from_le_bytes(self.stack.pop().ok_or(MachineError::Stack)?.to_le_bytes()) as usize;
                    match &block.circuit {
                        Some(c) => {
                            let (data_option, new_message_index) = c.recv(block.id, message_index);
                            match data_option {
                                Some(data) => {
                                    for d in data {
                                        self.stack.push(*d);
                                    }
                                    self.stack.push(data.len() as f64);
                                    self.ip = jtrue;
                                },
                                None => {
                                    self.ip = jfalse;
                                }
                            }
                            message_index = new_message_index;
                        },
                        None => self.ip = jfalse,
                    }
                },
                _ => return Err(MachineError::Func)
            }
        }
        Ok(())
    }
    
    pub fn reset(&mut self) {
        self.stack.clear();
        self.ip = 0;
    }
}