use clap::{Parser, Subcommand};
use std::path::PathBuf;
use zalgo_codec_common::{encode_file, zalgo_encode};

#[derive(Debug, Clone, Subcommand)]
enum Source {
    /// Operate on all text after the command
    Stdin { text: Vec<String> },

    /// Operate on the contents of the file at the path given after the command
    File { path: PathBuf },
}

impl TryInto<String> for Source {
    type Error = std::io::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            Source::Stdin { text } => Ok(text.join(" ")),
            Source::File { path } => std::fs::read_to_string(path),
        }
    }
}

#[derive(Debug, Clone, Subcommand)]
enum Mode {
    /// Turn normal (printable ascii + newline) text into zalgo-encoded text
    Encode {
        #[command(subcommand)]
        source: Source,
    },

    /// Turn text that has been encoded back into its normal form
    Decode {
        #[command(subcommand)]
        source: Source,
    },
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,

    #[arg(short, long)]
    /// An optional path to a location where the result should be saved.
    /// If this is left unspecified the result is printed to stdout.
    /// If your OS uses a text encoding other than UTF-8 (e.g. Windows uses UTF-16)
    /// you might want to use this option instead of an OS pipe to save to a file
    /// in order to avoid broken text. If this option is used it must occur before any commands
    out_path: Option<PathBuf>,

    #[arg(short, long, required = false, requires = "out_path")]
    /// Overwrite the output file if it already exists.
    /// Only valid if OUT_PATH is also provided
    force: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Cli::parse();
    println!("{config:?}");

    if let Some(ref destination) = config.out_path {
        if destination.exists() && !config.force {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "to overwrite the contents you can supply -f/--force",
            )));
        }
    }

    match config.mode {
        Mode::Encode { source } => match source {
            Source::Stdin { text } => {
                let text = text.join(" ");
                let result = zalgo_encode(&text)?;
                match config.out_path {
                    Some(dst) => Ok(std::fs::write(dst, result)?),
                    None => {
                        println!("{result}");
                        Ok(())
                    }
                }
            }
            Source::File { path } => match config.out_path {
                Some(dst) => Ok(encode_file(path, dst)?),
                None => {
                    let text = std::fs::read_to_string(path)?;
                    let result = zalgo_encode(&text)?;
                    println!("{result}");
                    Ok(())
                }
            },
        },
        Mode::Decode { source } => {
            unimplemented!()
        }
    }
}
