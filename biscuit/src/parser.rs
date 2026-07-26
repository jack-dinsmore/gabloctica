use std::{fs::File, io::Read};

use sorted_vec::SortedSet;

use crate::parser::SyntaxNode::{Parenthesis, Unclassified};

#[derive(Clone)]
pub struct Token<T: std::fmt::Display + PartialEq> {
    s: T,
    filename: String,
    line_no: u32,
    col_no: u32,
}
impl<T: std::fmt::Display + PartialEq> Token<T> {
    pub fn raise_str(&self, message: &str) -> String {
        format!("{}:{}:{}\n{}", self.filename, self.line_no+1, self.col_no+1, message)
    }
    pub fn raise<R>(&self, message: &str) -> Result<R, String> {
        Err(self.raise_str(message))
    }
    pub(crate) fn get_inner(&self) -> &T {
        &self.s
    }
}
impl<T: std::fmt::Display + PartialEq> std::fmt::Debug for Token<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.s)
    }
}

impl PartialEq<str> for Token<String> {
    fn eq(&self, other: &str) -> bool {
        self.s == *other
    }
}
impl PartialEq<f64> for Token<f64> {
    fn eq(&self, other: &f64) -> bool {
        self.s == *other
    }
}

#[derive(Clone)]
pub enum SyntaxNode {
    Unclassified(Token<String>),
    Number(Token<f64>),
    Adjacent(Vec<SyntaxNode>),
    Parenthesis(&'static str, Box<SyntaxNode>),
    List(&'static str, Box<SyntaxNode>),
    Binop(&'static str, Box<SyntaxNode>, Box<SyntaxNode>),
    Unop(&'static str, Box<SyntaxNode>),
    Block(Box<SyntaxNode>, Box<SyntaxNode>),
}
impl SyntaxNode {
    fn reduce(&mut self) -> Result<(), String> {
        self.strip_comment()?;
        self.strip_newline()?;
        self.reduce_parens("{", "}")?;
        self.reduce_blocks()?;
        self.reduce_semicolon()?;
        self.reduce_parens("(", ")")?;
        self.reduce_parens("[", "]")?;
        self.reduce_list(",")?;
        self.reduce_total_binop(&["*=", "/=", "+=", "-="])?;
        self.reduce_binop(&["==", "!="])?;
        self.reduce_total_binop(&["="])?;
        self.reduce_unop(&["!", "-"])?;
        self.reduce_binop(&["&&"])?;
        self.reduce_binop(&["||"])?;
        self.reduce_binop(&["**"])?;
        self.reduce_binop(&["*", "/"])?;
        self.reduce_binop(&["+", "-"])?;
        self.reduce_number();
        self.reduce_singletons();
        dbg!(&self);
        Ok(())
    }

    fn strip_comment(&mut self) -> Result<(), String> {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                let mut comment_index = None;
                let mut i = 0;
                while i < nodes.len() {
                    if let Unclassified(t) = &nodes[i] {
                        if t == "//" {
                            comment_index = Some(i);
                        } else if t == "\n" {
                            match comment_index {
                                Some(j) => {
                                    nodes.drain(j..i);
                                    comment_index = None;
                                    i = j;
                                },
                                None => (),
                            }
                        }
                    } else {
                        nodes[i].strip_comment()?;
                    }
                    i += 1;
                }
            },
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.strip_comment()?;
            },
            SyntaxNode::Binop(_, n1, n2) | SyntaxNode::Block(n1, n2) => {
                n1.strip_comment()?;
                n2.strip_comment()?;
            },
            SyntaxNode::Unclassified(token) => return token.raise("Invalid syntax 9"),
            SyntaxNode::Number(_) => unreachable!()
        }
        Ok(())
    }

    fn strip_newline(&mut self) -> Result<(), String> {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                let mut i = 0;
                while i < nodes.len() {
                    if let Unclassified(t) = &nodes[i] {
                        if t.s == "\n" {
                            nodes.remove(i);
                            i -= 1;
                        }
                    }
                    i += 1;
                }
            },
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.strip_comment()?;
            },
            SyntaxNode::Binop(_, n1, n2) | SyntaxNode::Block(n1, n2) => {
                n1.strip_comment()?;
                n2.strip_comment()?;
            },
            SyntaxNode::Unclassified(token) => return token.raise("Invalid syntax 9"),
            SyntaxNode::Number(_) => unreachable!()
        }
        Ok(())
    }

    fn reduce_parens(&mut self, symbol_open: &'static str, symbol_close: &'static str) -> Result<(), String> {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                let mut parentheses = Vec::new();
                let mut i = 0;
                while i < nodes.len() {
                    match &nodes[i] {
                        Unclassified(t) => {
                            if t == symbol_open {
                                parentheses.push(i);
                                i += 1
                            } else if t == symbol_close {
                                let start_index = parentheses.pop().ok_or(t.raise_str("Parenthesis was not opened"))?;
                                let inner_node = SyntaxNode::Adjacent(nodes[start_index+1..i].to_vec());
                                let paren = SyntaxNode::Parenthesis(symbol_open, Box::new(inner_node));
                                nodes.splice(start_index..=i, [paren]);
                                i = start_index+1
                            } else {
                                i += 1
                            }
                        },
                        _ => {
                            nodes[i].reduce_parens(symbol_open, symbol_close)?;
                            i += 1;
                        }
                    };
                }
                if !parentheses.is_empty() {
                    return self.raise("Parenthesis was not closed");
                }
            }
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.reduce_parens(symbol_open, symbol_close)?;
            },
            SyntaxNode::Binop(_, n1, n2) | SyntaxNode::Block(n1, n2) => {
                n1.reduce_parens(symbol_open, symbol_close)?;
                n2.reduce_parens(symbol_open, symbol_close)?;
            },
            SyntaxNode::Unclassified(token) => return token.raise("Invalid syntax 10"),
            SyntaxNode::Number(_) => unreachable!()
        };
        Ok(())
    }

    fn reduce_blocks(&mut self) -> Result<(), String> {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                let mut start_predicate = 0;
                let mut i = 0;
                while i < nodes.len() {
                    match &nodes[i] {
                        Unclassified(token) => if token.s == ";" {
                            start_predicate = i;
                        } else {
                            i += 1;
                        },
                        Parenthesis("{", inner) =>{
                            let predicate = SyntaxNode::Adjacent(nodes[start_predicate..i].to_vec());
                            let block = SyntaxNode::Block(Box::new(predicate), inner.clone());
                            nodes.splice(start_predicate..=i, [block]);
                            start_predicate += 1;
                            i = start_predicate;
                        },
                        _ => i += 1,
                    }
                }
            }
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.reduce_blocks()?;
            },
            SyntaxNode::Binop(_, n1, n2) | SyntaxNode::Block(n1, n2) => {
                n1.reduce_blocks()?;
                n2.reduce_blocks()?;
            },
            SyntaxNode::Unclassified(_) => (),
            SyntaxNode::Number(_) => unreachable!()
        };
        Ok(())
    }

    fn reduce_semicolon(&mut self) -> Result<(), String> {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                let mut start_index = 0;
                let mut i = 0;
                while i < nodes.len() {
                    match &nodes[i] {
                        SyntaxNode::Unclassified(t) => if t == ";" {
                            if start_index < i {
                                let line = SyntaxNode::Adjacent(nodes[start_index..i].to_vec());
                                nodes.splice(start_index..=i, [line]);
                            }
                            start_index += 1;
                            i = start_index;
                        } else {
                            i += 1;
                        },
                        SyntaxNode::Block(_, _) => {
                            let mut block_copy = nodes[i].clone();
                            if let SyntaxNode::Block(_, body) = &mut block_copy { body.reduce_semicolon()?; }
                            if start_index > i {
                                let line = SyntaxNode::Adjacent(nodes[start_index..i].to_vec());
                                nodes.splice(start_index..=i, [line, block_copy]);
                                start_index += 2;
                            } else {
                                nodes[i] = block_copy;
                                start_index += 1;
                            }
                            i = start_index;
                        },
                        _ => i += 1
                    };
                }
                if start_index != nodes.len() {
                    return nodes[nodes.len() - 1].raise("Last line must end in a semicolon");
                }
            }

            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.reduce_semicolon()?;
            },
            SyntaxNode::Binop(_, n1, n2) => {
                n1.reduce_semicolon()?;
                n2.reduce_semicolon()?;
            },
            SyntaxNode::Block(_, n2) => {
                n2.reduce_semicolon()?;
            },
            SyntaxNode::Unclassified(token) => return token.raise("Invalid syntax 11"),
            SyntaxNode::Number(_) => unreachable!()
        };
        Ok(())
    }

    fn reduce_list(&mut self, symbol: &'static str) -> Result<(), String> {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                let mut start_index = 0;
                let mut i = 0;
                while i < nodes.len() {
                    match &nodes[i] {
                        SyntaxNode::Unclassified(t) => if t == symbol {
                            let line = SyntaxNode::Adjacent(nodes[start_index..i].to_vec());
                            nodes.splice(start_index..=i, [line]);
                            start_index += 1;
                            i = start_index;
                        },
                        _ => i += 1
                    };
                }
            }
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.reduce_list(symbol)?;
            },
            SyntaxNode::Binop(_, n1, n2) | SyntaxNode::Block(n1, n2) => {
                n1.reduce_list(symbol)?;
                n2.reduce_list(symbol)?;
            },
            SyntaxNode::Unclassified(token) => return token.raise("Invalid syntax 11"),
            SyntaxNode::Number(_) => unreachable!()
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
                                nodes.drain(i..=i+1);
                                nodes[i-1] = new_node;
                                continue;
                            },
                            None => (),
                        }
                    } else {
                        nodes[i].reduce_binop(symbols)?;
                    }
                    i += 1;
                }
            }
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.reduce_binop(symbols)?;
            },
            SyntaxNode::Binop(_, n1, n2) | SyntaxNode::Block(n1, n2) => {
                n1.reduce_binop(symbols)?;
                n2.reduce_binop(symbols)?;
            },
            SyntaxNode::Unclassified(token) => return token.raise("Invalid syntax 12"),
            SyntaxNode::Number(_) => unreachable!()
        };
        Ok(())
    }

    fn reduce_total_binop(&mut self, symbols: &[&'static str]) -> Result<(), String> {
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
                                let suffix = SyntaxNode::Adjacent(nodes.drain(i+1..).collect::<Vec<_>>());
                                let prefix = SyntaxNode::Adjacent(nodes.drain(..i).collect::<Vec<_>>());
                                let new_node = SyntaxNode::Binop(op, Box::new(prefix), Box::new(suffix));
                                nodes[0] = new_node;
                                return Ok(());
                            },
                            None => (),
                        }
                    } else {
                        nodes[i].reduce_total_binop(symbols)?;
                    }
                    i += 1;
                }
            }
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.reduce_total_binop(symbols)?;
            },
            SyntaxNode::Binop(_, n1, n2) | SyntaxNode::Block(n1, n2) => {
                n1.reduce_total_binop(symbols)?;
                n2.reduce_total_binop(symbols)?;
            },
            SyntaxNode::Unclassified(token) => return token.raise("Invalid syntax 13"),
            SyntaxNode::Number(_) => unreachable!()
        };
        Ok(())
    }

    fn reduce_unop(&mut self, symbols: &[&'static str]) -> Result<(), String> {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                let mut i = 0;
                while i < nodes.len() {
                    if let Unclassified(t) = &nodes[i] {
                        match symbols.iter().position(|x| t == *x) {
                            Some(index) => {
                                let op = symbols[index];
                                if i == 0 {
                                    return t.raise("Unary operation encountered with no left side");
                                }
                                let new_node = SyntaxNode::Unop(op, Box::new(nodes[i-1].clone()));
                                nodes.remove(i);
                                nodes[i-1] = new_node;
                                continue;
                            },
                            None => (),
                        }
                    } else {
                        nodes[i].reduce_unop(symbols)?;
                    }
                    i += 1;
                }
            }
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.reduce_unop(symbols)?;
            },
            SyntaxNode::Binop(_, n1, n2) | SyntaxNode::Block(n1, n2) => {
                n1.reduce_unop(symbols)?;
                n2.reduce_unop(symbols)?;
            },
            SyntaxNode::Unclassified(token) => return token.raise("Invalid syntax 14"),
            SyntaxNode::Number(_) => unreachable!()
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
            SyntaxNode::Binop(_, n1, n2) | SyntaxNode::Block(n1, n2) => {
                n1.reduce_number();
                n2.reduce_number();
            },
            SyntaxNode::Unclassified(v) => {
                if let Ok(n) = v.s.parse::<f64>() {
                    let new_token = Token {
                        s: n,
                        filename: v.filename.clone(),
                        line_no: v.line_no,
                        col_no: v.col_no,
                    };
                    *self = SyntaxNode::Number(new_token);
                }
            },
            SyntaxNode::Number(_) => (),
        };
    }

    fn reduce_singletons(&mut self) {
        match self {
            SyntaxNode::Adjacent(nodes) => {
                if nodes.len() == 1 {
                    *self = nodes[0].clone();
                    self.reduce_singletons();
                } else {
                    for node in nodes {
                        node.reduce_singletons();
                    }
                }
            }
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => {
                n.reduce_singletons();
            },
            SyntaxNode::Binop(_, n1, n2) | SyntaxNode::Block(n1, n2) => {
                n1.reduce_singletons();
                n2.reduce_singletons();
            },
            SyntaxNode::Unclassified(_) | SyntaxNode::Number(_) => (),
        };
    }

    fn internal_raise<T>(&self, message: &str) -> Option<Result<T, String>> {
        match self {
            Unclassified(token) => Some(token.raise(message)),
            SyntaxNode::Number(token) => Some(token.raise(message)),
            SyntaxNode::Adjacent(syntax_nodes) => match syntax_nodes.first() {
                Some(n) => n.internal_raise(message),
                None => None,
            },
            SyntaxNode::Parenthesis(_, n) | SyntaxNode::List(_, n) | SyntaxNode::Unop(_, n) => n.internal_raise(message),
            SyntaxNode::Binop(_, n1, n2) | SyntaxNode::Block(n1, n2) => {
                match n1.internal_raise(message) {
                    Some(t) => Some(t),
                    None => n2.internal_raise(message),
                }
            },
        }
    }

    pub fn raise<T>(&self, message: &str) -> Result<T, String> {
        match self.internal_raise(message) {
            Some(r) => r,
            None => Err(format!("No file\n{}", message)),
        }
    }

    pub fn raise_str(&self, message: &str) -> String {
        self.raise::<()>(message).err().unwrap()
    }
    
    pub fn tree(tokens: Vec<Token<String>>) -> Result<SyntaxNode, String> {
        let mut tree = Self::Adjacent(tokens.into_iter().map(|t| SyntaxNode::Unclassified(t)).collect::<Vec<_>>());
        tree.reduce()?;
        Ok(tree)
    }
}

impl std::fmt::Debug for SyntaxNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unclassified(arg0) => write!(f, "`{:?}`", arg0),
            Self::Number(arg0) => write!(f, "Number({:?})", arg0),
            Self::Adjacent(arg0) => write!(f, "{:?}", arg0),
            Self::Parenthesis(op, arg1) => write!(f, "Parens `{}` ({:?})", op, arg1),
            Self::List(op, arg1) => write!(f, "List `{}` ({:?})", op, arg1),
            Self::Binop(op, arg1, arg2) => write!(f, "Binop `{}` ({:?}, {:?})", op, arg1, arg2),
            Self::Unop(op, arg1) => write!(f, "Unop `{}` ({:?})", op, arg1),
            Self::Block(arg0, arg1) => write!(f, "{{ {:?} | {:?} }}", arg0, arg1),
        }
    }
}

/// Load file, handling import statements
pub fn load_file(filename: &str) -> Result<Vec<Token<String>>, String> {
    let mut file = File::open(filename).map_err(|_| format!("Could not find file {}", filename))?;
    let mut text = "".to_owned();
    file.read_to_string(&mut text).map_err(|_| format!("Could not read file {}", filename))?;
    load_str(&text, filename)
}

/// Load string, handling import statements
pub fn load_str(text: &str, filename: &str) -> Result<Vec<Token<String>>, String> {
    let mut stream = get_stream(&text, filename)?;
    reduce_pragmas(&mut stream)?;
    Ok(stream)
}

/// Get a stream of all tokens
fn get_stream(text: &str, filename: &str) -> Result<Vec<Token<String>>, String> {
    let singletons = unsafe { SortedSet::from_sorted(vec!['\t', '\n', ' ', '"', '#', '\'', '(', ')', ',', ';', '[', ']', '{', '}']) };
    let specials = unsafe { SortedSet::from_sorted(vec!['!', '$', '%', '&', '*', '+', '-', '/', ':', '<', '=', '>', '?', '@', '\\', '^', '`', '|', '~']) };
    let mut tokens = Vec::new();
    let mut token = Vec::new();
    let mut token_is_special = false;
    let mut line_no = 0;
    let mut col_no = 0;
    for c in text.chars() {
        if !c.is_ascii(){
            return Err(format!("{}:{}:{}\nNon-ascii characters are not allowed", filename, line_no, col_no));
        }

        let c_is_special = specials.contains(&c);
        let c_is_singleton = singletons.contains(&c);
        // Push the existing token if necessary
        if !token.is_empty() {
            if c_is_singleton {
                tokens.push(Token {
                    s: token.drain(..).collect(),
                    filename: filename.to_owned(),
                    line_no,
                    col_no,
                });
            } else if token_is_special ^ c_is_special {
                // The token is special but c is not, or vice versa
                tokens.push(Token {
                    s: token.drain(..).collect(),
                    filename: filename.to_owned(),
                    line_no,
                    col_no,
                });
            }
        }
        if c != ' ' && c != '\t' {
            if c_is_singleton {
                tokens.push(Token {
                    s: format!("{}", c),
                    filename: filename.to_owned(),
                    line_no,
                    col_no,
                });
            } else {
                if token.is_empty() {
                    token_is_special = c_is_special;
                }
                token.push(c);
            }
        }
        col_no += 1;
        if c == '\n' {
            line_no += 1;
            col_no = 0;
        }
    }
    if !token.is_empty() {
        // Push the token immediately
        tokens.push(Token {
            s: token.drain(..).collect(),
            filename: filename.to_owned(),
            line_no,
            col_no,
        });
    } 
    Ok(tokens)
}

/// Reduce the pragmas in each file
fn reduce_pragmas(tokens: &mut Vec<Token<String>>) -> Result<(), String> {
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

                        // Load the file
                        let mut file = File::open(&lib_name).map_err(|_| format!("Could not find file {}", lib_name))?;
                        let mut text = "".to_owned();
                        file.read_to_string(&mut text).map_err(|_| format!("Could not read file {}", lib_name))?;
                        load_str(&text, &lib_name)
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