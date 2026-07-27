use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;

use crate::{bytecode::{FUNCTION_MAP_LOWER, VariableType}, compiler::Function, parser::SyntaxNode};

#[derive(Clone, Debug)]
pub enum Instruction {
    Argument,
    LiteralVector(Vec<f64>),
    LiteralFloat(f64),

    Call(u8, Location),
    LocalCall(String, Location),
    Theta(usize, String),
    Action(usize),

    Ld(Location, Location), // Get index B f vector A
    St(Location, Location, Location), // Store C in index B of vector A 
    Stb(Location, Location), // Store C in back of vector A 
    
    Lt(Location, Location),
    Gt(Location, Location),
    Le(Location, Location), 
    Ge(Location, Location), 
    Eq(Location, Location),
    
    And(Location, Location),
    Or(Location, Location),
    Xor(Location, Location),
    Not(Location),

    Add(Location, Location),
    Sub(Location, Location),
    Mul(Location, Location),
    Div(Location, Location),
    Neg(Location),
    Pow(Location, Location),
}
impl Instruction {
    fn is_action(&self) -> bool {
        match &self {
            Instruction::Action(_) | Instruction::Call(_, _) | Instruction::LocalCall(_, _) | Instruction::Stb(_, _) |
            Instruction::St(_, _, _) => true,
            _ => false,
        }
    }
    fn typ(&self) -> VariableType {
        match &self {
            Instruction::LiteralVector(_) | Instruction::St(_, _, _) | Instruction::Stb(_, _) => VariableType::List,
            Instruction::Ld(_, _) | Instruction::LiteralFloat(_) | Instruction::Lt(_, _) | Instruction::Gt(_, _) | 
            Instruction::Le(_, _) |  Instruction::Ge(_, _) | Instruction::Eq(_, _) | Instruction::And(_, _) |
            Instruction::Or(_, _) | Instruction::Xor(_, _) | Instruction::Not(_) | Instruction::Add(_, _) | 
            Instruction::Sub(_, _) | Instruction::Mul(_, _) | Instruction::Div(_, _) | Instruction::Neg(_) | 
            Instruction::Pow(_, _) => VariableType::Float,
            Instruction::LocalCall(_, _) | Instruction::Action(_) => VariableType::Null,
            Instruction::Argument | Instruction::Call(_, _) | Instruction::Theta(_, _) => unreachable!(),
        }
    }
    fn get_var_dependencies(&self) -> Vec<Location> {
        match &self {
            Instruction::Argument | Instruction::LiteralVector(_) | Instruction::LiteralFloat(_) | Instruction::Theta(_, _) |
            Instruction::Action(_)=> vec![],
            Instruction::Call(_, a) | Instruction::LocalCall(_, a) | Instruction::Not(a) | Instruction::Neg(a) => vec![*a],
            Instruction::Ld(a, b) | Instruction::Stb(a, b) | Instruction::Lt(a, b) | Instruction::Gt(a, b) |
            Instruction::Le(a, b) | Instruction::Ge(a, b) | Instruction::Eq(a, b) | Instruction::And(a, b) |
            Instruction::Or(a, b) | Instruction::Xor(a, b) | Instruction::Add(a, b) | Instruction::Sub(a, b) |
            Instruction::Mul(a, b) | Instruction::Div(a, b) | Instruction::Pow(a, b) => vec![*a, *b],
            Instruction::St(a, b, c) => vec![*a, *b, *c],
        }
    }
    fn get_branch_dependencies(&self) -> Vec<usize> {
        match &self {
            Instruction::Theta(branch, _) | Instruction::Action(branch) => { vec![*branch] },
            _ => Vec::new()
        }
    }
}

pub(super) enum Branch {
    If(Vec<(Ssa, Ssa)>, Option<Ssa>), // ((body code, if condition), else code)
    Loop(Ssa),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Location {
    pub tier: u32,
    pub index: u32,
}
impl Location {
    pub fn internal(index: u32) -> Self {
        Self {
            tier: 0,
            index
        }
    }
    pub fn graduate(&self) -> Self{
        Self {
            tier: self.tier + 1,
            index: self.index
        }
    }
}
impl std::fmt::Debug for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.tier == 0 {
            write!(f, "{:04}", self.index)
        } else {
            write!(f, "{}_{:04}", self.tier, self.index)
        }
    }
}

pub(super) struct Ssa {
    pub instructions: FxHashMap<u32, Instruction>,
    pub types: FxHashMap<Location, VariableType>,
    pub declared_variables: FxHashMap<String, Location>,
    instruction_counter: u32,
    pub instruction_order: Vec<u32>, // Contains all the instructions in the order in which they should be performed.
    pub return_variables: Vec<Location>,
    pub branches: Vec<Branch>, // The hash map maps from the local block variable to the main block variable
}
impl Ssa {
    pub fn new(lines: &[SyntaxNode], arguments: &FxHashMap<String, VariableType>, available_functions: &FxHashMap<String, Function>) -> Result<Self, String> {
        let mut ssa = Self {
            instructions: FxHashMap::default(),
            types: FxHashMap::default(),
            declared_variables: FxHashMap::default(),
            instruction_counter: 0,
            instruction_order: Vec::new(),
            return_variables: Vec::new(),
            branches: Vec::new(),
        };
        for (name, typ) in arguments.iter() {
            let var = ssa.push_instruction_typ(Instruction::Argument, *typ);
            ssa.declared_variables.insert(name.to_owned(), var);
        }
        for line in lines {
            ssa.process_node(line, available_functions)?;
        }
        Ok(ssa)
    }

    /// Add a node to the SSA
    pub fn process_node(&mut self, node: &SyntaxNode, available_functions: &FxHashMap<String, Function>) -> Result<Location, String> {
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
                            let (body_ssa, variables) = self.compile_branch_ssa(body, available_functions)?;
                            let (pred_ssa, _) = self.compile_branch_ssa(predicate, available_functions)?;
                            is_action = is_action && body_ssa.is_action();
                            match &**predicate {
                                SyntaxNode::Adjacent(_) => {
                                    ifs.push((body_ssa, pred_ssa))
                                },
                                SyntaxNode::Unclassified(_) => {
                                    els = Some(body_ssa)
                                },
                                _ => unreachable!()
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
            // Keyword phrase or function call
            SyntaxNode::Adjacent(nodes) => {
                let first: &str = match &nodes[0] {
                    SyntaxNode::Unclassified(text) => &text.get_inner(),
                    _ => {return node.raise("Invalid syntax 17");}
                };
                match first {
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
            SyntaxNode::Parenthesis(_, _) => { return node.raise("Lines cannot start with a parenthesis"); },
            SyntaxNode::List(_, _) => { return node.raise("Lines cannot start with a list"); },
        })
    }

    pub fn get_last_used(&self) -> FxHashMap<Location, usize> {
        let mut last_usage = FxHashMap::default();
        for (i, op) in self.instruction_order.iter().enumerate() {
            for loc in self.instructions[op].get_var_dependencies() {
                match last_usage.get_mut(&loc) {
                    Some(index) => {*index = i;},
                    None => {last_usage.insert(loc, i);},
                }
            }
        }
        last_usage
    }

    // Get the order of instructions, including skipping unnecessary ones
    pub fn order_instructions(&mut self) {
        let mut used_branches = SortedSet::new();
        let mut used_vars = SortedSet::new();
        let mut used_queue = Vec::new();

        // Add all the return variables
        for v in &self.return_variables {
            if v.tier == 0 { 
                used_queue.push(v.index);
            }
        }

        // Add all the functions
        for (index, intruction) in self.instructions.iter() {
            if intruction.is_action() {
                used_queue.push(*index);
            }
        }
        
        // Figure out which other variables are used by the above
        while let Some(next_item) = used_queue.pop() {
            if used_vars.push(next_item).1.is_some() {
                // The variable was already used
                continue;
            };
            for var in self.instructions[&next_item].get_var_dependencies() {
                if var.tier == 0 { used_queue.push(var.index); }
            }
            for branch in self.instructions[&next_item].get_branch_dependencies() {
                used_branches.push(branch);
            }
        }
        self.instruction_order = used_vars.to_vec();

        for branch in used_branches {
            // Get all used variables
            let mut variable_names = SortedSet::new();
            for var in &used_vars {
                if let Instruction::Theta(b, name) = &self.instructions[&var] {
                    if *b == branch {
                        variable_names.push(name);
                    }
                }
            }
            
            match &mut self.branches[branch] {
                Branch::If(items, ssa) => {
                    for (ssa, _) in items {
                        for v in &variable_names {
                            self.return_variables.push(*self.declared_variables.get(*v).unwrap());
                        }
                        ssa.order_instructions();
                    }
                    if let Some(ssa) = ssa {
                        for v in &variable_names {
                            self.return_variables.push(*self.declared_variables.get(*v).unwrap());
                        }
                        ssa.order_instructions();
                    }
                },
                Branch::Loop(ssa) => {ssa.order_instructions()}
            }
        }
    }

    pub fn push_instruction(&mut self, instruction: Instruction) -> Location {
        let typ = instruction.typ();
        self.push_instruction_typ(instruction, typ)
    }

    pub fn push_instruction_typ(&mut self, instruction: Instruction, typ: VariableType) -> Location {
        self.instructions.insert(self.instruction_counter, instruction);
        self.types.insert(Location::internal(self.instruction_counter), typ);
        self.instruction_counter += 1;
        Location::internal(self.instruction_counter - 1)
    }

    /// Compile an SSA from code in a branch, returning code and the theta variables
    fn compile_branch_ssa(&self, node: &SyntaxNode, available_functions: &FxHashMap<String, Function>) -> Result<(Self, SortedSet<String>), String> {
        let lines : &[SyntaxNode] = match node {
            SyntaxNode::Adjacent(nodes) => nodes,
            _ => std::slice::from_ref(node)
        };

        let declared_variables = FxHashMap::from_iter(self.declared_variables.iter().map(|(k, v)| (k.to_owned(), v.graduate())));
        let types = FxHashMap::from_iter(self.types.iter().map(|(k, v)| (k.graduate(), *v)));
        let mut ssa = Self {
            instructions: FxHashMap::default(),
            declared_variables,
            types,
            instruction_counter: 0,
            instruction_order: Vec::new(),
            return_variables: Vec::new(),
            branches: Vec::new(),
        };
        for line in lines {
            ssa.process_node(line, available_functions)?;
        }

        // Find all the external variables written to
        let mut variables = SortedSet::new();
        for (name, loc) in self.declared_variables.iter() {
            let branch_loc = ssa.declared_variables.get(name).unwrap();
            if *branch_loc != loc.graduate() {
                variables.push(name.to_owned());
            }
        }
    
        Ok((ssa, variables))
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

    pub fn is_action(&self) -> bool {
        for intruction in self.instructions.values() {
            if intruction.is_action() { return true; }
        }
        false
    }
}
impl std::fmt::Debug for Ssa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Ssa (")?;
        let indices = SortedSet::from_unsorted(self.instructions.keys().collect());
        for index in indices {
            writeln!(f, "    {:04} {:?}", index, self.instructions[index])?;
        }
        write!(f, ")")
    }
}
