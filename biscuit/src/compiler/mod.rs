// Functions to perform the full compilation process
mod ssa;
mod implementer;

use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use crate::{bytecode::VariableType, compiler::implementer::Bytecode, parser::SyntaxNode};
use ssa::Ssa;

lazy_static! {
    static ref CONSTANT_PRECURSOR: SyntaxNode = {
        let tokens = crate::parser::load_str("fn const() {}", "root").unwrap();
        let tree = SyntaxNode::tree(tokens).unwrap();
        match tree {
            SyntaxNode::Block(header, _) => (&*header).clone(),
            _ => unreachable!()
        }
    };
}

pub struct Function {
    pub name: String,
    lines: Vec<SyntaxNode>,
    pub return_value: VariableType,
    arguments: Vec<(String, VariableType)>,
}

impl Function {
    fn new(header: &SyntaxNode, body: &SyntaxNode) -> Result<Self, String> {
        let mut return_value = VariableType::Float;
        let mut node_iter = match header {
            SyntaxNode::Adjacent(v) => v.iter(),
            _ => return header.raise("Invalid syntax 1"),
        };

        let n = node_iter.next();
        match n {
            Some(SyntaxNode::Unclassified(t)) => if t != "fn" {
                return t.raise("Function declarations must start with fn")
            }
            _ => return header.raise("Invalid syntax 2"),
        };

        let function_name = match node_iter.next() {
            Some(SyntaxNode::Unclassified(t)) => t.get_inner(),
            _ => return header.raise("Invalid syntax 3"),
        };

        let mut next = node_iter.next();
        if let Some(SyntaxNode::Parenthesis(c, _)) = next {
            if *c == "[" {
                return_value = VariableType::List;
                next = node_iter.next();
            }
        }

        // Handle arguments
        let mut arguments = Vec::new();
        match next {
            Some(SyntaxNode::Parenthesis(c, arg)) => {
                if *c != "(" { return header.raise("Function declaractions must have parentheses"); }
                let mut last_argument : Option<String> = None;
                match &**arg {
                    SyntaxNode::Adjacent(list) => {
                        for item in list {
                            match item {
                                SyntaxNode::Unclassified(token) => {
                                    // Commit float
                                    if let Some(item) = &last_argument {
                                        arguments.push((item.to_owned(), VariableType::Float))
                                    }
                                    last_argument = Some(token.get_inner().clone());
                                },
                                SyntaxNode::Parenthesis(c, syntax_node) => {
                                    if *c != "[" { return syntax_node.raise("Only brackets can appear in variable definitions"); }
                                    // Commit list
                                    if let Some(item) = &last_argument {
                                        arguments.push((item.to_owned(), VariableType::List))
                                    } else {
                                        return syntax_node.raise("Brackets must follow variable name");
                                    }
                                },
                                _ => {return header.raise("Invalid symbol in argument list");}
                            };
                        }
                        if let Some(item) = &last_argument {
                            arguments.push((item.to_owned(), VariableType::Float))
                        }
                    },
                    _ => return header.raise("Invalid syntax 4"),
                };
            }
            _ => return header.raise("Function declaractions must have parentheses")
        }

        let lines = match body {
            SyntaxNode::Adjacent(nodes) => nodes,
            _ => return header.raise("Braces must contain expression"),
        };

        Ok(Self {
            name: function_name.to_owned(),
            lines: lines.clone(),
            return_value,
            arguments,
        })
    }

    fn compile(&self, available_functions: &FxHashMap<String, Function>) -> Result<Ssa, String> {
        let mut arguments = FxHashMap::default();
        for (name, typ) in &self.arguments {
            arguments.insert(name.to_owned(), *typ);
        }
        let mut ssa = Ssa::new(&self.lines, &arguments, available_functions)?;
        ssa.order_instructions();
        Ok(ssa)
    }
}

struct Compiler {
    functions: FxHashMap<String, Function>,
}

impl Compiler {
    pub fn new(tree: &SyntaxNode) -> Result<Self, String> {
        let mut constants = Vec::new();
        let mut functions = FxHashMap::default();

        let main_list: &[SyntaxNode] = match tree {
            SyntaxNode::Adjacent(nodes) => nodes,
            _ => std::slice::from_ref(&tree),
        };

        for entry in main_list {
            match entry {
                SyntaxNode::Block(header, node_end) => {
                    let function = Function::new(&header, &node_end)?;
                    functions.insert(function.name.to_owned(), function);
                },
                SyntaxNode::List(_, syntax_node) => {
                    match &**syntax_node {
                        SyntaxNode::Adjacent(syntax_nodes) => constants.append(&mut syntax_nodes.clone()),
                        _ => return syntax_node.raise("Invalid syntax 7"),
                    }
                },
                _ => return entry.raise("All code outside functions must be pragmas or function definitions"),
            }
        }
        
        let constants = Function::new(&CONSTANT_PRECURSOR, &SyntaxNode::Adjacent(constants))?;
        functions.insert(constants.name.to_owned(), constants);

        Ok(Self {
            functions,
        })
    }

    pub fn compile(&self, name: &str) -> Result<FxHashMap<String, Ssa>, String> {
        let mut compiled = FxHashMap::default();
        let mut queue = vec![name.to_owned()];
        while !queue.is_empty() {
            let name = queue.pop().unwrap();
            let ssa = self.functions[&name].compile(&self.functions)?;
            for f in &ssa.get_used_functions() {
                queue.push(f.clone());
            }
            compiled.insert(name, ssa);
        }
        Ok(compiled)
    }
}

fn compile_tree(tree: &SyntaxNode) -> Result<Vec<u8>, String> {
    let compiler = Compiler::new(tree)?;
    if !compiler.functions.contains_key("main") {
        return tree.raise("No function named main");
    }
    let ssa = compiler.compile("main")?;
    dbg!(&ssa);
    // let const_ssa = compiler.compile("const")?; // TODO implement constants
    
    // Optimize IR

    // Write to bytecode
    let mut names = vec!["main".to_owned()];
    let mut compiled_functions = vec![Bytecode::new(&ssa["main"])];
    for (name, ssa) in &ssa {
        if name == "main" { continue; }
        names.push(name.to_owned());
        compiled_functions.push(Bytecode::new(ssa));
    }

    // Get all the function locations
    let mut locations = FxHashMap::default();
    let mut net_loc = 0;
    for (name, bytecode) in names.iter().zip(&compiled_functions) {
        locations.insert(name.to_owned(), net_loc);
        net_loc += bytecode.len();
    }

    let mut code = Vec::new();
    for mut f in compiled_functions {
        f.replace_calls(&locations);
        code.extend(f.code().into_iter());
    }
    Ok(code)
}

/// Compile a string (usually read from a file) of Biscuit code to binary
pub fn compile_str(s: &str, filename: &str) -> Result<Vec<u8>, String> {
    let tokens = crate::parser::load_str(s, filename)?;
    let tree = crate::parser::SyntaxNode::tree(tokens)?;
    compile_tree(&tree)
}