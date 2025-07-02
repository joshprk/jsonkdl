use crate::convert::convert_file_contents;
use crate::convert::ConversionError;
use std::env;
use std::path::Path;

const HELP_TEXT: &str = "\
jsonkdl - convert json to kdl
Usage:
  jsonkdl <input> <output> [options]

Arguments:
  <input>          Path to input json file
  <output>         Path to output kdl file

Aptions:
  -f, --force      Overwrite output if it exists
  -v, --verbose    Print extra information during conversion
  -h, --help       Show this help message
";

#[derive(Debug)]
pub enum CliError {
    MissingInput,
    MissingOutput,
    HelpRequested,
    InvalidInputPath(String),
    FileExists(String),
    InputNotFound(String),
    Conversion(ConversionError),
}

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug)]
pub struct Args {
    pub input: String,
    pub output: String,
    pub force: bool,
    pub verbose: bool,
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::MissingInput => writeln!(f, "missing input path"),
            CliError::MissingOutput => writeln!(f, "missing output path"),
            CliError::HelpRequested => writeln!(f, "help requested"),
            CliError::InvalidInputPath(path) => writeln!(f, "not a file: {}", path),
            CliError::FileExists(path) => writeln!(f, "file exists: {} (use --force to overwrite)", path),
            CliError::InputNotFound(path) => writeln!(f, "no such file: {}", path),
            CliError::Conversion(e) => writeln!(f, "conversion error: {}", e),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CliError::Conversion(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ConversionError> for CliError {
    fn from(err: ConversionError) -> Self {
        CliError::Conversion(err)
    }
}

impl Args {
    fn parse() -> Result<Self> {
        let args: Vec<String> = env::args().collect();

        // If no args besides the program name, show help
        if args.len() == 1
            || args.iter().any(|arg| arg == "--help" || arg == "-h")
        {
            return Err(CliError::HelpRequested);
        }

        let force = args.iter().any(|arg| arg == "--force" || arg == "-f");
        let verbose = args.iter().any(|arg| arg == "--verbose" || arg == "-v");

        let positional: Vec<String> = args
            .iter()
            .skip(1)
            .filter(|arg| !arg.starts_with('-'))
            .cloned()
            .collect();

        let input = positional.get(0).ok_or(CliError::MissingInput)?.to_string();
        let output = positional.get(1).ok_or(CliError::MissingOutput)?.to_string();

        Ok(Self {
            input,
            output,
            force,
            verbose,
        })
    }
}

fn print_help() {
    print!("{}", HELP_TEXT);
}

pub fn run() -> Result<()> {
    let args = match Args::parse() {
        Ok(args) => args,
        Err(CliError::HelpRequested) => {
            print_help();
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let input_path = Path::new(&args.input);
    let output_path = Path::new(&args.output);

    if !input_path.exists() {
        return Err(CliError::InputNotFound(args.input));
    }

    if !input_path.is_file() {
        return Err(CliError::InvalidInputPath(args.input));
    }

    if output_path.exists() && !args.force {
        return Err(CliError::FileExists(args.output));
    }

    convert_file_contents(input_path, output_path, args.verbose)?;
    Ok(())
}
