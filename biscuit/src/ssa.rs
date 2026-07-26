use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;

use crate::{Command, bytecode::FUNCTION_MAP_LOWER, compiler::Function, parser::SyntaxNode};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VariableType {
    Null,
    Float,
    List,
}

#[derive(Clone, Debug)]
pub enum Instruction {
    Argument,
    LiteralVector(Vec<u32>),
    LiteralFloat(f64),

    Call(u8, u32),
    LocalCall(String, u32),
    IfElse(usize, Option<usize>, u32),
    Loop(usize),

    Drp(u32),
    Ld(u32, u32), // Get index B f vector A
    St(u32, u32, u32), // Store C in index B of vector A 
    Stb(u32, u32), // Store C in back of vector A 
    
    Lt(u32, u32),
    Gt(u32, u32),
    Le(u32, u32), 
    Ge(u32, u32), 
    Eq(u32, u32),
    
    And(u32, u32),
    Or(u32, u32),
    Xor(u32, u32),
    Not(u32),

    Add(u32, u32),
    Sub(u32, u32),
    Mul(u32, u32),
    Div(u32, u32),
    Neg(u32),
    Pow(u32, u32),
}

pub struct Ssa {
    instructions: FxHashMap<u32, (Instruction, VariableType)>,
    declared_variables: FxHashMap<String, u32>,
    instruction_counter: u32,
    instruction_order: Vec<u32>,
    blocks: Vec<Ssa>,
}
impl Ssa {
    pub fn new() -> Self {
        Self {
            instructions: FxHashMap::default(),
            declared_variables: FxHashMap::default(),
            instruction_counter: 0,
            instruction_order: Vec::new(),
            blocks: Vec::new(),
        }
    }

    pub fn compile(lines: &[SyntaxNode], arguments: &FxHashMap<String, VariableType>, available_functions: &FxHashMap<String, Function>) -> Result<Self, String> {
        let mut ssa = Ssa::new();
        for (name, typ) in arguments.iter() {
            let var = ssa.push_instruction_typ(Instruction::Argument, *typ);
            ssa.declared_variables.insert(name.to_owned(), var);
        }
        for line in lines {
            ssa.process_node(line, available_functions)?;
        }
        ssa.order_instructions();
        Ok(ssa)
    }

    fn construct_arguments(&self) -> FxHashMap<String, VariableType> {
        let mut arguments = FxHashMap::default();
        for (k, v) in self.declared_variables.iter() {
            arguments.insert(k.to_owned(), self.instructions[v].1);
        }
        arguments
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
            let instruction = &self.instructions.get(&i).unwrap().0;
            // Use arguments
            match instruction {
                Instruction::Argument | Instruction::LiteralVector(_) | Instruction::LiteralFloat(_) | Instruction::Loop(_) => (),
                Instruction::IfElse(_, _, a) | Instruction::Drp(a) | Instruction::Not(a) | Instruction::Neg(a) |
                Instruction::Call(_, a) | Instruction::LocalCall(_, a)  => {
                    used.push(*a);
                }
                Instruction::Ld(a, b) | Instruction::Lt(a, b) | Instruction::Gt(a, b) |
                Instruction::Le(a, b) | Instruction::Ge(a, b) | Instruction::Eq(a, b) | Instruction::And(a, b) |
                Instruction::Or(a, b) | Instruction::Xor(a, b) | Instruction::Add(a, b) | Instruction::Sub(a, b) |
                Instruction::Mul(a, b) | Instruction::Div(a, b) | Instruction::Pow(a, b) | Instruction::Stb(a, b) => {
                    used.push(*a);
                    used.push(*b);
                },
                Instruction::St(a, b, c) => {
                    used.push(*a);
                    used.push(*b);
                    used.push(*c);
                },
            };
            match instruction {
                Instruction::Loop(_) | Instruction::IfElse(_, _, _) | Instruction::Drp(_) | Instruction::Call(_, _) | Instruction::LocalCall(_, _) | Instruction::St(_, _, _) | Instruction::Stb(_, _) => {used.push(i);},
                _ => ()
            };
        }
        self.instruction_order = used.to_vec();
    }

    /// Add a node to the SSA
    pub fn process_node(&mut self, node: &SyntaxNode, available_functions: &FxHashMap<String, Function>) -> Result<u32, String> {
        Ok(match node {
            // Control flow
            SyntaxNode::Block(header, body) => {
                let body_nodes = match &**body {
                    SyntaxNode::Adjacent(nodes) => nodes.clone(),
                    _ => vec![(&**body).clone()]
                };
                match &**header {
                    SyntaxNode::Unclassified(token) => {
                        // Else, loop
                        let s: &str = &token.get_inner();
                        match s {
                            "else" => {
                                let body_block = self.add_block(&body_nodes, available_functions)?;
                                let previous = &mut self.instructions.get_mut(&(self.instruction_counter-2)).unwrap().0;
                                match previous {
                                    Instruction::IfElse(_, else_block, _) => {
                                        if else_block.is_some() {
                                            return node.raise(&format!("Each if block can only have one else block"));
                                        }
                                        *else_block = Some(body_block);
                                    },
                                _ => {return node.raise(&format!("else blocks must be preceeded by if"));}
                                }
                            },
                            "loop" => {
                                let body_block = self.add_block(&body_nodes, available_functions)?;
                                self.push_instruction(Instruction::Loop(body_block));
                            },
                            _ => {return node.raise(&format!("Invalid keyword `{}`", token.get_inner()));}
                        }

                    },
                    SyntaxNode::Adjacent(nodes) => {
                        // If
                        let first: &str = match &nodes[0] {
                            SyntaxNode::Unclassified(text) => &text.get_inner(),
                            _ => {return node.raise("Invalid syntax 15");}
                        };
                        match first {
                            "if" => {
                                if nodes.len() != 2 {
                                    return node.raise("Invalid if statement");
                                }
                                let if_bool = self.process_node(&nodes[1], available_functions)?;
                                let body_block = self.add_block(&body_nodes, available_functions)?;
                                self.push_instruction(Instruction::IfElse(body_block, None, if_bool));
                            }
                            _ => {return node.raise(&format!("Invalid keyword `{}`", first));}
                        }
                    }
                    _ => {return node.raise("Invalid syntax 16");}
                };
                self.instruction_counter-1
            },
            SyntaxNode::Adjacent(nodes) => {
                // Keyword phrase or function call
                let first: &str = match &nodes[0] {
                    SyntaxNode::Unclassified(text) => &text.get_inner(),
                    _ => {return node.raise("Invalid syntax 17");}
                };
                match first {
                    "del" => {
                        let mut last_instruction = self.instruction_counter-1;
                        for item in &nodes[1..] {
                            let name = match &item {
                                SyntaxNode::Unclassified(token) => &token.get_inner(),
                                _ => {return node.raise("Only variables can be deleted");},
                            };
                            let var = self.check_var(name, VariableType::List).ok_or(item.raise_str(&format!("`{}` is not a valid list", name)))?;
                            last_instruction = self.push_instruction(Instruction::Drp(var));
                        }
                        last_instruction
                    },
                    _ => {
                        // Function call
                        if nodes.len() != 2 {
                            return node.raise("Invalid syntax 18");
                        }
                        let arguments = match &nodes[1] {
                            SyntaxNode::Parenthesis("(", token) => token,
                            _ => {return node.raise("Invalid function call 1");}
                        };
                        let arguments: &[SyntaxNode] = match &**arguments {
                            SyntaxNode::List(",", nodes) => {
                                match &**nodes {
                                    SyntaxNode::Adjacent(tokens) => tokens,
                                    _ => {return node.raise("Invalid function call 4");}
                                }
                            },
                            SyntaxNode::Adjacent(tokens) => if tokens.is_empty() {
                                // There were no arguments
                                &[]
                            } else {
                                return node.raise("Invalid function call 2");
                            },
                            _ => {
                                // There was a single argument
                                std::slice::from_ref(arguments)
                            }
                        };
                        let mut arg_v = self.push_instruction(Instruction::LiteralVector(Vec::new()));
                        for argument in arguments {
                            let node = self.process_node(argument, available_functions)?;
                            arg_v = self.push_instruction(Instruction::Stb(arg_v, node));
                        }
                        
                        match FUNCTION_MAP_LOWER.get(first) {
                            Some(f) => {
                                let typ = f.return_type();
                                self.push_instruction_typ(Instruction::Call(*f as u8, arg_v), typ)
                            },
                            None => match available_functions.get(first) {
                                Some(f) => {
                                    self.push_instruction_typ(Instruction::LocalCall(f.name.clone(), arg_v), f.return_value)
                                },
                                None => {return node.raise(&format!("Unrecognized function {}", first));},
                            }
                        }
                    }
                }
            },
            SyntaxNode::Unclassified(token) => {
                match self.declared_variables.get(token.get_inner()) {
                    Some(n) => *n,
                    None => {return node.raise(&format!("Undeclared variable {}", token.get_inner()));},
                }
            }
            // Usually some kind of assignment
            SyntaxNode::Binop(op, a, b) => {
                let b_var = self.process_node(b, available_functions)?;
                let a_name = match &**a {
                    SyntaxNode::Unclassified(token) => Some(token.get_inner()),
                    _ => None
                };
                match *op {
                    // Handle equal sign
                    "=" => {
                        let a_name = a_name.ok_or(a.raise_str("Invalid syntax 19"))?;
                        self.declared_variables.insert(a_name.clone(), b_var);
                        b_var
                    },
                    // Handle assignment operators
                    "+=" | "-=" | "*=" | "/=" => {
                        let a_name = a_name.ok_or(a.raise_str("Invalid syntax 20"))?;
                        let a_var = self.process_node(b, available_functions)?;
                        let result = match *op {
                            "+=" => self.push_instruction(Instruction::Add(a_var, b_var)),
                            "-=" => self.push_instruction(Instruction::Add(a_var, b_var)),
                            "*=" => self.push_instruction(Instruction::Add(a_var, b_var)),
                            "/=" => self.push_instruction(Instruction::Add(a_var, b_var)), 
                            _ => unreachable!()
                        };
                        *self.declared_variables.get_mut(a_name).unwrap() = result;
                        result
                    },
                    _ => {
                        // Handle not assignment operators
                        let a_var = self.process_node(a, available_functions)?;
                        match *op {
                            ">" => self.push_instruction(Instruction::Gt(a_var, b_var)),
                            "<" => self.push_instruction(Instruction::Lt(a_var, b_var)),
                            ">=" => self.push_instruction(Instruction::Ge(a_var, b_var)),
                            "<=" => self.push_instruction(Instruction::Le(a_var, b_var)),
                            "==" => self.push_instruction(Instruction::Eq(a_var, b_var)),
                            "&&" => self.push_instruction(Instruction::And(a_var, b_var)),
                            "||" => self.push_instruction(Instruction::Or(a_var, b_var)),
                            "^" => self.push_instruction(Instruction::Xor(a_var, b_var)),
                            "+" => self.push_instruction(Instruction::Add(a_var, b_var)),
                            "-" => self.push_instruction(Instruction::Sub(a_var, b_var)),
                            "*" => self.push_instruction(Instruction::Mul(a_var, b_var)),
                            "/" => self.push_instruction(Instruction::Div(a_var, b_var)),
                            "**" => self.push_instruction(Instruction::Pow(a_var, b_var)),
                            _ => {return node.raise(&format!("Unrecognized binary operation {}", op));},
                        }
                    }
                }
            },
            // Usually some kind of assignment
            SyntaxNode::Unop(op, a) => {
                let a_var = self.process_node(a, available_functions)?;
                match *op {
                    "!" => self.push_instruction(Instruction::Not(a_var)),
                    "-" => self.push_instruction(Instruction::Neg(a_var)),
                    _ => {return a.raise(&format!("Unrecognized unary operation {}", op));},
                }
            },
            // Usually some kind of assignment
            SyntaxNode::Number(t) => {
                self.push_instruction(Instruction::LiteralFloat(*t.get_inner()))
            },
            _ => {
                return node.raise("Invalid syntax 21");
            }
        })
    }

    fn check_var(&mut self, name: &str, typ: VariableType) -> Option<u32> {
        match self.declared_variables.get(name) {
            Some(v) => {
                if typ == self.instructions[v].1 {
                    Some(*v)
                } else {
                    None
                }
            },
            None => None,
        }
    }

    pub fn push_instruction(&mut self, instruction: Instruction) -> u32 {
        let typ = match &instruction {
            Instruction::LiteralVector(_) | Instruction::St(_, _, _) | Instruction::Stb(_, _) => VariableType::List,
            Instruction::Ld(_, _) | Instruction::LiteralFloat(_) | Instruction::Lt(_, _) | Instruction::Gt(_, _) | 
            Instruction::Le(_, _) |  Instruction::Ge(_, _) | Instruction::Eq(_, _) | Instruction::And(_, _) |
            Instruction::Or(_, _) | Instruction::Xor(_, _) | Instruction::Not(_) | Instruction::Add(_, _) | 
            Instruction::Sub(_, _) | Instruction::Mul(_, _) | Instruction::Div(_, _) | Instruction::Neg(_) | 
            Instruction::Pow(_, _) => VariableType::Float,
            Instruction::Loop(_) | Instruction::Drp(_) | Instruction::LocalCall(_, _) |
            Instruction::IfElse(_, _, _) => VariableType::Null,
            Instruction::Argument | Instruction::Call(_, _) => unreachable!(),
        };
        self.push_instruction_typ(instruction, typ)
    }

    pub fn push_instruction_typ(&mut self, instruction: Instruction, typ: VariableType) -> u32 {
        self.instructions.insert(self.instruction_counter, (instruction, typ));
        self.instruction_counter += 1;
        self.instruction_counter - 1
    }

    fn add_block(&mut self, lines: &[SyntaxNode], available_functions: &FxHashMap<String, Function>) -> Result<usize, String> {
        let ssa = Self::compile(lines, &self.construct_arguments(), available_functions)?;
        self.blocks.push(ssa);
        Ok(self.blocks.len() - 1)
    }

    pub fn get_used_functions(&self) -> Vec<String> {
        let mut funcs = SortedSet::new();
        for command in self.instructions.values() {
            match &command.0 {
                Instruction::LocalCall(name, _) => {funcs.push(name.clone());},
                _ => ()
            };
        }
        funcs.to_vec()
    }
}
impl std::fmt::Debug for Ssa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Ssa (")?;
        let indices = SortedSet::from_unsorted(self.instructions.keys().collect());
        for index in indices {
            writeln!(f, "    {:04} {:?}", index, self.instructions[index].0)?;
        }
        write!(f, ")")
    }
}

// =================================
// Stack manipulation functions
// =================================

/// Move the float "op" to the top of the stack if it is not already there
fn topify(op: u32, bytecode: &mut Vec<u8>, running_stack: &mut Vec<u32>) {
    if running_stack.last() == Some(&op) {return;}
    bytecode.push(Command::Push as u8);
    let pos = running_stack.iter().position(|i| *i == op).unwrap() as f64;
    bytecode.extend(&pos.to_le_bytes());
    bytecode.push(Command::Pick as u8);
    running_stack.push(op);
}

/// Move floats "op1" and "op2" to the top of the stack so that a binary operation can be run. Commutative
fn binopify_comm(op1: u32, op2: u32, bytecode: &mut Vec<u8>, running_stack: &mut Vec<u32>) {
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

/// Move floats "op1" and "op2" to the top of the stack so that a binary operation can be run. Not commutative
fn binopify(op1: u32, op2: u32, bytecode: &mut Vec<u8>, running_stack: &mut Vec<u32>) {
    let top = running_stack.last().copied();
    let second = running_stack.iter().nth_back(1).copied();
    if top == Some(op1) && second == Some(op2) {
    } else if top == Some(op2) && second == Some(op1) {
        bytecode.push(Command::Swp as u8);
    } else if top == Some(op2) {
        topify(op1, bytecode, running_stack);
    } else if top == Some(op1) {
        topify(op2, bytecode, running_stack);
        bytecode.push(Command::Swp as u8);
    } else {
        topify(op2, bytecode, running_stack);
        topify(op1, bytecode, running_stack);
    }
}

pub struct Bytecode {
    bytecode: Vec<u8>,
    functions: Vec<(usize, String)>,

}
impl Bytecode {
    pub fn new(ssa: &Ssa, program_start: usize) -> Self {
        let mut bytecode = Vec::new();
        let mut functions = Vec::new();
        let last_used = ssa.get_last_used();

        let mut running_stack = Vec::new();

        for (i, op) in ssa.instruction_order.iter().enumerate() {
            while match last_used.get(op) {
                None => true,
                Some(last_used_index) => *last_used_index < i,
            } {
                // Pop all unused variables
                if let Some(i) = running_stack.pop() {
                    match ssa.instructions[&i].1 {
                        VariableType::Float | VariableType::List => {
                            bytecode.push(Command::Pop as u8);
                        },
                        VariableType::Null => {}
                    }
                }
            }
            match &ssa.instructions[op].0 {
                Instruction::Argument => running_stack.push(*op),
                Instruction::LiteralVector(items) => {
                    bytecode.push(Command::Alc as u8);
                    for item in items {
                        bytecode.push(Command::Push as u8);
                        bytecode.extend(&item.to_le_bytes());
                        bytecode.push(Command::Push as u8);
                        bytecode.extend(&2f64.to_le_bytes());
                        bytecode.push(Command::Pick as u8);
                        bytecode.push(Command::Stb as u8);
                    }
                    running_stack.push(*op);
                },
                Instruction::LiteralFloat(f) => {
                    bytecode.push(Command::Push as u8);
                    bytecode.extend(&f.to_le_bytes());
                    running_stack.push(*op);
                },
                Instruction::Add(op1, op2) => {
                    binopify_comm(*op1, *op2, &mut bytecode, &mut running_stack);
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
                    binopify_comm(*op1, *op2, &mut bytecode, &mut running_stack);
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
                Instruction::Lt(op1, op2) => {
                    binopify(*op1, *op2, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Lt as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Gt(op1, op2) => {
                    binopify(*op1, *op2, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Gt as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Le(op1, op2) => {
                    binopify(*op1, *op2, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Le as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Ge(op1, op2) => {
                    binopify(*op1, *op2, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Ge as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Eq(op1, op2) => {
                    binopify(*op1, *op2, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Eq as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::And(op1, op2) => {
                    binopify(*op1, *op2, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::And as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Or(op1, op2) => {
                    binopify(*op1, *op2, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Or as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Xor(op1, op2) => {
                    binopify(*op1, *op2, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Xor as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Not(a) => {
                    topify(*a, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Not as u8);
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Neg(a) => {
                    topify(*a, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Not as u8);
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Drp(adr) => {
                    topify(*adr, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Drp as u8);
                    running_stack.pop();
                },
                Instruction::Ld(adr, idx) => {
                    binopify(*adr, *idx, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Ld as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::St(adr, idx, val) => {
                    topify(*val, &mut bytecode, &mut running_stack);
                    topify(*idx, &mut bytecode, &mut running_stack);
                    topify(*adr, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::St as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Stb(adr, val) => {
                    topify(*val, &mut bytecode, &mut running_stack);
                    topify(*adr, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Stb as u8);
                    running_stack.pop();
                    running_stack.pop();
                    running_stack.push(*op);
                },
                Instruction::Call(func, arg) => {
                    topify(*arg, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Call as u8);
                    bytecode.push(*func);
                    running_stack.pop();
                },
                Instruction::LocalCall(name, args) => {
                    topify(*args, &mut bytecode, &mut running_stack);
                    functions.push((bytecode.len() + program_start, name.to_owned()));
                    running_stack.pop();
                },
                Instruction::IfElse(plus, minus, cond) => {
                    topify(*cond, &mut bytecode, &mut running_stack);
                    bytecode.push(Command::Jnz as u8);
                    let start = bytecode.len() + program_start;
                    for _ in 0..8 { bytecode.push(0); }
                    running_stack.pop();

                    // Parse plus
                    let mut block = Bytecode::new(&ssa.blocks[*plus], bytecode.len());
                    bytecode.append(&mut block.bytecode);
                    functions.append(&mut block.functions);

                    let end = bytecode.len()+1 + program_start;
                    if minus.is_some() {
                        bytecode.push(Command::Jmp as u8);
                        for _ in 0..8 { bytecode.push(0); }
                    }
                    for (i, b) in u64::to_le_bytes((bytecode.len() + program_start) as u64).iter().enumerate() {
                        bytecode[start + i] = *b;
                    }
                    if let Some(minus) = minus {
                        // Parse minus
                        let mut block = Bytecode::new(&ssa.blocks[*minus], bytecode.len());
                        bytecode.append(&mut block.bytecode);
                        functions.append(&mut block.functions);
                        for (i, b) in u64::to_le_bytes((bytecode.len() + program_start) as u64).iter().enumerate() {
                            bytecode[end + i] = *b;
                        }
                    }
                },
                Instruction::Loop(label) => {
                    let start = (bytecode.len() + program_start) as u64;
                    bytecode.push(Command::Jmp as u8);

                    // Parse label
                    let mut block = Bytecode::new(&ssa.blocks[*label], bytecode.len());
                    bytecode.append(&mut block.bytecode);
                    functions.append(&mut block.functions);

                    bytecode.extend(&u64::to_le_bytes(start));
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