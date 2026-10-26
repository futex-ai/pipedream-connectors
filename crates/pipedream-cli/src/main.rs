//! Pipedream Connect command line for smoke tests and JSON inspection.

#![warn(unreachable_pub)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]

mod app;
mod argument;
mod cli;
mod commands;
mod connector;
mod document;
mod error;
mod inputs;
mod redact;
mod report;
mod settings;
mod target;
mod terminal;
mod variables;

use std::env;
use std::io;
use std::process::ExitCode;
use std::sync::Arc;

use crate::app::{App, Application};
use crate::connector::SdkConnector;
use crate::inputs::FileInputs;
use crate::terminal::Terminal;
use crate::variables::ProcessVariables;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let app = App::new(
        Arc::new(ProcessVariables),
        Arc::new(FileInputs),
        Arc::new(SdkConnector),
    );
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let mut terminal = Terminal::new(&mut stdout, &mut stderr);
    app.run(env::args_os().collect(), &mut terminal)
        .await
        .into()
}

#[cfg(test)]
#[path = "_tests_/support.rs"]
mod test_support;
