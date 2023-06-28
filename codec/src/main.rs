use clap::{Parser, Subcommand};
use std::path::PathBuf;
use zalgo_codec_common::{zalgo_decode, zalgo_encode, zalgo_wrap_python};

#[derive(Debug, Clone, Subcommand)]
enum Source {
    /// Operate on all text after the command
    Text { text: Vec<String> },

    /// Operate on the contents of the file at the path given after the command.
    /// Ignores carriage return characters
    File { path: PathBuf },
}

#[derive(Debug, Clone, Subcommand)]
enum Mode {
    /// Turn normal (printable ascii + newline) text into a single grapheme cluster
    Encode {
        #[command(subcommand)]
        source: Source,
    },

    /// Turn python code into a decoder wrapped around encoded source code.
    Wrap {
        /// The path to the file that is to be encoded. Ignores carriage return characters.
        file: PathBuf,
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
    /// If this is left unspecified the result is printed to stdout
    /// (not everything might appear visually, but it's still there).
    /// If your OS uses a text encoding other than UTF-8 (e.g. Windows uses UTF-16)
    /// you might want to use this option instead of an OS pipe to save to a file
    /// in order to avoid broken text. NOTE: If this option is used it must occur before any commands
    out_path: Option<PathBuf>,

    #[arg(short, long, required = false, requires = "out_path")]
    /// Overwrite the output file if it already exists.
    /// Only valid if OUT_PATH is also provided
    force: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Cli::parse();

    if let Some(ref destination) = config.out_path {
        if destination.exists() && !config.force {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "to overwrite the contents you can supply -f/--force",
            )));
        }
    }

    match config.mode {
        Mode::Encode { source } => {
            let text = match source {
                Source::Text { text } => text.join(" "),
                Source::File { path } => std::fs::read_to_string(path)?,
            };
            let encoded = zalgo_encode(&text)?;
            match config.out_path {
                Some(dst) => Ok(std::fs::write(dst, encoded)?),
                None => {
                    println!("{encoded}");
                    Ok(())
                }
            }
        }
        Mode::Wrap { file } => {
            let text = std::fs::read_to_string(file)?.replace("\r", "");
            let wrapped = zalgo_wrap_python(&text)?;
            match config.out_path {
                Some(dst) => Ok(std::fs::write(dst, wrapped)?),
                None => {
                    println!("{wrapped}");
                    Ok(())
                }
            }
        }
        Mode::Decode { source } => {
            let encoded = match source {
                Source::Text { mut text } => {
                    if text.len() == 1 {
                        text.swap_remove(0)
                    } else {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "can only decode one grapheme cluster at a time",
                        )));
                    }
                }
                Source::File { path } => std::fs::read_to_string(path)?.replace('\r', ""),
            };

            let decoded = zalgo_decode(&encoded)?;

            match config.out_path {
                Some(dst) => Ok(std::fs::write(dst, decoded)?),
                None => {
                    println!("{decoded}");
                    Ok(())
                }
            }
        }
    }
}
