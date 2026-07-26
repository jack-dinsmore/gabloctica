use rustc_hash::FxHashMap;
use sorted_vec::SortedVec;

use crate::{Command, bytecode::Function};

// Compile a string (usually read from a file) of Biscuit binary to assembly
pub fn disassemble_bytes(s: &[u8], _filename: &str) -> Result<String, String> {
    let mut iter = s.iter();
    let mut lines = Vec::new();
    let mut label_index = 0usize;
    let mut labels = FxHashMap::default();

    loop {
        let next = match iter.next() {
            Some(c) => *c,
            None => break,
        };
        let command = match Command::try_from(next) {
            Ok(c) => c,
            _ => return Err("Invalid code".to_owned())
        };

        let mut line = command.to_string().to_lowercase();
        match command {
            Command::Push => {
                let arg = f64::from_le_bytes([
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?
                ]);
                line = format!("\t{} {}", line, arg);
            },
            
            Command::Jmp | Command::Jnz=> {
                let label_pos = u64::from_le_bytes([
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?,
                    *iter.next().ok_or("Corrupted file")?
                ]) as usize;
                label_index += 1;
                labels.insert(label_pos, format!("label{}", label_index));
                line = format!("\t{} label{}", line, label_index);
            },
            Command::Call => {
                let function: Function = Function::try_from(*iter.next().ok_or("Corrupted file")?).unwrap();
                line = format!("\t{} {}", line, function.to_string().to_uppercase());
            },
            _ => ()
        };

        lines.push(line);
    }

    let keys = labels.keys().collect::<SortedVec<_>>();
    for key in keys.iter().rev() {
        lines.insert(**key, labels[key].clone());
    }
    let text = lines.join("\n");
    Ok(text)
}
