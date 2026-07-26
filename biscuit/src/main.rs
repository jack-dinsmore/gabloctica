use std::path::Path;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "biscuit", version, about = "Interface with biscuit code")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build the project
    Build(Build),
    /// Run the debugger
    Run(Run),
    /// Assemble an assembly code
    Asm(Asm),
    /// Disassemble the output binary code
    Dis(Dis),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Build(args) => args.run(),
        Command::Run(args) => args.run(),
        Command::Asm(args) => args.run(),
        Command::Dis(args) => args.run(),
    }
}


#[derive(Args)]
struct Build {
    #[arg()]
    input: String,

    #[arg(short, long)]
    output: Option<String>,
}
impl Build {
    fn run(self) {
        let input = Path::new(&self.input);
        let output = Path::new(&match self.output {
            Some(v) => v,
            None => format!("{}.b", input.file_stem().unwrap().to_string_lossy()),
        });

        let bytes = biscuit::compile_file(input.to_string_lossy().as_ref());

        std::fs::write(output, bytes).unwrap();
    }
}


#[derive(Args)]
struct Asm {
    #[arg()]
    input: String,

    #[arg(short, long)]
    output: Option<String>,
}
impl Asm {
    fn run(self) {
        let input = Path::new(&self.input);
        let output = Path::new(&match self.output {
            Some(v) => v,
            None => format!("{}.b", input.file_stem().unwrap().to_string_lossy()),
        });

        let bytes = biscuit::assemble_file(input.to_string_lossy().as_ref());

        std::fs::write(output, bytes).unwrap();
    }
}


#[derive(Args)]
struct Dis {
    #[arg()]
    input: String,

    #[arg(short, long)]
    output: Option<String>,
}
impl Dis {
    fn run(self) {
        let input = Path::new(&self.input);
        let output = Path::new(&match self.output {
            Some(v) => v,
            None => format!("{}.basm", input.file_stem().unwrap().to_string_lossy()),
        });

        let bytes = biscuit::disassemble_file(input.to_string_lossy().as_ref());

        std::fs::write(output, bytes).unwrap();
    }
}

#[derive(Args)]
struct Run {
}
impl Run {
    fn run(self) {
        println!("run");
    }
}