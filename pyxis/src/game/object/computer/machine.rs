use std::{cell::RefCell, rc::Rc};

use cgmath::{Rotation, Vector3};

use crate::{game::object::{computer::{Instructions, memory::Memory}, internals::CommandBlockInfo}, quahog::Command};

const MAX_LINES_PER_TICK: usize = 100;

pub enum MachineError {
    Stack,
    Memory,
    Ip,
    Func,
    OpCode,
}

pub struct Machine {
    stack: Vec<f64>,
    memory: Rc<RefCell<Memory>>,
    ip: usize,
    pub interrupts: Vec<f64>,
    instructions: Instructions,
}

impl Machine {
    pub fn new(instructions: Instructions, memory: Rc<RefCell<Memory>>) -> Self {
        Self {
            stack: Vec::new(),
            ip: 0,
            memory,
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
            let command = match Command::try_from(self.instructions.instructions[self.ip]) {
                Ok(c) => c,
                _ => return Err(MachineError::OpCode)
            };
            match command {
                Command::Nop => (),
                Command::Push => {
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
                Command::Pop => { self.stack.pop().ok_or(MachineError::Stack)?; },
                Command::Dup => { self.stack.push(*self.stack.last().ok_or(MachineError::Stack)?); }
                Command::Pip => { self.stack.push(self.ip as f64); },
                Command::Jpop => { self.ip = self.stack.pop().ok_or(MachineError::Stack)?.round() as usize; },
                
                Command::Jmp => {
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
                Command::Jnz => {
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
                Command::Lt => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push((a < b) as i64 as f64)
                },
                Command::Gt => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push((a > b) as i64 as f64)
                }, 
                Command::Le => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push((a <= b) as i64 as f64)
                },
                Command::Ge => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push((a >= b) as i64 as f64)
                },
                Command::Eq => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push((a == b) as i64 as f64)
                },
                Command::Add => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(a + b)
                },
                Command::Sub => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(a - b)
                },
                Command::Mul => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(a * b)
                },
                Command::Div => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(a / b)
                },
                Command::Neg => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(-a)
                },
                Command::Pow => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(a.powf(b))
                },
                Command::And => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(((a != 0.) && (a != 0.)) as i64 as f64);
                },
                Command::Or => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(((a != 0.) || (a != 0.)) as i64 as f64);
                },
                Command::Xor => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(((a != 0.) ^ (a != 0.)) as i64 as f64);
                },
                Command::Not => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push((!(a != 0.)) as i64 as f64);
                },
                Command::Call => {
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
                Command::Tick => {
                    self.ip += 1;
                    return Ok(None);
                },
                Command::Irp => {
                    self.stack.push(self.interrupts.pop().unwrap_or(0.));
                },
                Command::Swp => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(a);
                    self.stack.push(b);
                },
                Command::Pick => {
                    let index = self.stack.pop().ok_or(MachineError::Stack)?;
                    let index = (index.round() as u32) as usize;
                    let b = self.stack.iter().nth(index).ok_or(MachineError::Stack)?;
                    self.stack.push(*b);
                },
                Command::Alc => {
                    self.stack.push(self.memory.borrow_mut().allocate() as f64)
                },
                Command::Drp => {
                    let address = self.stack.pop().ok_or(MachineError::Stack)?.round() as u32;
                    self.memory.borrow_mut().drop(address).ok_or(MachineError::Memory)?;
                },
                Command::Ld => {
                    let address = self.stack.pop().ok_or(MachineError::Stack)?.round() as u32;
                    let index = self.stack.pop().ok_or(MachineError::Stack)?.round() as u32;
                    self.stack.push(self.memory.borrow().load(address, index).ok_or(MachineError::Memory)?);
                },
                Command::Str => {
                    let item = self.stack.pop().ok_or(MachineError::Stack)?;
                    let address = self.stack.pop().ok_or(MachineError::Stack)?.round() as u32;
                    let index = self.stack.pop().ok_or(MachineError::Stack)?.round() as u32;
                    self.memory.borrow_mut().store(address, index, item).ok_or(MachineError::Memory)?;
                },
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