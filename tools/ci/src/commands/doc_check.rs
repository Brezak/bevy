use crate::json::JsonCommandOutput;
use argh::FromArgs;

use super::{run_cargo_command, run_cargo_command_with_json, RustChannel};

/// Checks that all docs compile.
#[derive(FromArgs, Default)]
#[argh(subcommand, name = "doc-check")]
pub struct DocCheckCommand {
    #[argh(switch)]
    /// emit errors as json
    emit_json: bool,
}

impl DocCheckCommand {
    const FLAGS: &'static [&'static str] = &[
        "--workspace",
        "--all-features",
        "--no-deps",
        "--document-private-items",
    ];
    const ENV_VARS: &'static [(&'static str, &'static str)] = &[];

    /// Runs this command.
    ///
    /// For use in aliases.
    pub fn run_with_intermediate() -> Result<(), ()> {
        run_cargo_command("doc", RustChannel::Stable, Self::FLAGS, Self::ENV_VARS)
    }

    /// Runs this command with json output.
    ///
    /// For use in aliases.
    pub fn run_with_intermediate_json() -> Result<JsonCommandOutput, ()> {
        run_cargo_command_with_json(
            "doc",
            "doc-check",
            RustChannel::Stable,
            Self::FLAGS,
            Self::ENV_VARS,
        )
    }

    /// Runs this command.
    pub fn run(self) -> Result<(), ()> {
        if self.emit_json {
            Self::run_with_intermediate_json().map(|json| {
                println!("[{}]", json.as_json_string());
            })
        } else {
            Self::run_with_intermediate()
        }
    }
}
