use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;

use crate::{Command, compiler::Function, parser::SyntaxNode};

#[derive(Clone, Copy, Debug)]
pub enum VariableType {
    Null,
    Float,
    List,
}

#[derive(Clone, Debug)]
pub enum Instruction {
    Argument,
    Add(u32, u32),
    Sub(u32, u32),
    Mul(u32, u32),
    Div(u32, u32),
    Pow(u32, u32),
    /// List, index
    ListLoad(u32, u32),
    /// List, item, index
    ListStore(u32, u32, u32),
    /// List
    ListAppend(u32),
    ListLiteral(Vec<u32>),
}

pub struct Ssa {
    instructions: FxHashMap<u32, (Instruction, VariableType)>,
    declared_variables: FxHashMap<String, u32>,
    instruction_counter: u32,
    functions_used: Vec<String>,
    instruction_order: Vec<u32>,
}
impl Ssa {
    pub fn new() -> Self {
        Self {
            instructions: FxHashMap::default(),
            functions_used: Vec::new(),
            declared_variables: FxHashMap::default(),
            instruction_counter: 0,
            instruction_order: Vec::new(),
        }
    }

    pub fn get_last_used(&self) -> FxHashMap<u32, usize> {
        let mut last_usage = FxHashMap::default();
        for (i, op) in self.instruction_order.iter().enumerate() {
            match last_usage.get_mut(op) {
                Some(index) => {*index = i;},
                None => {last_usage.insert(*op, i);},
            }
        }
        last_usage
    }

    pub fn order_instructions(&mut self) {
        let mut used = SortedSet::new();
        for i in 0..self.instruction_counter {
            let instructions = &self.instructions.get(&i).unwrap().0;
            match instructions {
                Instruction::Argument | Instruction::ListLiteral(_) => (),
                Instruction::ListAppend(i) => { used.push(*i); },
                Instruction::Add(i, j) | Instruction::Sub(i, j) | Instruction::Mul(i, j) | Instruction::Div(i, j) | Instruction::Pow(i, j) | Instruction::ListLoad(i, j) => { used.push(*i); used.push(*j); },
                Instruction::ListStore(i, j, k) => { used.push(*i); used.push(*j);  used.push(*k); },
            };
        }
        self.instruction_order = used.to_vec();
    }

    pub fn add_arg(&mut self, name: &str, typ: VariableType) {
        self.declared_variables.insert(name.to_owned(), self.instruction_counter);
        self.push_instruction(Instruction::Argument, typ);
        
    }

    pub fn process_node(&mut self, node: &SyntaxNode, available_functions: &FxHashMap<String, Function>, used_functions: &mut Vec<String>) {
        // TODO
    }

    fn check_var(&mut self, name: &str, typ: VariableType) -> Option<(u32, VariableType)> {
        match self.declared_variables.get(name) {
            Some(v) => Some((*v, self.instructions[v].1)),
            None => None,
        }
    }

    fn push_instruction(&mut self, instruction: Instruction, typ: VariableType) {
        self.instructions.insert(self.instruction_counter, (instruction, typ));
        self.instruction_counter += 1;
    }
    fn add_fun(&mut self, name: &str) {
        self.functions_used.push(name.to_owned());
    }
}

// =================================
// Stack manipulation functions
// =================================

/// Move the float "op" to the top of the stack if it is not already there
fn topify(op: u32, bytecode: &mut Vec<u8>, running_stack: &mut Vec<u32>) {
    if running_stack.last() == Some(&op) {return;}
    let pos = running_stack.iter().position(|i| *i == op).unwrap() as u32;
    let pos_bytes = pos.to_le_bytes();
    bytecode.push(Command::Push as u8);
    bytecode.push(pos_bytes[0]);
    bytecode.push(pos_bytes[1]);
    bytecode.push(pos_bytes[2]);
    bytecode.push(pos_bytes[3]);
    bytecode.push(Command::Pick as u8);
    running_stack.push(op);
}

/// Move floats "op1" and "op2" to the top of the stack so that a binary operation can be run
fn binopify(op1: u32, op2: u32, bytecode: &mut Vec<u8>, running_stack: &mut Vec<u32>) {
        let top = running_stack.last().copied();
        let second = running_stack.iter().nth_back(1).copied();
        if top == Some(op1) {
            if second != Some(op2) { topify(op2, bytecode, running_stack); }
        } else if top == Some(op2) {
            if second != Some(op1) { topify(op1, bytecode, running_stack); }
        } else {
            topify(op1, bytecode, running_stack);
            topify(op2, bytecode, running_stack);
        }
        if op1 == op2 {
            bytecode.push(Command::Dup as u8);
        }
}

pub struct Bytecode {
    bytecode: Vec<u8>,
    functions: Vec<(usize, String)>,

}
impl Bytecode {
    pub fn new(ssa: &Ssa) -> Self {
        let mut bytecode = Vec::new();
        let mut functions = Vec::new();
        let last_used = ssa.get_last_used();

        let mut running_stack = Vec::new();

        for (i, op) in ssa.instruction_order.iter().enumerate() {
            while match last_used.get(op) {
                None => true,
                Some(last_used_index) => *last_used_index < i,
            } {
                // Pop the top of the stack
                if let Some(i) = running_stack.pop() {
                    match ssa.instructions[&i].1 {
                        VariableType::Null | VariableType::Float => {
                            // Pop a float
                            bytecode.push(Command::Pop as u8);
                        },
                        VariableType::List => {
                            // TODO pop a list
                        },
                    }
                }
                
            }
            match &ssa.instructions[op].0 {
                Instruction::Argument => running_stack.push(*op),
                Instruction::ListLiteral(items) => {
                    // TODO push a list
                },
                Instruction::Add(op1, op2) => {
                    binopify(*op1, *op2, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Add as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Pow(op1, op2) => {
                    binopify(*op1, *op2, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Pow as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Div(op1, op2) => {
                    binopify(*op1, *op2, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Div as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Mul(op1, op2) => {
                    binopify(*op1, *op2, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Mul as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Sub(op1, op2) => {
                    binopify(*op1, *op2, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Sub as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
            }
        }
        
        Self {
            bytecode,
            functions,
        }
    }

    pub fn len(&self) -> usize {
        self.bytecode.len()
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

    pub fn code(&self) -> &[u8] {
        &self.bytecode
    }
}