use super::parser::Function;

pub enum Operation {
    
}

pub struct Ir {
    pub name: String,
    pub required_functions: Vec<String>,
    filename: String,
    start: u32,
    end: u32,
}

impl Ir {
    pub fn new(f: Function) -> Self {
        let mut required_functions = Vec::new();

        // TODO get name

        // TODO get IR

        Self {
            name,
            required_functions,
            filename: f.filename,
            start: f.start,
            end: f.end,
        }
    }
    
    pub fn bytecode(&self) -> Vec<u8> {
        unimplemented!()
    }
}
