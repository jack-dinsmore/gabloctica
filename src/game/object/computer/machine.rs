use crate::game::object::computer::Instructions;

const MAX_LINES_PER_TICK: usize = 100;

pub struct Machine {
    stack: Vec<f64>,
    ip: usize,
    pub calls: Vec<f64>,
    pub interrupts: Vec<f64>,
    instructions: Instructions,
}

impl Machine {
    pub fn new(instructions: Instructions) -> Self {
        Self {
            stack: Vec::new(),
            ip: 0,
            instructions,
            calls: Vec::new(),
            interrupts: Vec::new(),
        }
    }

    pub fn tick(&mut self) -> Option<()> {
        for _ in 0..MAX_LINES_PER_TICK {
            if self.instructions.instructions.len() == 35 {
                dbg!(self.ip, &self.stack);
            };
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
                2 => { self.stack.pop()?; }, // Pop
                3 => { self.stack.push(*self.stack.last()?); } // Dup
                4 => { self.stack.push(self.ip as f64); }, // Puship
                5 => { self.ip = self.stack.pop()?.round() as usize; }, // Puship
                
                6 => { // Jmp
                    self.ip = u32::from_le_bytes([
                        self.instructions.instructions[self.ip+1],
                        self.instructions.instructions[self.ip+2],
                        self.instructions.instructions[self.ip+3],
                        self.instructions.instructions[self.ip+4],
                    ]) as usize - 1;
                },
                7 => { // Jmp if not equal
                    if self.stack.pop()? != 0. {
                        self.ip = u32::from_le_bytes([
                            self.instructions.instructions[self.ip+1],
                            self.instructions.instructions[self.ip+2],
                            self.instructions.instructions[self.ip+3],
                            self.instructions.instructions[self.ip+4],
                        ]) as usize;
                    } else {
                        self.ip += 4;
                    }
                },
                8 => { // Less
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push((a < b) as i64 as f64)
                },
                9 => { // Greater
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push((a > b) as i64 as f64)
                }, 
                10 => { // Less equal
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push((a <= b) as i64 as f64)
                },
                11 => { // Greater equal
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push((a >= b) as i64 as f64)
                },
                12 => { // Equals
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push((a == b) as i64 as f64)
                },
                13 => { // Float add
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a + b)
                },
                14 => { // Float sub
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a - b)
                },
                15 => { // Float mul
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a * b)
                },
                16 => { // Float div
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a / b)
                },
                17 => { // Float negate
                    let a = self.stack.pop()?;
                    self.stack.push(-a)
                },
                18 => { // Float power
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a.powf(b))
                },
                19 => { // And
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(((a != 0.) && (a != 0.)) as i64 as f64);
                },
                20 => { // Or
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(((a != 0.) || (a != 0.)) as i64 as f64);
                },
                21 => { // Xor
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(((a != 0.) ^ (a != 0.)) as i64 as f64);
                },
                22 => { // not
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push((!(a != 0.)) as i64 as f64);
                },
                23 => { self.stack.remove(self.stack.len()-2); },// dupn
                24 => { self.stack.push(*self.stack.get(self.stack.len()-2)?); },// dupn

                25 => {// call
                    let n_args = u32::from_le_bytes([
                        self.instructions.instructions[self.ip+1],
                        self.instructions.instructions[self.ip+2],
                        self.instructions.instructions[self.ip+3],
                        self.instructions.instructions[self.ip+4],
                    ]) as usize;

                    for _ in 0..n_args {
                        self.calls.push(0.);
                    }
                    let last_index = self.calls.len() - 1;
                    for i in 0..n_args {
                        self.calls[last_index - i] = self.stack.pop()?;
                    }
                    self.ip += 4;
                },
                26 => {// tick
                    self.ip += 1;
                    break;
                }
                27 => {// Query interrupt
                    self.stack.push(self.interrupts.pop().unwrap_or(0.));
                }
                28 => {// Swap
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a);
                    self.stack.push(b);
                }
                _ => panic!("Unrecognized code")
            }
            self.ip += 1;
        }
        Some(())
    }
    
    pub fn reset(&mut self) {
        self.calls.clear();
        self.stack.clear();
        self.ip = 0;
    }
}