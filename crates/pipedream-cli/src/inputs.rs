//! Command input files and standard input, read through an injected boundary.

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use pipedream_connect_types::ConfiguredProps;
use serde_json::error::Category;

use crate::argument::Argument;
use crate::error::{Error, JsonProblem, Result};

/// Where a command reads one input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum InputSource {
    /// Standard input, selected with `-`.
    Stdin,
    /// A file path.
    File(PathBuf),
}

impl From<PathBuf> for InputSource {
    fn from(path: PathBuf) -> Self {
        if path == Path::new("-") {
            Self::Stdin
        } else {
            Self::File(path)
        }
    }
}

/// Reads input sources, so that tests never touch the disk or standard input.
#[cfg_attr(test, unimock::unimock(api = [ReadInputMock]))]
pub(crate) trait Inputs: Send + Sync {
    /// Reads every byte of `source`.
    fn read(&self, source: &InputSource) -> io::Result<Vec<u8>>;
}

/// Reads files from the disk and standard input from the process.
pub(crate) struct FileInputs;

impl Inputs for FileInputs {
    fn read(&self, source: &InputSource) -> io::Result<Vec<u8>> {
        match source {
            InputSource::Stdin => {
                let mut bytes = Vec::new();
                io::stdin().read_to_end(&mut bytes)?;
                Ok(bytes)
            }
            InputSource::File(path) => fs::read(path),
        }
    }
}

/// Reads configured props JSON for `argument`.
///
/// Errors keep only the failure position, with lines and columns counted from 1.
pub(crate) fn read_props(
    inputs: &dyn Inputs,
    source: &InputSource,
    argument: Argument,
) -> Result<ConfiguredProps> {
    let bytes = match inputs.read(source) {
        Ok(bytes) => bytes,
        Err(cause) => return Err(Error::UnreadableInput { argument, cause }),
    };
    match serde_json::from_slice(&bytes) {
        Ok(props) => Ok(props),
        Err(error) => Err(Error::InvalidJson {
            argument,
            line: error.line(),
            column: error.column().max(1),
            problem: match error.classify() {
                Category::Data => JsonProblem::Shape,
                Category::Io | Category::Syntax | Category::Eof => JsonProblem::Syntax,
            },
        }),
    }
}

#[cfg(test)]
#[path = "_tests_/inputs_tests.rs"]
mod inputs_tests;
