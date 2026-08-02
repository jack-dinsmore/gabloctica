mod memory;
use std::borrow::Cow;

use crate::{Command, bytecode::GlobalFunction, machine::memory::Memory, util::Tagged};

pub type Instructions = Tagged<InstructionData>;

pub struct InstructionData {
    pub instructions: Cow<'static, [u8]>,
}
impl InstructionData {
    pub fn from_compiled(instructions: &[u8]) -> Self {
        let instructions = Cow::Owned(instructions.to_vec());
        // Process the script
        Self {
            instructions,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MachineError {
    Stack,
    Memory,
    Ip,
    Func,
    OpCode,
}

pub enum MachineOutput<'a> {
    None,
    Call { func: GlobalFunction, args: &'a [f64]},
}

#[derive(Clone)]
pub struct Machine {
    pub stack: Vec<f64>,
    memory: Memory,
    pub ip: usize,
    pub interrupts: Vec<f64>,
    instructions: Instructions,
    max_lines_per_tick: usize
}

impl Machine {
    pub fn new(instructions: Instructions, max_lines_per_tick: usize) -> Self {
        let memory = Memory::new();
        Self {
            stack: Vec::new(),
            ip: 0,
            memory,
            instructions,
            interrupts: Vec::new(),
            max_lines_per_tick,
        }
    }

    /// Run until a call is encountered, or tick. Run the function call.
    pub fn run_to_call<'a> (&'a mut self) -> Result<MachineOutput<'a>,MachineError> {
        for _ in 0..self.max_lines_per_tick {
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
                    self.stack.push(((a != 0.) && (b != 0.)) as i64 as f64);
                },
                Command::Or => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(((a != 0.) || (b != 0.)) as i64 as f64);
                },
                Command::Xor => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    let b = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push(((a != 0.) ^ (b != 0.)) as i64 as f64);
                },
                Command::Not => {
                    let a = self.stack.pop().ok_or(MachineError::Stack)?;
                    self.stack.push((!(a != 0.)) as i64 as f64);
                },
                Command::Call => {
                    let func = GlobalFunction::try_from(u8::from_le_bytes([
                        self.instructions.instructions[self.ip+1],
                    ])).map_err(|_| MachineError::OpCode)?;
                    let arg_index = self.stack.pop().ok_or(MachineError::Stack)?.round() as u32;
                    let args = self.memory.access(arg_index).ok_or(MachineError::Memory)?;
                    self.ip += 1;
                    self.ip += 1;
                    return Ok(MachineOutput::Call{func, args});
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
                    self.stack.push(self.memory.allocate() as f64)
                },
                Command::Drp => {
                    let address = self.stack.pop().ok_or(MachineError::Stack)?.round() as u32;
                    self.memory.drop(address).ok_or(MachineError::Memory)?;
                },
                Command::Ld => {
                    let address = self.stack.pop().ok_or(MachineError::Stack)?.round() as u32;
                    let index = self.stack.pop().ok_or(MachineError::Stack)?.round() as u32;
                    self.stack.push(self.memory.load(address, index).ok_or(MachineError::Memory)?);
                },
                Command::St => {
                    let item = self.stack.pop().ok_or(MachineError::Stack)?;
                    let address = self.stack.pop().ok_or(MachineError::Stack)?.round() as u32;
                    let index = self.stack.pop().ok_or(MachineError::Stack)?.round() as u32;
                    self.memory.store(address, index, item).ok_or(MachineError::Memory)?;
                },
                Command::Stb => {
                    let item = self.stack.pop().ok_or(MachineError::Stack)?;
                    let index = self.stack.pop().ok_or(MachineError::Stack)?.round() as u32;
                    self.memory.store_back(index, item).ok_or(MachineError::Memory)?;
                },
                Command::Roll => {
                    let dist = self.stack.pop().ok_or(MachineError::Stack)?.round() as usize;
                    let len = self.stack.len();
                    self.stack[(len - dist)..len].rotate_left(1);
                }
                Command::Rolr => {
                    let dist = self.stack.pop().ok_or(MachineError::Stack)?.round() as usize;
                    let len = self.stack.len();
                    self.stack[(len - dist)..len].rotate_right(1);
                },
            }
            self.ip += 1;
        }
        Ok(MachineOutput::None)
    }
    
    pub fn reset(&mut self) {
        self.stack.clear();
        self.ip = 0;
    }
}