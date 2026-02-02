//! CLI command implementations.

mod compile;
mod font;
mod init;
mod template;
mod validate;
mod watch;

pub use compile::{compile, CompileArgs};
pub use font::{font, FontArgs};
pub use init::{init, InitArgs};
pub use template::{template, TemplateArgs};
pub use validate::{validate, ValidateArgs};
pub use watch::{watch, WatchArgs};
