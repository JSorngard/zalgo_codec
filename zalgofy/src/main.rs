use zalgo_codec_common::{zalgo_encode};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Clone, Subcommand)]
enum Source {
    /// Operate on all text after the command
    Stdin {
        text: Vec<String>,
    },

    /// Operate on the contents of the file at the path given after the command
    File {
        path: PathBuf,
    },
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
    }
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

fn main() {
    let args = Cli::parse();
    println!("{args:?}");
}
