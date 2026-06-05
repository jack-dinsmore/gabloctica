use rustc_hash::FxHashMap;

const COMMANDS: [&'static str; 29] = [
    "nop",      // No operation
    "push",     // Push to stack
    "pop",      // Pop from stack
    "dup",      // Copy top of stack
    "puship",   // Push the instruction pointer
    "jpop",     // Pop top to the  instruction pointer
    
    "jmp",      // Unconditional jump
    "jnz",      // Jump if not equal to zero
    "lt",       // Less than
    "gt",       // Greater than
    "le",       // Less than or equal to
    "ge",       // Greater than or equal to
    "eq",       // Equals

    "add",      // Float Add
    "sub",      // Float Subtract
    "mul",      // Float Multiply
    "div",      // Float Divide
    "neg",      // Float Negate
    "pow",      // Float Exponentiate

    "and",      // Boolean and
    "or",       // Boolean or
    "xor",      // Boolean exclusive or
    "not",      // Boolean not

    "popn",     // Pop next item
    "dupn",     // Duplicate next item
    
    "call",     // Call a function
    "tick",     // Tick
    "irp",      // Get interrupt
    "swp",      // Swap
];

const LIT_ARG: [&'static str; 1] = [
    "push",
];
const INT_ARG: [&'static str; 1] = [
    "call"
];
const LABEL_ARG: [&'static str; 2] = [
    "jmp",
    "jnz",
];

pub fn compile(text: &str) -> Vec<u8> {
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
            let command = &line[..index];
            match COMMANDS.iter().position(|&v| v==command) {
                Some(index) => {
                    output.push(index as u8);
                },
                None => panic!("Line {}: Command {} not recognized", line_number+1, command),
            }

            // Handle the argument
            let arg = &line[(index+1)..];
            if LIT_ARG.contains(&command) {
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
            } else if INT_ARG.contains(&command) {
                // Push an unsigned integer
                if let Ok(d) = arg.parse::<u64>() {
                    output.extend(d.to_le_bytes());
                }
                else {
                    panic!("Line {}: Command {} take an integer argument, but one was not passed", line_number+1, command);
                }
            } else if LABEL_ARG.contains(&command) {
                // Push a label
                label_positions.push((output.len(), arg.to_owned()));
                output.extend([0,0,0,0,0,0,0,0]);
            }
        } else {
            // Handle a unary command
            match COMMANDS.iter().position(|&v| v==line) {
                Some(index) => {
                    output.push(index as u8);
                },
                None => panic!("Line {}: Command {} not recognized", line_number+1, line),
            };
            if LABEL_ARG.contains(&line) {
                panic!("Line {}: Command {} takes a label argument, but one was not passed", line_number+1, line);
            };
            if LIT_ARG.contains(&line) {
                panic!("Line {}: Command {} takes a literal argument, but one was not passed", line_number+1, line);
            };
            if INT_ARG.contains(&line) {
                panic!("Line {}: Command {} takes an integer argument, but one was not passed", line_number+1, line);
            };
        }
    }

    // Replace the labels
    for (pos, label) in label_positions {
        let instruction_number = match labels.get(&label) {
            Some(l) => *l as u64,
            None => panic!("Could not find label {}", label)
        };
        for (i, byte) in instruction_number.to_le_bytes().into_iter().enumerate() {
            output[pos + i] = byte;
        }
    }

    output
}