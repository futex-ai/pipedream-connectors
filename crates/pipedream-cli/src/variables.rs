//! Configuration variables read from the process environment.

use std::env;
use std::ffi::OsString;

use crate::settings::Setting;

/// Reads configuration variables, so that tests never touch the process environment.
#[cfg_attr(test, unimock::unimock(api = [GetVariableMock]))]
pub(crate) trait Variables: Send + Sync {
    /// Returns the raw value of the variable for `setting`, or `None` when it is unset.
    fn get(&self, setting: Setting) -> Option<OsString>;
}

/// Reads variables from the environment of the running process.
pub(crate) struct ProcessVariables;

impl Variables for ProcessVariables {
    fn get(&self, setting: Setting) -> Option<OsString> {
        env::var_os(setting.variable())
    }
}
