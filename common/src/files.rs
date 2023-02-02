use super::*;

    use std::{
        fs, io,
        path::{Path, PathBuf},
    };

    use core::str::Utf8Error;

    /// Encodes the contents of the file and stores the result in another file.
    /// If carriage return characters are found it will print a message and
    /// attempt to encode the file anyway by ignoring them.
    pub fn encode_file<P: AsRef<Path>>(
        in_file: P,
        out_file: P,
    ) -> Result<(), UnencodableFileError> {
        let mut string_to_encode = fs::read_to_string(in_file)?;

        if string_to_encode.contains('\t') {
            eprintln!("found tabs in the file, replacing with four spaces");
            string_to_encode = string_to_encode.replace('\t', "    ");
        }

        if string_to_encode.contains('\r') {
            eprintln!(
                r"file contains the carriage return character (\r). Will attempt to encode the file anyway by ignoring it. This may result in a different file when decoded"
            );
            string_to_encode = string_to_encode.replace('\r', "");
        }

        let mut out_path = PathBuf::new();
        out_path.push(&out_file);

        if out_path.exists() {
            return Err(UnencodableFileError::OutputFileExists);
        }

        fs::File::create(&out_file)?;
        fs::write(out_file, zalgo_encode(&string_to_encode)?)?;
        Ok(())
    }

    /// Decodes the contents of a file that has been encoded with [`encode_file`]
    /// and stores the result in another file.
    pub fn decode_file<P: AsRef<Path>>(
        in_file: P,
        out_file: P,
    ) -> Result<(), UndecodableFileError> {
        let mut string_to_decode = fs::read_to_string(in_file)?;

        if string_to_decode.contains('\r') {
            eprintln!(
                r"file contains the carriage return character (\r). Will attempt to decode the file anyway by ignoring it"
            );
            string_to_decode = string_to_decode.replace('\r', "");
        }

        let decoded_string = zalgo_decode(&string_to_decode)?;

        let mut out_path = PathBuf::new();
        out_path.push(&out_file);

        if out_path.exists() {
            return Err(UndecodableFileError::OutputFileExists);
        }

        fs::File::create(&out_file)?;
        fs::write(out_file, decoded_string)?;
        Ok(())
    }

    /// Encodes the contents of the given file and stores the result wrapped in
    /// a decoder in another file. This file will still work the same
    /// as the original python code. If the source file contains carriage return characters
    /// this function will print a message and then attempt to encode the file anyway by ignoring them.
    /// # Notes
    /// The resulting python file may not work correctly on python versions before 3.10,
    /// (see [this github issue](https://github.com/DaCoolOne/DumbIdeas/issues/1)).
    pub fn encode_python_file<P: AsRef<Path>>(
        in_file: P,
        out_file: P,
    ) -> Result<(), UnencodableFileError> {
        let mut string_to_encode = fs::read_to_string(in_file)?;

        if string_to_encode.contains('\t') {
            eprintln!("found tabs in the file, replacing with four spaces");
            string_to_encode = string_to_encode.replace('\t', "    ");
        }

        if string_to_encode.contains('\r') {
            eprintln!(
                r"file contains the carriage return character (\r). Will attempt to encode the file anyway by ignoring it. This may result in a different file when decoded"
            );
            string_to_encode = string_to_encode.replace('\r', "");
        }

        let mut out_path = PathBuf::new();
        out_path.push(&out_file);

        if out_path.exists() {
            return Err(UnencodableFileError::OutputFileExists);
        }

        fs::File::create(&out_file)?;
        fs::write(out_file, zalgo_wrap_python(&string_to_encode)?)?;
        Ok(())
    }

    /// The error returned by the encoding functions that
    /// interact with the file system.
    #[derive(Debug)]
    pub enum UnencodableFileError {
        Io(io::Error),
        OutputFileExists,
        UnencodableContent(UnencodableByteError),
    }

    impl fmt::Display for UnencodableFileError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Io(e) => write!(f, "{e}"),
                Self::UnencodableContent(e) => write!(f, "{e}"),
                Self::OutputFileExists => {
                    write!(f, "a file with the given output name already exists")
                }
            }
        }
    }

    impl Error for UnencodableFileError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::Io(e) => Some(e),
                Self::UnencodableContent(e) => Some(e),
                Self::OutputFileExists => None,
            }
        }
    }

    impl From<io::Error> for UnencodableFileError {
        fn from(err: io::Error) -> Self {
            Self::Io(err)
        }
    }

    impl From<UnencodableByteError> for UnencodableFileError {
        fn from(err: UnencodableByteError) -> Self {
            Self::UnencodableContent(err)
        }
    }

    /// The error returned by the decoding functions that
    /// interact with the file system.
    #[derive(Debug)]
    pub enum UndecodableFileError {
        Io(io::Error),
        OutputFileExists,
        DecodesToInvalidUnicode(Utf8Error),
    }

    impl fmt::Display for UndecodableFileError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Io(e) => write!(f, "{e}"),
                Self::DecodesToInvalidUnicode(e) => write!(f, "{e}"),
                Self::OutputFileExists => {
                    write!(f, "a file with the given output name already exists")
                }
            }
        }
    }

    impl Error for UndecodableFileError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::Io(e) => Some(e),
                Self::DecodesToInvalidUnicode(e) => Some(e),
                Self::OutputFileExists => None,
            }
        }
    }

    impl From<io::Error> for UndecodableFileError {
        fn from(err: io::Error) -> Self {
            Self::Io(err)
        }
    }

    impl From<Utf8Error> for UndecodableFileError {
        fn from(err: Utf8Error) -> Self {
            Self::DecodesToInvalidUnicode(err)
        }
    }