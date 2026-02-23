use std::ffi::OsStr;

use eyre::Result;

use crate::cli::{term_copy, term_paste};

mod base64;
mod cli;
mod lock_ignore_poison_ext;
mod read_append_ext;
mod term;

#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

fn format_error<I: clap::CommandFactory>(err: clap::Error) -> clap::Error {
    let mut cmd = I::command();
    err.format(&mut cmd)
}

fn parse<T>(f: impl Fn(clap::Command) -> clap::Command) -> T
where
    T: clap::CommandFactory + clap::FromArgMatches,
{
    let cmd = <T as clap::CommandFactory>::command();
    let cmd = f(cmd);
    let mut matches = cmd.get_matches();
    let res =
        <T as clap::FromArgMatches>::from_arg_matches_mut(&mut matches).map_err(format_error::<T>);
    match res {
        Ok(s) => s,
        Err(e) => e.exit(),
    }
}

fn main() -> Result<()> {
    reset_sigpipe();

    let Some(bin_name) = std::env::args()
        .next()
        .map(std::path::PathBuf::from)
        .and_then(|argv0| {
            argv0
                .file_name()
                .and_then(OsStr::to_str)
                .map(str::to_string)
        })
    else {
        std::process::exit(1);
    };

    match &*bin_name {
        term_copy::NAME => {
            let opts =
                parse::<term_copy::Options>(|c| c.name(term_copy::NAME).version(cli::VERSION));
            term_copy::main(opts)
        }
        term_paste::NAME => {
            let opts =
                parse::<term_paste::Options>(|c| c.name(term_paste::NAME).version(cli::VERSION));
            term_paste::main(opts)
        }
        _ => {
            let opts = parse::<cli::Options>(|c| c.name(cli::NAME).version(cli::VERSION));
            cli::main(opts)
        }
    }
}
