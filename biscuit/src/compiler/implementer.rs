
// =================================
// Stack manipulation functions
// =================================

use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;

use crate::{Command, compiler::ssa::{Branch, Location}};

use super::ssa::{Ssa, Instruction};


pub struct Bytecode<'a> {
    ssa: &'a Ssa,
    bytecode: Vec<u8>,
    running_stack: Vec<Location>,
    functions: Vec<(usize, String)>,
    last_used: FxHashMap<Location, usize>,
    written_branches: SortedSet<usize>,
}
impl<'a> Bytecode<'a> {
    pub fn new(ssa: &'a Ssa) -> Self {
        let mut bytecode = Self {
            ssa,
            bytecode: Vec::new(),
            running_stack: Vec::new(),
            functions: Vec::new(),
            last_used: ssa.get_last_used(),
            written_branches: SortedSet::new(),
        };
        dbg!(&bytecode.last_used);
        for (instruction_index, op) in ssa.instruction_order.iter().enumerate() {
            bytecode.pop_unused(instruction_index);
            bytecode.write_bytecode(*op, instruction_index);
        }
        bytecode.dump_scope();
        bytecode
    }

    pub fn len(&self) -> usize {
        self.bytecode.len()
    }
    pub fn code(&self) -> &[u8] {
        &self.bytecode
    }

    /// Pop all the unused items in the stack until a used item is at top
    fn pop_unused(&mut self, instruction_index: usize) {
        loop {
            let top_instruction = match self.running_stack.last() {
                Some(loc) => if loc.tier == 0 { loc.index } else { return },
                None => return
            };
            match self.last_used.get(&Location::internal(top_instruction)) {
                Some(index) => if *index >= instruction_index { return },
                None => ()
            };
            self.bytecode.push(Command::Pop as u8);
            self.running_stack.pop();
        }
    }

    /// Write an operation to bytecode
    fn write_bytecode(&mut self, op: u32, instruction_index: usize) {
        let instruction = &self.ssa.instructions[&op];
        match instruction {
            Instruction::Argument => self.running_stack.push(Location::internal(op)),
            Instruction::LiteralVector(items) => {
                self.bytecode.push(Command::Alc as u8);
                for item in items {
                    self.bytecode.push(Command::Push as u8);
                    self.bytecode.extend(&item.to_le_bytes());
                    self.bytecode.push(Command::Stb as u8);
                }
                self.running_stack.push(Location::internal(op));
            },
            Instruction::LiteralFloat(f) => {
                self.bytecode.push(Command::Push as u8);
                self.bytecode.extend(&f.to_le_bytes());
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Add(op1, op2) => {
                self.set_state(&[*op1, *op2]);
                self.bytecode.push(Command::Add as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Pow(op1, op2) => {
                self.set_state(&[*op1, *op2]);
                self.bytecode.push(Command::Pow as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Div(op1, op2) => {
                self.set_state(&[*op1, *op2]);
                self.bytecode.push(Command::Div as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Mul(op1, op2) => {
                self.set_state(&[*op1, *op2]);
                self.bytecode.push(Command::Mul as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Sub(op1, op2) => {
                self.set_state(&[*op1, *op2]);
                self.bytecode.push(Command::Sub as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Lt(op1, op2) => {
                self.set_state(&[*op1, *op2]);
                self.bytecode.push(Command::Lt as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Gt(op1, op2) => {
                self.set_state(&[*op1, *op2]);
                self.bytecode.push(Command::Gt as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Le(op1, op2) => {
                self.set_state(&[*op1, *op2]);
                self.bytecode.push(Command::Le as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Ge(op1, op2) => {
                self.set_state(&[*op1, *op2]);
                self.bytecode.push(Command::Ge as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Eq(op1, op2) => {
                self.set_state(&[*op1, *op2]);
                self.bytecode.push(Command::Eq as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::And(op1, op2) => {
                self.set_state(&[*op1, *op2]);
                self.bytecode.push(Command::And as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Or(op1, op2) => {
                self.set_state(&[*op1, *op2]);
                self.bytecode.push(Command::Or as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Xor(op1, op2) => {
                self.set_state(&[*op1, *op2]);
                self.bytecode.push(Command::Xor as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Not(a) => {
                self.set_state(&[*a]);
                self.bytecode.push(Command::Not as u8);
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Neg(a) => {
                self.set_state(&[*a]);
                self.bytecode.push(Command::Not as u8);
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Ld(adr, idx) => {
                self.set_state(&[*adr, *idx]);
                self.bytecode.push(Command::Ld as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::St(adr, idx, val) => {
                self.set_state(&[*val]);
                self.set_state(&[*idx]);
                self.set_state(&[*adr]);
                self.bytecode.push(Command::St as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Stb(adr, val) => {
                self.set_state(&[*val]);
                self.set_state(&[*adr]);
                self.bytecode.push(Command::Stb as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Call(func, arg) => {
                self.set_state(&[*arg]);
                self.bytecode.push(Command::Call as u8);
                self.bytecode.push(*func);
                self.running_stack.pop();
            },
            Instruction::LocalCall(name, args) => {
                self.set_state(&[*args]);
                self.functions.push((self.bytecode.len(), name.to_owned()));
                self.running_stack.pop();
            },
            Instruction::Theta(branch, _) | Instruction::Action(branch) => if !self.written_branches.contains(branch) {
                match &self.ssa.branches[*branch] {
                    Branch::If(items, ssa) => {
                        // Determine the final layout
                        let mut theta_names = Vec::new();
                        for loc in &self.ssa.instruction_order[instruction_index..] {
                            if let Instruction::Theta(b, n) = &self.ssa.instructions[loc] {
                                if b == branch {
                                    theta_names.push(n.clone());
                                }
                            }
                        }

                        let mut terminal_locations = Vec::new();

                        // Embed the branch code, with branching logic
                        for (ssa, if_statement) in items {
                            let branch = self.compile_branch(&if_statement);
                            self.embed_branch(branch);

                            self.bytecode.push(Command::Jnz as u8);
                            let start = self.bytecode.len();
                            self.bytecode.extend(0u64.to_le_bytes());
                            
                            // Roll to match scope
                            let mut branch = self.compile_branch(&ssa);
                            branch.roll_scope_names(&theta_names);
                            self.embed_branch(branch);
                            
                            self.bytecode.push(Command::Jmp as u8);
                            terminal_locations.push(self.bytecode.len());
                            self.bytecode.extend(0u64.to_le_bytes());
                            self.bytecode.splice(start..start+8, (self.bytecode.len() as u64).to_le_bytes());
                        }
                        if let Some(ssa) = ssa {
                            // Else
                            let mut branch = self.compile_branch(&ssa);
                            branch.roll_scope_names(&theta_names);
                            self.embed_branch(branch);
                        }

                        for loc in terminal_locations {
                            self.bytecode.splice(loc..loc+8, (self.bytecode.len() as u64).to_le_bytes());
                        }
                    },
                    Branch::Loop(ssa) => {
                        // Embed the branch code, with branching logic
                        let start = self.bytecode.len() as u64;
                        let branch = self.compile_branch(&ssa);
                        self.embed_branch(branch);
                        self.bytecode.push(Command::Jmp as u8);
                        self.bytecode.extend(start.to_le_bytes());
                    },
                }
                self.written_branches.push(*branch);
            },
        }
    }

    /// Dump all scope not relevant for the return statement
    fn dump_scope(&mut self) {
        while self.running_stack.len() > self.ssa.return_variables.len() {
            if self.ssa.return_variables.contains(self.running_stack.last().unwrap()) {
                // Roll
                todo!()
            } else {
                self.bytecode.push(Command::Pop as u8);
                self.running_stack.pop();
            }
        }
    }

    fn compile_branch(&self, ssa: &'a Ssa) -> Bytecode<'a> {
        // Compile the bytecode
        let mut bytecode = Self {
            ssa,
            bytecode: Vec::new(),
            running_stack: self.running_stack.iter().map(|l| l.graduate()).collect(),
            functions: Vec::new(),
            last_used: ssa.get_last_used(),
            written_branches: SortedSet::new(),
        };
        for (instruction_index, op) in ssa.instruction_order.iter().enumerate() {
            bytecode.pop_unused(instruction_index);
            bytecode.write_bytecode(*op, instruction_index);
        }
        bytecode.dump_scope();
        bytecode
    }

    fn roll_scope_names(&mut self, variable_names: &[String]) {
        // Add code to set final state to the desired value
        let mut final_state = Vec::new();
        for name in variable_names {
            match self.ssa.declared_variables.get(name) {
                Some(loc) => final_state.push(*loc),
                None => unreachable!()
            }
        };
        self.roll_state(&final_state);
    }

    fn embed_branch(&mut self, mut bytecode: Bytecode) {
        let offset = self.bytecode.len();
        self.bytecode.append(&mut bytecode.bytecode);
        for (call_pos, name) in bytecode.functions {
            self.functions.push((call_pos + offset, name));
        }
        self.running_stack.append(&mut bytecode.running_stack);
    }

    pub fn replace_calls(&mut self, locations: &FxHashMap<String, usize>) {
        for (pos, name) in &self.functions {
            let location_bytes = (locations[name] as u32).to_le_bytes();
            self.bytecode[pos + 0] = location_bytes[0];
            self.bytecode[pos + 1] = location_bytes[1];
            self.bytecode[pos + 2] = location_bytes[2];
            self.bytecode[pos + 3] = location_bytes[3];
        }
    }

    fn set_state(&mut self, target: &[Location]) {
        for op in target.iter().rev() {
            let top = self.running_stack.last();
            if top == Some(op) {
                self.bytecode.push(Command::Dup as u8);
                self.running_stack.push(*op);
            } else {
                let pos = self.running_stack.iter().position(|i| i == op).unwrap() as f64;
                self.bytecode.push(Command::Push as u8);
                self.bytecode.extend(&pos.to_le_bytes());
                self.bytecode.push(Command::Pick as u8);
                self.running_stack.push(*op);
            }
        }
        // OPTIMIZE
    }

    pub fn roll(&mut self, n: usize, shift: usize) {
        let length = self.running_stack.len();
        if shift == 0 {return;}
        if n == 1 {return;}
        if n == 2 {
            self.bytecode.push(Command::Swp as u8);
            self.running_stack.swap(length-1, length-2);
            return;
        }
        if shift < n/2 {
            for _ in 0..shift {
                self.bytecode.push(Command::Push as u8);
                self.bytecode.extend((n as f64).to_le_bytes());
                self.bytecode.push(Command::Roll as u8);
            }
            self.running_stack[length-n..].rotate_right(shift);
        } else {
            for _ in 0..(n - shift) {
                self.bytecode.push(Command::Push as u8);
                self.bytecode.extend((n as f64).to_le_bytes());
                self.bytecode.push(Command::Rolr as u8);
            }
            self.running_stack[length-n..].rotate_left(n - shift);
        }
    }

    fn roll_state(&mut self, target: &[Location]) {
        // Roll until the target is matched, building up from the back
        for (n_chars, var) in target.iter().enumerate().rev() {
            let stack_start = self.running_stack.len()-n_chars;
            let pos = self.running_stack[stack_start..].iter().position(|v| v==var).unwrap();
            let dist_from_n = self.running_stack.len() - pos - 1;
            let shift = (dist_from_n + 1) % n_chars;
            self.roll(n_chars, shift);
            dbg!(self.running_stack.last().unwrap());
            dbg!(var);
        }
    }
}