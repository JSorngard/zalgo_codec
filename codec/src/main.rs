#[cfg(feature = "gui")]
mod gui;

use std::path::PathBuf;

use zalgo_codec_common::{zalgo_decode, zalgo_encode, zalgo_wrap_python};

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

#[derive(Debug, Clone, Subcommand)]
enum Source {
    /// Operate on all text after the command.
    Text { text: Vec<String> },

    /// Operate on the contents of the file at the path given after the command.
    /// Ignores carriage return characters.
    File { path: PathBuf },
}

#[derive(Debug, Clone, Subcommand)]
enum Mode {
    #[cfg(feature = "gui")]
    /// Opens up a rudimentary GUI application where you can apply the functions of the codec to text
    /// entered through a text box as well as copy the results or save them to a file.
    /// It is currently not possible to enter newlines into the text box.
    /// Overrides all other options.
    Gui,

    /// Turn normal (printable ascii + newline) text into a single grapheme cluster.
    Encode {
        #[command(subcommand)]
        source: Source,
    },

    /// Turn python code into a decoder wrapped around encoded source code.
    Wrap {
        /// The path to the file that is to be encoded. Ignores carriage return characters.
        path: PathBuf,
    },

    /// Turn text that has been encoded back into its normal form.
    Decode {
        #[command(subcommand)]
        source: Source,
    },

    /// Unwrap and decode a wrapped python file.
    Unwrap {
        /// The path to the file to unwrap and decode.
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,

    #[arg(short, long)]
    /// An optional path to a location where the result should be saved.
    /// If this is left unspecified the result is printed to stdout
    /// (not everything might appear visually, but it's still there).
    /// If your OS uses a text encoding other than UTF-8 (e.g. Windows uses UTF-16)
    /// you might want to use this option instead of an OS pipe to save to a file
    /// in order to avoid broken text. NOTE: If this option is used it must occur before any commands.
    out_path: Option<PathBuf>,

    #[arg(short, long, required = false, requires = "out_path")]
    /// Overwrite the output file if it already exists.
    /// Only valid if OUT_PATH is also provided
    force: bool,
}

fn main() -> Result<()> {
    let config = Cli::parse();

    if let Some(ref destination) = config.out_path {
        if destination.exists() && !config.force {
            match config.mode {
                #[cfg(feature = "gui")]
                Mode::Gui => (),
                _ => return Err(anyhow!("the file \"{}\" already exists, to overwrite its contents you can supply the -f or --force arguments", destination.to_string_lossy())),
            }
        }
    }

    let output = match config.mode {
        #[cfg(feature = "gui")]
        Mode::Gui => gui::run_gui(),
        Mode::Encode { source } => {
            let text = match source {
                Source::Text { text } => text.join(" "),
                Source::File { path } => std::fs::read_to_string(path)?.replace('\r', ""),
            };
            zalgo_encode(&text)?
        }
        Mode::Wrap { path } => {
            let text = std::fs::read_to_string(path)?.replace('\r', "");
            zalgo_wrap_python(&text)?
        }
        Mode::Decode { source } => {
            let encoded = match source {
                Source::Text { mut text } => {
                    if text.len() == 1 {
                        Ok(text.swap_remove(0))
                    } else {
                        Err(anyhow!("can only decode one grapheme cluster at a time"))
                    }?
                }
                Source::File { path } => std::fs::read_to_string(path)?.replace('\r', ""),
            };

            zalgo_decode(&encoded)?
        }
        Mode::Unwrap { path } => {
            let contents = std::fs::read_to_string(path)?;
            let mut chars = contents.chars();
            for _ in 0..3 {
                chars.next();
            }
            for _ in 0..88 {
                chars.next_back();
            }
            let encoded: String = chars.collect();
            zalgo_decode(&encoded)?
        }
    };

    match config.out_path {
        Some(dst) => Ok(std::fs::write(dst, output)?),
        None => {
            println!("{output}");
            Ok(())
        }
    }
}
