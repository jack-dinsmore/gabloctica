use std::path::{Path, PathBuf};

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

    let result = match cli.command {
        Command::Build(args) => args.run(),
        Command::Run(args) => args.run(),
        Command::Asm(args) => args.run(),
        Command::Dis(args) => args.run(),
    };
    if let Err(message) = result {
        println!("Error:\n{}", message);
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
    fn run(self) -> Result<(), String> {
        let input = Path::new(&self.input);
        let output = match &self.output {
            Some(v) => PathBuf::from(v),
            None => input.with_extension("b"),
        };

        let bytes = biscuit::compile_file(input.to_string_lossy().as_ref())?;

        std::fs::write(output, bytes).map_err(|_| "Could not write output file".to_owned())?;

        Ok(())
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
    fn run(self) -> Result<(), String> {
        let input = Path::new(&self.input);
        let output = match &self.output {
            Some(v) => PathBuf::from(v),
            None => input.with_extension("b"),
        };

        let bytes = biscuit::assemble_file(input.to_string_lossy().as_ref())?;

        std::fs::write(output, bytes).map_err(|_| "Could not write output file".to_owned())?;

        Ok(())
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
    fn run(self) -> Result<(), String> {
        let input = Path::new(&self.input);
        let output = match &self.output {
            Some(v) => PathBuf::from(v),
            None => input.with_extension("basm"),
        };

        let bytes = biscuit::disassemble_file(input.to_string_lossy().as_ref())?;

        std::fs::write(output, bytes).map_err(|_| "Could not write output file".to_owned())?;

        Ok(())
    }
}

#[derive(Args)]
struct Run {

}
impl Run {
    fn run(self) -> Result<(), String> {
        println!("run");
        Ok(())
    }
}