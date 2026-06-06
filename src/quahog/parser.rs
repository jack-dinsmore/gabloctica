use std::{fs::File, io::Read};

use sorted_vec::SortedSet;

use crate::quahog::parser::SyntaxNode::Unclassified;

#[derive(Clone, Debug)]
pub struct Token {
    pub s: String,
    filename: String,
    line_no: u32,
    col_no: u32,
}
impl Token {
    pub fn raise<T>(&self, message: &str) -> Result<T, String> {
        Err(format!("{}:{}:{}\n{}", self.filename, self.line_no, self.col_no, message))
    }
}
impl PartialEq<str> for Token {
    fn eq(&self, other: &str) -> bool {
        self.s == other
    }
}

pub struct Function {
    pub filename: String,
    pub start: u32,
    pub end: u32,
    pub declaration: SyntaxNode,
    pub statements: SyntaxNode,
}
impl Function {
    fn new(declaration_tokens: &[Token], statement_tokens: &[Token]) -> Self {
        let declaration = declaration_tokens.iter().map(|s| SyntaxNode::Unclassified(s.clone())).collect::<Vec<_>>();
        let statements = statement_tokens.iter().map(|s| SyntaxNode::Unclassified(s.clone())).collect::<Vec<_>>();
        Self {
            filename: declaration_tokens[0].filename.clone(),
            start: declaration_tokens[0].line_no,
            end: statement_tokens[statement_tokens.len()-1].line_no,
            declaration: SyntaxNode::Adjacent(declaration),
            statements: SyntaxNode::Adjacent(statements),
        }
    }

    fn reduce(&mut self) -> Result<(), String> {
        self.statements.reduce()?;
        self.declaration.reduce()?;
        Ok(())
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename && self.start == other.start && self.end == other.end
    }
}

#[derive(Clone, Debug)]
pub enum SyntaxNode {
    Unclassified(Token),
    Number(f64),
    Adjacent(Vec<SyntaxNode>),
    Parenthesis(&'static str, Box<SyntaxNode>),
    List(&'static str, Box<SyntaxNode>),
    Binop(&'static str, Box<SyntaxNode>, Box<SyntaxNode>),
    Unop(&'static str, Box<SyntaxNode>),
}
impl SyntaxNode {
    fn reduce(&mut self) -> Result<(), String> {
        self.strip_whitespace();
        self.reduce_list(";")?;
        self.reduce_parens("(", ")")?;
        self.reduce_parens("[", "]")?;
        self.reduce_list(",")?;
        self.reduce_binop(&["=="])?;
        self.reduce_binop(&["="])?;
        self.reduce_binop(&["&&"])?;
        self.reduce_binop(&["||"])?;
        self.reduce_binop(&["**"])?;
        self.reduce_binop(&["*", "/"])?;
        self.reduce_unop("!")?;
        self.reduce_unop("-")?;
        self.reduce_binop(&["&"])?;
        self.reduce_binop(&["^"])?;
        self.reduce_binop(&["|"])?;
        self.reduce_binop(&["+", "-"])?;
        self.reduce_number();
        Ok(())
    }

    fn strip_whitespace(&mut self) {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                let mut i = 0;
                while i < nodes.len() {
                    if let Unclassified(t) = &nodes[i] {
                        if t == " " || t == "\n" || t == "\t" {
                            nodes.swap_remove(i);
                            continue;
                        }
                    }
                    i += 1;
                }
            }
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.strip_whitespace();
            },
            SyntaxNode::Binop(_, n1, n2) => {
                n1.strip_whitespace();
                n2.strip_whitespace();
            },
            SyntaxNode::Unclassified(_) | SyntaxNode::Number(_) => (),
        };
    }

    fn reduce_parens(&mut self, symbol_open: &str, symbol_close: &str) -> Result<(), String> {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                let mut parentheses = Vec::new();
                for i in 0..nodes.len(){
                    if let Unclassified(t) = &nodes[i] {
                        if t == symbol_open {
                            parentheses.push(i);
                        } else if t == symbol_close {
                            let start_index = parentheses.pop().ok_or(t.raise::<()>("Parenthesis was not opened").unwrap_err())?;
                            let inner_node = SyntaxNode::Adjacent(nodes[start_index..i].to_vec());
                            nodes.splice(start_index..i, [inner_node]);
                        }
                    }
                }
                if !parentheses.is_empty() {
                    return Err(format!("{}\nParenthesis was not closed", self.get_filename()));
                }
            }
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.reduce_parens(symbol_open, symbol_close)?;
            },
            SyntaxNode::Binop(_, n1, n2) => {
                n1.reduce_parens(symbol_open, symbol_close)?;
                n2.reduce_parens(symbol_open, symbol_close)?;
            },
            SyntaxNode::Unclassified(_) | SyntaxNode::Number(_) => (),
        };
        Ok(())
    }

    fn reduce_list(&mut self, symbol: &'static str) -> Result<(), String> {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                let mut new_nodes = Vec::new();
                for i in 0..nodes.len(){
                    if let Unclassified(t) = &nodes[i] {
                        if t == symbol {
                            new_nodes.push(SyntaxNode::Adjacent(nodes.drain(..i).collect::<Vec<_>>()));
                        }
                    }
                }
                if !new_nodes.is_empty() {
                    new_nodes.push(SyntaxNode::Adjacent(nodes.drain(..).collect::<Vec<_>>()));
                    *nodes = new_nodes;
                }
            }
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.reduce_list(symbol)?;
            },
            SyntaxNode::Binop(_, n1, n2) => {
                n1.reduce_list(symbol)?;
                n2.reduce_list(symbol)?;
            },
            SyntaxNode::Unclassified(_) | SyntaxNode::Number(_) => (),
        };
        Ok(())
    }

    fn reduce_binop(&mut self, symbols: &[&'static str]) -> Result<(), String> {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                let mut i = 0;
                while i < nodes.len() {
                    if let Unclassified(t) = &nodes[i] {
                        match symbols.iter().position(|x| t == *x) {
                            Some(index) => {
                                let op = symbols[index];
                                if i == 0 {
                                    return t.raise("Binary operation encountered with no left side");
                                } else if i == nodes.len()-1 {
                                    return t.raise("Binary operation encountered with no right side");
                                }
                                let new_node = SyntaxNode::Binop(op, Box::new(nodes[i-1].clone()), Box::new(nodes[i+1].clone()));
                                nodes.swap_remove(i);
                                nodes.swap_remove(i+1);
                                nodes[i-1] = new_node;
                                continue;
                            },
                            None => (),
                        }
                    }
                    i += 1;
                }
            }
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.reduce_binop(symbols)?;
            },
            SyntaxNode::Binop(_, n1, n2) => {
                n1.reduce_binop(symbols)?;
                n2.reduce_binop(symbols)?;
            },
            SyntaxNode::Unclassified(_) | SyntaxNode::Number(_) => (),
        };
        Ok(())
    }

    fn reduce_unop(&mut self, symbol: &'static str) -> Result<(), String> {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                let mut i = 0;
                while i < nodes.len() {
                    if let Unclassified(t) = &nodes[i] {
                        if t == symbol {
                            if i == 0 {
                                return t.raise("Unary operation encountered with no left side");
                            }
                            let new_node = SyntaxNode::Unop(symbol, Box::new(nodes[i-1].clone()));
                            nodes.swap_remove(i);
                            nodes[i-1] = new_node;
                            continue;
                        }
                    }
                    i += 1;
                }
            }
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.reduce_unop(symbol)?;
            },
            SyntaxNode::Binop(_, n1, n2) => {
                n1.reduce_unop(symbol)?;
                n2.reduce_unop(symbol)?;
            },
            SyntaxNode::Unclassified(_) | SyntaxNode::Number(_) => (),
        };
        Ok(())
    }

    fn reduce_number(&mut self) {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                for node in nodes {
                    node.reduce_number();
                }
            }
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.reduce_number();
            },
            SyntaxNode::Binop(_, n1, n2) => {
                n1.reduce_number();
                n2.reduce_number();
            },
            SyntaxNode::Unclassified(v) => {
                if let Ok(n) = v.s.parse::<f64>() {
                    *self = SyntaxNode::Number(n);
                }
            },
            SyntaxNode::Number(_) => (),
        };
    }

    fn get_filename(&self) -> &str {
        match self {
            Unclassified(token) => &token.filename,
            SyntaxNode::Number(_) => unreachable!(),
            SyntaxNode::Adjacent(syntax_nodes) => syntax_nodes[0].get_filename(),
            SyntaxNode::Parenthesis(_, syntax_node) => syntax_node.get_filename(),
            SyntaxNode::List(_, syntax_node) => syntax_node.get_filename(),
            SyntaxNode::Binop(_, syntax_node, _) => syntax_node.get_filename(),
            SyntaxNode::Unop(_, syntax_node) => syntax_node.get_filename(),
        }
    }
}

/// Load file, handling import statements
fn load_file(filename: &str) -> Result<Vec<Token>, String> {
    let mut file = File::open(filename).map_err(|_| format!("Could not find file {}", filename))?;
    let mut text = "".to_owned();
    file.read_to_string(&mut text).map_err(|_| format!("Could not read file {}", filename))?;
    load_str(&text, filename)
}

/// Load file, handling import statements
fn load_str(text: &str, filename: &str) -> Result<Vec<Token>, String> {
    let mut stream = get_stream(&text, filename)?;
    reduce_pragmas(&mut stream)?;
    Ok(stream)
}

/// Get a stream of all tokens
fn get_stream(text: &str, filename: &str) -> Result<Vec<Token>, String> {
    let split_chars = unsafe { SortedSet::from_sorted(vec![' ', '!', '|', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';', '<', '=', '>', '?', '@', '[', '\\', ']', '^', '`', '{', '|', '}', '~']) };
    let mut tokens = Vec::new();
    let mut token = Vec::new();
    let mut line_no = 0;
    let mut col_no = 0;
    for c in text.chars() {
        if !c.is_ascii(){
            return Err(format!("{}:{}:{}\nNon-ascii characters are not allowed", filename, line_no, col_no));
        }

        if split_chars.contains(&c) {
            tokens.push(Token {
                s: token.drain(..).collect(),
                filename: filename.to_owned(),
                line_no,
                col_no,
            });
        }
        col_no += 1;
        token.push(c);
        if c == '\n' {
            line_no += 1;
            col_no = 0;
        }
    }
    Ok(tokens)
}

fn reduce_pragmas(tokens: &mut Vec<Token>) -> Result<(), String> {
    for i in 0..tokens.len() {
        if &tokens[i] != "#" {continue;}
        if i != 0 && &tokens[i-1] != "\n" {continue;}
        // It's a pragma
        let command: &str = &tokens[i+1].s;
        match command {
            "include" => {
                let open_bracket: &str = &tokens[i+2].s;
                let new_stream = match open_bracket {
                    "<" => {
                        // Library import
                        let mut end_index = i+3;
                        while &tokens[end_index] != ">" { end_index += 1; }
                        let lib_name: String = tokens[i+3..end_index].iter().map(|t| &t.s as &str).collect();

                        match &lib_name as &str {
                            "math" => load_str(include_str!("../../assets/quahog/math.qhg"), "assets/quahog/math.qhg"),
                            _ => tokens[i].raise(&format!("Could not find library {}", lib_name)),
                        }
                    },
                    "\"" => {
                        // File import
                        let mut end_index = i+3;
                        while &tokens[end_index] != "\"" { end_index += 1; }
                        let lib_name: String = tokens[i+3..end_index].iter().map(|t| &t.s as &str).collect();

                        load_file(&lib_name)
                    },
                    _ => return tokens[i+2].raise("Invalid symbol in an import pragma")
                }?;

                let mut end_index = i;
                while &tokens[end_index] != "\n" { end_index += 1; }

                // Insert the stream
                tokens.splice(i..end_index, new_stream);
            },
            _ => return tokens[i].raise("Invalid pragma"),
        };
    }
    Ok(())
}

/// Split a token stream into a list of fucntions
fn get_functions(tokens: Vec<Token>) -> Result<Vec<Function>, String> {
    let mut last_newline = 0;
    let mut start_function = None;
    let mut functions = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if token == "\n" { last_newline = i; }
        if token == "{" {
            match start_function {
                Some(_) => return token.raise("Nested braces are not allowed"),
                None => start_function = Some((last_newline, i)),
            };
        }
        if token == "}" {
            match start_function {
                Some((start_newline, start_brace)) => {
                    let declaration_tokens = &tokens[start_newline+1..start_brace];
                    let statement_tokens = &tokens[start_brace+1..i];
                    let mut function = Function::new(declaration_tokens, statement_tokens);
                    function.reduce()?;
                    functions.push(function);
                    start_function = None;
                },
                None => { return token.raise("Closed brace was not opened"); }
            }
        }
    }
    if let Some(_) = start_function {
        return Err(format!("{}\nOpen brace was not closed", tokens[0].filename));
    }

    Ok(functions)
}

pub fn parse_file(filename: &str) -> Result<Vec<Function>, String> {
    get_functions(load_file(filename)?)
}

pub fn parse_str(s: &str, filename: &str) -> Result<Vec<Function>, String> {
    get_functions(load_str(s, filename)?)
}