use std::ffi::OsStr;

use eyre::Result;

use crate::cli::{term_copy, term_paste};

mod base64;
mod cli;
mod lock_ignore_poison_ext;
mod read_append_ext;
mod term;

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
