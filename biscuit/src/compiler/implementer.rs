
// =================================
// Stack manipulation functions
// =================================

use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;

use crate::{Command, bytecode::VariableType, compiler::ssa::Location};

use super::ssa::{Ssa, Instruction};

fn set_state(ops: &[Location], bytecode: &mut Vec<u8>, running_stack: &mut Vec<Location>) {
    for op in ops.iter().rev() {
        let top = running_stack.last();
        if top == Some(op) {
            bytecode.push(Command::Dup as u8);
            running_stack.push(*op);
        } else {
            let pos = running_stack.iter().position(|i| i == op).unwrap() as f64;
            bytecode.push(Command::Push as u8);
            bytecode.extend(&pos.to_le_bytes());
            bytecode.push(Command::Pick as u8);
            running_stack.push(*op);
        }
    }
    // OPTIMIZE
}

pub struct Bytecode<'a> {
    ssa: &'a Ssa,
    bytecode: Vec<u8>,
    running_stack: Vec<Location>,
    functions: Vec<(usize, String)>,
    last_used: FxHashMap<Location, usize>,
}
impl<'a> Bytecode<'a> {
    pub fn new(ssa: &'a Ssa) -> Self {
        let mut bytecode = Self {
            ssa,
            bytecode: Vec::new(),
            running_stack: Vec::new(),
            functions: Vec::new(),
            last_used: ssa.get_last_used(),
        };
        for (instruction_index, op) in ssa.instruction_order.iter().enumerate() {
            bytecode.pop_unused(instruction_index);
            bytecode.write_bytecode(*op);
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
        todo!()
    }

    /// Write an operation to bytecode
    fn write_bytecode(&mut self, op: u32) {
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
                set_state(&[*op1, *op2], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Add as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Pow(op1, op2) => {
                set_state(&[*op1, *op2], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Pow as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Div(op1, op2) => {
                set_state(&[*op1, *op2], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Div as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Mul(op1, op2) => {
                set_state(&[*op1, *op2], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Mul as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Sub(op1, op2) => {
                set_state(&[*op1, *op2], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Sub as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Lt(op1, op2) => {
                set_state(&[*op1, *op2], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Lt as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Gt(op1, op2) => {
                set_state(&[*op1, *op2], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Gt as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Le(op1, op2) => {
                set_state(&[*op1, *op2], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Le as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Ge(op1, op2) => {
                set_state(&[*op1, *op2], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Ge as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Eq(op1, op2) => {
                set_state(&[*op1, *op2], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Eq as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::And(op1, op2) => {
                set_state(&[*op1, *op2], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::And as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Or(op1, op2) => {
                set_state(&[*op1, *op2], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Or as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Xor(op1, op2) => {
                set_state(&[*op1, *op2], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Xor as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Not(a) => {
                set_state(&[*a], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Not as u8);
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Neg(a) => {
                set_state(&[*a], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Not as u8);
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Drp(adr) => {
                set_state(&[*adr], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Drp as u8);
                self.running_stack.pop();
            },
            Instruction::Ld(adr, idx) => {
                set_state(&[*adr, *idx], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Ld as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::St(adr, idx, val) => {
                set_state(&[*val], &mut self.bytecode, &mut self.running_stack);
                set_state(&[*idx], &mut self.bytecode, &mut self.running_stack);
                set_state(&[*adr], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::St as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Stb(adr, val) => {
                set_state(&[*val], &mut self.bytecode, &mut self.running_stack);
                set_state(&[*adr], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Stb as u8);
                self.running_stack.pop();
                self.running_stack.pop();
                self.running_stack.push(Location::internal(op));
            },
            Instruction::Call(func, arg) => {
                set_state(&[*arg], &mut self.bytecode, &mut self.running_stack);
                self.bytecode.push(Command::Call as u8);
                self.bytecode.push(*func);
                self.running_stack.pop();
            },
            Instruction::LocalCall(name, args) => {
                set_state(&[*args], &mut self.bytecode, &mut self.running_stack);
                self.functions.push((self.bytecode.len(), name.to_owned()));
                self.running_stack.pop();
            },
            Instruction::Theta(_, _) => todo!(),
            Instruction::Action(_) => todo!(),
        }
    }

    /// Dump all scope not relevant for the return statement
    fn dump_scope(&mut self) {
        while self.running_stack.len() > self.ssa.return_variables.len() {
            if self.ssa.return_variables.contains(self.running_stack.last().unwrap()) {
                // Roll
            } else {
                self.bytecode.push(Command::Pop as u8);
                self.running_stack.pop();
            }
        }
    }

    fn embed_bytecode(&mut self, mut bytecode: Bytecode) {
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
}