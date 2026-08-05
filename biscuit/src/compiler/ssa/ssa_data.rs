use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;

use crate::{bytecode::{FUNCTION_MAP_LOWER, VariableType}, compiler::{Function, ssa::{Branch, Instruction, Location, Ssa}}, parser::SyntaxNode};


#[derive(Clone)]
pub(crate) struct SsaData {
    pub instructions: FxHashMap<u32, Instruction>,
    pub types: FxHashMap<Location, VariableType>,
    pub declared_variables: FxHashMap<String, Location>,
    instruction_counter: u32,
    pub return_variables: Vec<Location>,
    pub branches: Vec<Branch>, // The hash map maps from the local block variable to the main block variable
}
impl SsaData {
    pub fn new(node: &SyntaxNode, arguments: &FxHashMap<String, VariableType>, available_functions: &FxHashMap<String, Function>) -> Result<Self, String> {
        let mut data = Self {
            instructions: FxHashMap::default(),
            types: FxHashMap::default(),
            declared_variables: FxHashMap::default(),
            instruction_counter: 0,
            return_variables: Vec::new(),
            branches: Vec::new(),
        };
        for (name, typ) in arguments.iter() {
            let var = data.push_instruction_typ(Instruction::Argument, *typ);
            data.declared_variables.insert(name.to_owned(), var);
        }
        data.process_node(node, available_functions)?;
        Ok(data)
    }

    /// Add a node to the SSA
    fn process_node(&mut self, node: &SyntaxNode, available_functions: &FxHashMap<String, Function>) -> Result<Location, String> {
        Ok(match node {
            // Non-if statement block
            SyntaxNode::Block(header, body) => {
                match &**header {
                    SyntaxNode::Unclassified(token) => {
                        // loop
                        match token.get_inner().as_str() {
                            "loop" => {
                                let (ssa, variables) = self.compile_branch_ssa(body, available_functions)?;
                                if ssa.is_action() {
                                    self.push_instruction(Instruction::Action(self.branches.len()));
                                }
                                self.branches.push(Branch::Loop(ssa));
                                for variable in variables {
                                    *self.declared_variables.get_mut(&variable).unwrap() = Location::internal(self.instruction_counter);
                                    self.push_instruction(Instruction::Theta(self.branches.len()-1, variable));
                                }
                            },
                            _ => {return node.raise(&format!("Invalid keyword `{}`", token.get_inner()));}
                        }

                    },
                    _ => {return node.raise("Invalid syntax 16");}
                };
                Location::internal(self.instruction_counter-1)
            },
            // If statement
            SyntaxNode::IfChain(nodes) => {
                let mut thetas = SortedSet::new();
                let mut ifs = Vec::new();
                let mut els = None;
                let mut is_action = false;
                for node in nodes {
                    match node {
                        SyntaxNode::Block(predicate, body) => {
                            let condition = match &mut *predicate.clone() {
                                SyntaxNode::Adjacent(nodes) => {
                                    if let Some(SyntaxNode::Unclassified(t)) = nodes.first() {
                                        if t == "else" {nodes.remove(0);}
                                    }
                                    if let Some(SyntaxNode::Unclassified(t)) = nodes.first() {
                                        if t == "if" {nodes.remove(0);}
                                    }
                                    if nodes.len() != 1 {
                                        return node.raise("If statements must contain exactly one condition");
                                    }
                                    Some(nodes[0].clone())
                                }
                                SyntaxNode::Unclassified(t) => {
                                    if t != "else" {
                                        return node.raise("Else statements must contain no conditions");
                                    }
                                    None
                                }
                                _ => unreachable!()
                            };

                            let (body_ssa, variables) = self.compile_branch_ssa(body, available_functions)?;
                            let condition_ssa = match condition {
                                Some(c) => Some(self.compile_condition_ssa(&c, available_functions)?.0),
                                None => None 
                            };
                            is_action = is_action || body_ssa.is_action();
                            match condition_ssa {
                                Some(condition_ssa) => ifs.push((body_ssa, condition_ssa)),
                                None => els = Some(body_ssa),
                            };
                            for v in variables {
                                thetas.push(v);
                            }
                        },
                        _ => unreachable!()
                    }
                }
                self.branches.push(Branch::If(ifs, els));
                if is_action {
                    self.push_instruction(Instruction::Action(self.branches.len()-1));
                }
                for variable in thetas {
                    *self.declared_variables.get_mut(&variable).unwrap() = Location::internal(self.instruction_counter);
                    self.push_instruction(Instruction::Theta(self.branches.len()-1, variable));
                }
                Location::internal(self.instruction_counter-1)
            },
            // Keyword phrase, function call, or just a bunch of commands
            SyntaxNode::Adjacent(nodes) => {
                match &nodes[0] {
                    SyntaxNode::Unclassified(text) => match text.get_inner() {
                        // There aren't any keywords yet
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
                            
                            let first = text.get_inner();
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
                        },
                    },
                    _ => {
                        let mut value = Location::internal(0);
                        for node in nodes {
                            value = self.process_node(node, available_functions)?;
                        }
                        value
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
            SyntaxNode::Parenthesis(_, _) => { return node.raise("Lines cannot start with a parenthesis"); },
            SyntaxNode::List(_, _) => { return node.raise("Lines cannot start with a list"); },
        })
    }

    fn push_instruction(&mut self, instruction: Instruction) -> Location {
        let typ = instruction.typ();
        self.push_instruction_typ(instruction, typ)
    }

    fn push_instruction_typ(&mut self, instruction: Instruction, typ: VariableType) -> Location {
        self.instructions.insert(self.instruction_counter, instruction);
        self.types.insert(Location::internal(self.instruction_counter), typ);
        self.instruction_counter += 1;
        Location::internal(self.instruction_counter - 1)
    }

    /// Compile an SSA from code in a branch, returning code and the theta variables
    fn compile_condition_ssa(&self, node: &SyntaxNode, available_functions: &FxHashMap<String, Function>) -> Result<(Ssa, SortedSet<String>), String> {
        let (mut ssa, thetas) = self.compile_branch_ssa(node, available_functions)?;
        let last_variable = Location::internal(ssa.instruction_counter-1);
        ssa.return_variables.push(last_variable);
        Ok((ssa, thetas))
    }

    /// Compile an SSA from code in a branch, returning code and the theta variables
    fn compile_branch_ssa(&self, node: &SyntaxNode, available_functions: &FxHashMap<String, Function>) -> Result<(Ssa, SortedSet<String>), String> {
        let declared_variables = FxHashMap::from_iter(self.declared_variables.iter().map(|(k, v)| (k.to_owned(), v.graduate())));
        let types = FxHashMap::from_iter(self.types.iter().map(|(k, v)| (k.graduate(), *v)));
        let mut data = Self {
            instructions: FxHashMap::default(),
            declared_variables,
            types,
            instruction_counter: 0,
            return_variables: Vec::new(),
            branches: Vec::new(),
        };
        data.process_node(node, available_functions)?;

        // Find all the external variables written to
        let mut variables = SortedSet::new();
        for (name, loc) in self.declared_variables.iter() {
            let branch_loc = data.declared_variables.get(name).unwrap();
            if *branch_loc != loc.graduate() {
                variables.push(name.to_owned());
            }
        }
    
        Ok((Ssa::Unordered { data }, variables))
    }

    pub fn get_used_functions(&self) -> Vec<String> {
        let mut funcs = SortedSet::new();
        for command in self.instructions.values() {
            match &command {
                Instruction::LocalCall(name, _) => {funcs.push(name.clone());},
                _ => ()
            };
        }
        let mut funcs = funcs.to_vec();
        
        for branch in &self.branches {
            match branch {
                Branch::If(items, ssa) => {
                    for (ssa, _) in items {
                        funcs.append(&mut ssa.get_used_functions());
                    }
                    if let Some(ssa) = ssa {
                        funcs.append(&mut ssa.get_used_functions());
                    }
                }
                Branch::Loop(ssa) => {funcs.append(&mut ssa.get_used_functions());}
            }
        }

        funcs
    }

    fn is_action(&self) -> bool {
        for intruction in self.instructions.values() {
            if intruction.is_action() { return true; }
        }
        false
    }
}
impl std::fmt::Debug for SsaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Ssa (")?;
        writeln!(f, "CORE")?;
        let indices = SortedSet::from_unsorted(self.instructions.keys().collect());
        for index in indices {
            writeln!(f, "|    {:04} {:?}", index, self.instructions[index])?;
        }
        for (branch_index, branch) in self.branches.iter().enumerate() {
            writeln!(f, "BRANCH {}", branch_index)?;
            match branch {
                Branch::If(items, ssa) => {
                    for (ssa, condition) in items {
                        let indices = SortedSet::from_unsorted(condition.instructions.keys().collect());
                        for index in indices {
                            writeln!(f, "|    {:04} {:?}", index, condition.instructions[index])?;
                        }
                        let indices = SortedSet::from_unsorted(ssa.instructions.keys().collect());
                        for index in indices {
                            writeln!(f, "+    {:04} {:?}", index, ssa.instructions[index])?;
                        }
                    }
                    if let Some(ssa) = ssa {
                        let indices = SortedSet::from_unsorted(ssa.instructions.keys().collect());
                        for index in indices {
                            writeln!(f, "-    {:04} {:?}", index, ssa.instructions[index])?;
                        }
                    }
                },
                Branch::Loop(ssa) => {
                    let indices = SortedSet::from_unsorted(ssa.instructions.keys().collect());
                    for index in indices {
                        writeln!(f, "|    {:04} {:?}", index, ssa.instructions[index])?;
                    }
                },
            }
        }
        write!(f, ")")
    }
}
