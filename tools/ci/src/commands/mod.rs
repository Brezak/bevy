mod bench_check;
mod clippy;
mod compile;
mod compile_check;
mod compile_check_no_std;
mod compile_fail;
mod doc;
mod doc_check;
mod doc_test;
mod example_check;
mod format;
mod lints;
mod test;
mod test_check;

pub use bench_check::*;
pub use clippy::*;
pub use compile::*;
pub use compile_check::*;
pub use compile_check_no_std::*;
pub use compile_fail::*;
pub use doc::*;
pub use doc_check::*;
pub use doc_test::*;
pub use example_check::*;
pub use format::*;
pub use lints::*;
pub use test::*;
pub use test_check::*;

use crate::json::JsonCommandOutput;
use std::process::{Command, ExitStatus, Stdio};

#[derive(Debug, Default)]
pub enum RustChannel {
    #[default]
    Stable,
    Nightly,
}

fn status_to_result(status: ExitStatus) -> Result<(), ()> {
    if status.success() {
        Ok(())
    } else {
        Err(())
    }
}

/// Runs a cargo command
///
/// stdout, stderr and stdin will be inherited.
/// Args will be passed as is and no splitting will be performed (see [`args`](Command::args)).
///
/// Err(()) will be returned when:
/// - There was a issue invoking the command (error message will be printed to stderr).
/// - The command returned a non success exit code.
pub fn run_cargo_command(
    cargo_command: &str,
    channel: RustChannel,
    args: &[&str],
    env: &[(&str, &str)],
) -> Result<(), ()> {
    let channel = match channel {
        RustChannel::Stable => "stable",
        RustChannel::Nightly => "nightly",
    };

    // We have to go through rustup as invoking cargo directly won't let us choose the toolchain.
    // The +channel syntax you might be used to doesn't work as the cargo invoked by `Command` is
    // different than the cargo you invoke in the console. Specifically your invocation goes through
    // rustup which picks the correct cargo installation for you. `Command` seems to go directly to
    // the system wide install of cargo.
    Command::new("rustup")
        .args(["run", channel, "cargo", cargo_command])
        .args(args)
        .envs(env.iter().copied())
        .status()
        .map_err(|err| eprintln!("{err}"))
        .and_then(status_to_result)
}

/// Runs a cargo command configured to output json and parses it's output.
///
/// stdout, stderr and stdin will be inherited.
/// Args will be passed as is and no splitting will be performed (see [`args`](Command::args)).
///
///
/// Err(()) will be returned when:
/// - There was a issue invoking the command (error message will be printed to stderr).
/// - The command returned a non success exit code.
pub fn run_cargo_command_with_json(
    cargo_command: &str,
    command_name: &str,
    channel: RustChannel,
    flags: &[&str],
    env: &[(&str, &str)],
) -> Result<JsonCommandOutput, ()> {
    let channel = match channel {
        RustChannel::Stable => "stable",
        RustChannel::Nightly => "nightly",
    };

    //  See comment in `run_cargo_command` for why we're invoking rustup.
    let mut child = Command::new("rustup")
        .args([
            "run",
            channel,
            "cargo",
            cargo_command,
            "--message-format",
            "json",
        ])
        .args(flags)
        .envs(env.iter().copied())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| eprintln!("{err}"))?;

    let Some(command_out) = child.stdout.take() else {
        let _ = child.kill(); // Kill the child so we don't leave it running in the background while we burn down everything
        unreachable!("Child was configured to pipe it's stdout to use but didn't")
    };

    match JsonCommandOutput::from_cargo_output(command_out, command_name.to_string()) {
        Ok(json) => {
            let _ = child.wait(); // Don't just leave the child running in the background
            Ok(json)
        }
        Err(err) => {
            let _ = child.wait();
            eprintln!("{err}");
            Err(())
        }
    }
}
