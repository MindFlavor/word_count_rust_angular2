use std::io;
use std::sync::mpsc;
use std::convert::From;

use processor::word_counter::WordCounter;

#[allow(enum_variant_names)]
#[derive(Debug)]
pub enum ProcessFileError {
    IOError(io::Error),
    SendError(mpsc::SendError<String>),
    RelinquishError(mpsc::SendError<WordCounter>),
    RecvError(mpsc::RecvError),
}

impl From<io::Error> for ProcessFileError {
    fn from(err: io::Error) -> ProcessFileError {
        ProcessFileError::IOError(err)
    }
}

impl From<mpsc::SendError<WordCounter>> for ProcessFileError {
    fn from(err: mpsc::SendError<WordCounter>) -> ProcessFileError {
        ProcessFileError::RelinquishError(err)
    }
}

impl From<mpsc::RecvError> for ProcessFileError {
    fn from(err: mpsc::RecvError) -> ProcessFileError {
        ProcessFileError::RecvError(err)
    }
}

impl From<mpsc::SendError<String>> for ProcessFileError {
    fn from(err: mpsc::SendError<String>) -> ProcessFileError {
        ProcessFileError::SendError(err)
    }
}

#[allow(enum_variant_names)]
#[derive(Debug)]
pub enum LoadFromFileError {
    IOError(io::Error),
    FormatError(String),
}

impl From<io::Error> for LoadFromFileError {
    fn from(err: io::Error) -> LoadFromFileError {
        LoadFromFileError::IOError(err)
    }
}
