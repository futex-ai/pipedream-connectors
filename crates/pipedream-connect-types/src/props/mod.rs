//! Lossless configurable property definitions and values.

mod definition;
mod kind;
mod options;
mod serde_helpers;
mod timer;
mod values;

pub use definition::ConfigurableProp;
pub use kind::PropKind;
pub use options::{PropOption, PropOptionValue};
pub use timer::TimerValue;
pub use values::{ConfiguredPropValue, ConfiguredProps};
