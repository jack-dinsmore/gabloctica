use rustc_hash::FxHashMap;
use sorted_vec::SortedVec;

use crate::{Command, bytecode::GlobalFunction};

// Compile a string (usually read from a file) of Biscuit binary to assembly
pub fn disassemble_bytes(s: &[u8], _filename: &str) -> Result<String, String> {
    let mut iter = s.iter().enumerate();
    let mut lines = Vec::new();
    let mut label_index = 0usize;
    let mut labels = FxHashMap::default();
    let mut instruction_to_line_map = FxHashMap::default();

    loop {
        let (index, next) = match iter.next() {
            Some(t) => t,
            None => break,
        };
        instruction_to_line_map.insert(index, lines.len());

        let command = match Command::try_from(*next) {
            Ok(c) => c,
            _ => return Err("Invalid code".to_owned())
        };

        let mut line = command.to_string().to_lowercase();
        match command {
            Command::Push => {
                let mut arg = f64::from_le_bytes([
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1
                ]);
                if (arg - arg.round()).abs() < 1e-10 {
                    arg = arg.round();
                }
                line = format!("{} {}", line, arg);
            },
            
            Command::Jmp | Command::Jnz=> {
                let label_pos = u64::from_le_bytes([
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1,
                    *iter.next().ok_or("Corrupted file")?.1
                ]) as usize;
                label_index += 1;
                labels.insert(label_pos, format!("label{}", label_index));
                line = format!("{} label{}", line, label_index);
            },
            Command::Call => {
                let function = GlobalFunction::try_from(*iter.next().ok_or("Corrupted file")?.1).unwrap();
                line = format!("{} {}", line, function.to_string().to_uppercase());
            },
            _ => ()
        };

        lines.push(format!("\t{}", line));
    }


    let instruction_nos = labels.keys().collect::<SortedVec<_>>();
    for instruction_no in instruction_nos.iter().rev() {
        let line_no = instruction_to_line_map.get(*instruction_no).unwrap();
        lines.insert(*line_no, labels[instruction_no].clone());
    }
    let text = lines.join("\n");
    Ok(text)
}
