use zalgo_codec_common::{zalgo_encode};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Clone)]
enum Source {
    Stdin,
    File
}

#[derive(Debug, Clone)]
enum Action {
    Encode,
    Decode,
}

#[derive(Debug, Clone, Subcommand)]
enum Mode {
    /// Encode text read from stdin and print it to stdout
    EncodeStdin {
        /// The text to encode
        text: Vec<String>,
    },
    
    /// Encode the contents of a file
    EncodeFile {
        /// The path to the file to be encoded
        in_path: PathBuf,
    },
    
    /// Decode text from stdin and print it to stdout
    DecodeStdin {
        /// The text to decode
        text: Vec<String>,
    },
    
    /// Decode the contents of a file
    DecodeFile {
        /// The path to the encoded file
        in_path: PathBuf,
    },
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,

    /// An optional path to a location where the result should be saved.
    /// If this is left unspecified the result is printed to stdout.
    /// If your OS uses a text encoding other than UTF-8 (e.g. Windows uses UTF-16)
    /// you might want to use this option instead of an OS pipe in order to avoid broken text
    out_path: Option<PathBuf>,
}

fn main() {
    let args = Cli::parse();
    println!("{args:?}");
}
