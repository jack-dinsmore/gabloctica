use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use crate::{parser::SyntaxNode, ssa::{Bytecode, Ssa, VariableType}};

lazy_static! {
    static ref CONSTANT_PRECURSOR: SyntaxNode = {
        let tokens = crate::parser::load_str("float const() ", "root").unwrap();
        SyntaxNode::tree(tokens).unwrap()
    };
}

pub struct Function {
    name: String,
    lines: Vec<SyntaxNode>,
    return_value: VariableType,
    arguments: Vec<(String, VariableType)>,
}

impl Function {
    fn new(header: &SyntaxNode, body: &SyntaxNode) -> Result<Self, String> {
        let mut return_value = VariableType::Float;
        let mut node_iter = match header {
            SyntaxNode::Adjacent(v) => v.iter(),
            _ => return header.raise("Invalid syntax"),
        };

        match node_iter.next() {
            Some(SyntaxNode::Unclassified(t)) => if t != "fn" {
                return t.raise("Function declarations must start with fn")
            }
            _ => return header.raise("Invalid syntax"),
        };

        let function_name = match node_iter.next() {
            Some(SyntaxNode::Unclassified(t)) => &t.s,
            _ => return header.raise("Invalid syntax"),
        };

        let mut next = node_iter.next();
        if let Some(SyntaxNode::Parenthesis(c, arg)) = next {
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
                                    last_argument = Some(token.s.clone());
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
                    _ => return header.raise("Braces must contain expression"),
                };
            }
            _ => return header.raise("Function declaractions must have parentheses")
        }

        let lines = match body {
            SyntaxNode::List(c, list) => {
                if *c != ";" { return body.raise("Braces must contain expressions"); }
                match &**list {
                    SyntaxNode::Adjacent(items) => items,
                    _ => return list.raise("Invalid syntax"),
                }
                
            },
            _ => return header.raise("Braces must contain expression"),
        };

        Ok(Self {
            name: function_name.to_owned(),
            lines: lines.clone(),
            return_value,
            arguments,
        })
    }

    fn compile(&self, available_functions: &FxHashMap<String, Function>) -> Result<(Ssa, Vec<String>), String> {
        let mut ssa = Ssa::new();
        let mut used_functions = Vec::new();
        for (name, typ) in &self.arguments {
            ssa.add_arg(name, *typ)
        }
        for line in &self.lines {
            ssa.process_node(line, available_functions, &mut used_functions);
        }
        ssa.order_instructions();
        Ok((ssa, used_functions))
    }
}

struct Compiler {
    functions: FxHashMap<String, Function>,
}

impl Compiler {
    pub fn new(tree: SyntaxNode) -> Result<Self, String> {
        let mut constants = Vec::new();
        let mut functions = FxHashMap::default();

        let main_list = match tree {
            SyntaxNode::Adjacent(nodes) => nodes,
            _ => return tree.raise("Invalid syntax"),
        };

        for entry in main_list {
            match entry {
                SyntaxNode::Block(header, node_end) => {
                    let function = Function::new(&header, &node_end)?;
                    functions.insert(function.name.to_owned(), function);
                },
                SyntaxNode::List(_, syntax_node) => {
                    match *syntax_node {
                        SyntaxNode::Adjacent(syntax_nodes) => constants.append(&mut syntax_nodes.clone()),
                        _ => return syntax_node.raise("Invalid syntax"),
                    }
                },
                _ => return entry.raise("Invalid syntax"),
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
            let (ssa, used_functions) = self.functions[&name].compile(&self.functions)?;
            for f in used_functions {
                queue.push(f);
            }
            compiled.insert(name, ssa);
        }
        Ok(compiled)
    }
}

fn compile_tree(tree: SyntaxNode, _filename: &str) -> Result<Vec<u8>, String> {
    let compiler = Compiler::new(tree)?;
    let ssa = compiler.compile("main")?;
    // let const_ssa = compiler.compile("const")?; // TODO
    
    // Optimize IR

    // Write to bytecode
    let mut names = vec!["main".to_owned()];
    let mut compiled_functions = vec![Bytecode::new(&ssa["main"])];
    for (name, ssa) in &ssa {
        if name == "main" { continue; }
        names.push(name.to_owned());
        compiled_functions.push(Bytecode::new(ssa));
    }
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
    compile_tree(tree, filename)
}