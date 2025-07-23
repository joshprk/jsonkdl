use crate::convert::{ConversionError, KdlVersion};
use crate::convert::{convert_and_write_file_content, convert_file_content};
use std::env;
use std::path::Path;

const HELP_TEXT: &str = "\
Usage: jsonkdl <input> <output> [options]
Converts JSON to KDL.
By default, KDL spec v2 is used.

Arguments:
  <input>          Path to input JSON file
  <output>         Path to output KDL file

Options:
  -1, --kdl-v1     Convert to KDL v1
  -2, --kdl-v2     Convert to KDL v2
  -f, --force      Overwrite output if it exists
  -v, --verbose    Print extra information during conversion
  -h, --help       Show this help message
";

#[derive(Debug)]
pub enum CliError {
    MissingInput,
    HelpRequested,
    MultipleKdlVersion,
    UnknownOption(String),
    InvalidInputPath(String),
    FileExists(String),
    InputNotFound(String),
    Conversion(ConversionError),
}

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug)]
pub struct Args {
    pub input: String,
    pub output: Option<String>,
    pub force: bool,
    pub verbose: bool,
    pub kdl_version: KdlVersion,
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::MissingInput => writeln!(f, "missing input path"),
            CliError::HelpRequested => writeln!(f, "help requested"),
            CliError::MultipleKdlVersion => writeln!(f, "specify only one of --kdl-v1 or --kdl-v2"),
            CliError::UnknownOption(opt) => writeln!(f, "unknown command-line option {opt}"),
            CliError::InvalidInputPath(path) => writeln!(f, "not a file: {}", path),
            CliError::FileExists(path) => {
                writeln!(f, "file exists: {} (use --force to overwrite)", path)
            }
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
        let mut positional: Vec<String> = vec![];
        let mut force = false;
        let mut verbose = false;
        let mut kdl_version = None;

        if args.len() == 1 {
            return Err(CliError::HelpRequested);
        }

        for arg in args.iter().skip(1) {
            if !arg.starts_with('-') {
                positional.push(arg.into());
            } else if arg == "-f" || arg == "--force" {
                force = true;
            } else if arg == "-v" || arg == "--verbose" {
                verbose = true;
            } else if arg == "-1" || arg == "--kdl-v1" {
                if kdl_version.replace(KdlVersion::V1).is_some() {
                    return Err(CliError::MultipleKdlVersion);
                }
            } else if arg == "-2" || arg == "--kdl-v2" {
                if kdl_version.replace(KdlVersion::V2).is_some() {
                    return Err(CliError::MultipleKdlVersion);
                }
            } else if arg == "-h" || arg == "--help" {
                return Err(CliError::HelpRequested);
            } else {
                return Err(CliError::UnknownOption(arg.into()));
            }
        }

        let kdl_version = kdl_version.unwrap_or_default();

        let input = positional.get(0).ok_or(CliError::MissingInput)?.to_string();

        let output = positional.get(1).map(|s| s.to_string());

        let result = Self {
            input,
            output,
            force,
            verbose,
            kdl_version,
        };

        Ok(result)
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

    if !input_path.exists() {
        return Err(CliError::InputNotFound(args.input));
    }

    if !input_path.is_file() {
        return Err(CliError::InvalidInputPath(args.input));
    }

    if let Some(output) = args.output {
        let output_path = Path::new(&output);

        if output_path.exists() && !args.force {
            return Err(CliError::FileExists(output));
        }

        convert_and_write_file_content(input_path, output_path, args.verbose, args.kdl_version)?;
    } else {
        let kdl_content = convert_file_content(input_path, args.kdl_version)?;

        println!("{}", kdl_content);
    }

    Ok(())
}
