//! Writing one outcome to the terminal and choosing its exit status.

use crate::document::{Notice, Output};
use crate::error::Result;
use crate::report::exit::Exit;
use crate::report::failure::{count, cursor_flag, describe};
use crate::settings::{ApiOrigin, Setting};
use crate::terminal::Terminal;

/// The message when standard output rejects a result.
const OUTPUT_FAILURE: &str =
    "The CLI cannot write to standard output. Check that the output stream is open.";

/// Writes help, version text, or a parse failure and returns its exit status.
pub(crate) fn usage(error: &clap::Error, terminal: &mut Terminal<'_>) -> Exit {
    let text = error.render().to_string();
    if error.use_stderr() {
        terminal.diagnostic_text(&text);
        return Exit::Usage;
    }
    match terminal.text(&text) {
        Ok(()) => Exit::Success,
        Err(_) => {
            terminal.diagnostic("error", OUTPUT_FAILURE);
            Exit::Internal
        }
    }
}

/// Writes the note that names a custom API origin and the flag or variable that set it.
pub(crate) fn origin_note(origin: &ApiOrigin, terminal: &mut Terminal<'_>) {
    let source = origin.supply.name(Setting::ApiOrigin);
    terminal.diagnostic(
        "note",
        &format!("The CLI sends requests to {}, set by {source}.", origin.url),
    );
}

/// Writes the result or the failure of one command and returns its exit status.
pub(crate) fn finish(outcome: Result<Output>, origin: &str, terminal: &mut Terminal<'_>) -> Exit {
    let output = match outcome {
        Ok(output) => output,
        Err(error) => {
            let failure = describe(&error, origin);
            terminal.diagnostic("error", &failure.message);
            return failure.exit;
        }
    };
    if terminal.result(output.document.bytes()).is_err() {
        terminal.diagnostic("error", OUTPUT_FAILURE);
        return Exit::Internal;
    }
    for notice in &output.notices {
        terminal.diagnostic("note", &notice_text(notice));
    }
    Exit::Success
}

fn notice_text(notice: &Notice) -> String {
    match notice {
        Notice::PageLimit { pages, next } => {
            let summary = format!("The CLI stopped after {}.", count(*pages));
            match cursor_flag(next) {
                Some((flag, cursor)) => format!(
                    "{summary} To read the next page, run the command again with {flag} {cursor}."
                ),
                None => summary,
            }
        }
        Notice::StuckCursor => "Pipedream repeated a page cursor, so the CLI stopped. The result \
                                can contain repeated records."
            .to_owned(),
    }
}

#[cfg(test)]
#[path = "_tests_/outcome_tests.rs"]
mod outcome_tests;
