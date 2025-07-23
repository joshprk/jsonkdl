use crate::convert::{ConversionError, KdlVersion};
use crate::convert::{convert_and_write_file_content, convert_file_content};
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

const HELP_TEXT: &str = "\
Usage: jsonkdl [options] [--] <input> <output>
Converts JSON to KDL.
By default, KDL spec v2 is used.

Options:
  -1, --kdl-v1     Convert to KDL v1
  -2, --kdl-v2     Convert to KDL v2
  -f, --force      Overwrite output if it exists
  -v, --verbose    Print extra information during conversion
  -h, --help       Show this help message

Arguments:
  <input>          Path to input JSON file
  <output>         Path to output KDL file
";

#[derive(Debug)]
pub enum CliError {
    MissingInput,
    HelpRequested,
    MultipleKdlVersion,
    UnknownOption(String),
    TooManyPositionals,
    NotUnicode(OsString),
    InvalidInputPath(PathBuf),
    FileExists(PathBuf),
    InputNotFound(PathBuf),
    Conversion(ConversionError),
}

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug)]
pub struct Args {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
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
            CliError::UnknownOption(opt) => writeln!(
                f,
                "unknown command-line option {opt} (use `--` to pass arbitrary filenames)"
            ),
            CliError::TooManyPositionals => writeln!(f, "too many positional arguments"),
            CliError::NotUnicode(arg) => writeln!(
                f,
                "the argument {arg:?} was not valid Unicode (use `--` to pass arbitrary filenames)"
            ),
            CliError::InvalidInputPath(path) => writeln!(f, "not a file: {}", path.display()),
            CliError::FileExists(path) => {
                writeln!(
                    f,
                    "file exists: {} (use --force to overwrite)",
                    path.display()
                )
            }
            CliError::InputNotFound(path) => writeln!(f, "no such file: {}", path.display()),
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
        let args = env::args_os();

        if args.len() == 1 {
            return Err(CliError::HelpRequested);
        }

        let args = args.skip(1);

        let mut force = false;
        let mut verbose = false;
        let mut kdl_version = None;

        let mut positionals_only = false;

        let mut input = None;
        let mut output = None;

        for arg in args {
            let is_positional;

            if positionals_only {
                is_positional = true;
            } else {
                let Some(arg) = arg.to_str() else {
                    return Err(CliError::NotUnicode(arg));
                };

                if arg.starts_with("-") {
                    is_positional = false;
                    match arg {
                        "--" => positionals_only = true,
                        "-f" | "--force" => force = true,
                        "-v" | "--verbose" => verbose = true,
                        "-1" | "--kdl-v1" => {
                            if kdl_version.replace(KdlVersion::V1).is_some() {
                                return Err(CliError::MultipleKdlVersion);
                            }
                        }
                        "-2" | "--kdl-v2" => {
                            if kdl_version.replace(KdlVersion::V2).is_some() {
                                return Err(CliError::MultipleKdlVersion);
                            }
                        }
                        "-h" | "--help" => return Err(CliError::HelpRequested),
                        _ => return Err(CliError::UnknownOption(arg.to_string())),
                    }
                } else {
                    is_positional = true;
                }
            }

            if is_positional {
                if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else if output.is_none() {
                    output = Some(PathBuf::from(arg));
                } else {
                    return Err(CliError::TooManyPositionals);
                }
            }
        }

        let kdl_version = kdl_version.unwrap_or_default();

        let input = input.ok_or(CliError::MissingInput)?;

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

    if !args.input.exists() {
        return Err(CliError::InputNotFound(args.input));
    }

    if !args.input.is_file() {
        return Err(CliError::InvalidInputPath(args.input));
    }

    if let Some(output) = args.output {
        let output_path = Path::new(&output);

        if output_path.exists() && !args.force {
            return Err(CliError::FileExists(output));
        }

        convert_and_write_file_content(&args.input, output_path, args.verbose, args.kdl_version)?;
    } else {
        let kdl_content = convert_file_content(&args.input, args.kdl_version)?;

        println!("{}", kdl_content);
    }

    Ok(())
}
