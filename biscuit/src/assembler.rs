use rustc_hash::FxHashMap;

use crate::{Command, bytecode::Function};

pub fn assemble_str(text: &str, _filename: &str) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut labels = FxHashMap::default();
    let mut label_positions = Vec::new();
    for (line_number, line) in text.split('\n').enumerate() {
        if line.starts_with("#") {continue;}
        let line = match line.find('#') {
            Some(i) => &line[..i],
            None => line,
        };
        let line = line.trim_ascii();
        if line.is_empty() {continue;}

        if let Some(index) = line.find(':') {
            let label = line[..index].to_owned();
            labels.insert(label, output.len());
        } else if let Some(index) = line.find(' ') {
            // Handle the command
            let command = match Command::from_string(&line[..index]) {
                Some(c) => c,
                None => return Err(format!("Line {}: Command {} not recognized", line_number+1, &line[..index])),
            };
            output.push(command as u8);

            // Handle the argument
            let arg = &line[(index+1)..];
            if command.takes_lit_arg() {
                if let Ok(d) = arg.parse::<i64>() {
                    // Push an integer
                    output.extend((d as f64).to_le_bytes());
                } else if let Ok(d) = arg.parse::<f64>() {
                    // Push a float
                    output.extend(d.to_le_bytes());
                } else {
                    // Push a label
                    label_positions.push((output.len(), arg.to_owned()));
                    output.extend([0,0,0,0,0,0,0,0]);
                }
            } else if command.takes_func_arg() {
                // Push an unsigned integer
                let func = Function::from_string(arg).ok_or(format!("Line {}: Function {} not recognized", line_number+1, arg))?;
                output.push(func as u8);
            } else if command.takes_label_arg() {
                // Push a label
                label_positions.push((output.len(), arg.to_owned()));
                output.extend([0,0,0,0,0,0,0,0]);
            }
        } else {
            // Handle a unary command
            let command = Command::from_string(line).ok_or(format!("Line {}: Command {} not recognized", line_number+1, line))?;
            output.push(command as u8);

            if command.takes_label_arg() {
                return Err(format!("Line {}: Command {} takes a label argument, but one was not passed", line_number+1, line));
            };
            if command.takes_lit_arg() {
                return Err(format!("Line {}: Command {} takes a literal argument, but one was not passed", line_number+1, line));
            };
            if command.takes_func_arg() {
                return Err(format!("Line {}: Command {} takes a function argument, but one was not passed", line_number+1, line));
            };
        }
    }

    // Replace the labels
    for (pos, label) in label_positions {
        let instruction_number = match labels.get(&label) {
            Some(l) => *l as u64,
            None => return Err(format!("Could not find label {}", label))
        };
        for (i, byte) in instruction_number.to_le_bytes().into_iter().enumerate() {
            output[pos + i] = byte;
        }
    }

    Ok(output)
}