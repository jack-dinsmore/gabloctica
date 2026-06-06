/// This mod compiles quahog code into stack machine bytecode.
mod parser;
mod ir;
mod compiler;

use parser::Function;
use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;

pub fn compile_file(filename: &str) -> Result<Vec<u8>, String> {
    let functions = parser::parse_file(filename)?;
    process_code(functions, filename)
}

pub fn compile_str(s: &str, filename: &str) -> Result<Vec<u8>, String> {
    let functions = parser::parse_str(s, filename)?;
    process_code(functions, filename)
}

fn process_code(functions: Vec<Function>, filename: &str) -> Result<Vec<u8>, String> {
    let mut irs = functions.into_iter().map(|f| ir::Ir::new(f)).collect::<Vec<_>>();
    
    // Put main first
    for i in 0..irs.len() {
        if irs[i].name == "main" {
            irs.swap(0, i);
            break;
        }
    }

    // Check for duplicates
    let mut function_locations = FxHashMap::default();
    for (i, ir) in irs.iter().enumerate() {
        if function_locations.contains_key(&ir.name) {
            return Err(format!("{}\nDuplicate functions named `{}`", filename, ir.name));
        }
        function_locations.insert(ir.name.to_owned(), i);
    }

    // Check dependencies
    let mut necessaries = vec!["main".to_owned()];
    let mut visited = SortedSet::new();
    let mut queue = vec!["main".to_owned()];
    while !queue.is_empty() {
        let name = queue.pop().unwrap();
        if visited.contains(&name) {continue;}
        visited.push(name.to_owned());
        match function_locations.get(&name) {
            Some(i) => {
                let function = &irs[*i];
                for name in &function.required_functions {
                    queue.push(name.to_owned());
                    necessaries.push(name.to_owned());
                }
            },
            None => return Err(format!("{}\nThe function `{}` was not found", filename, name))
        };
    }

    // Remove all unrequired functions
    let mut i = 0;
    while i < irs.len() {
        if !necessaries.contains(&irs[i].name) {
            irs.swap_remove(i);
            continue;
        }
        i += 1;
    }

    // Optimize
    for ir in &mut irs {
        compiler::optimize(ir);
    }

    // Write to bytecode
    let mut bytecode = Vec::new();
    for ir in irs {
        bytecode.append(&mut ir.bytecode());
    }
    Ok(bytecode)
}