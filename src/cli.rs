use eyre::Result;

// Cargo enforces semver, so have to resort to this hack to
// strip the major version component.
const fn format_version(s: &str) -> &str {
    assert!(s.is_ascii());
    let mut bytes = s.as_bytes();
    let mut dot_count = 0;
    loop {
        match bytes {
            [] => panic!("invalid version string"),
            [b'.', remaining @ ..] => {
                dot_count += 1;
                bytes = remaining;
            }
            [..] if dot_count == 1 => {
                break;
            }
            [_, remaining @ ..] => {
                bytes = remaining;
            }
        }
    }
    unsafe { str::from_utf8_unchecked(bytes) }
}

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = format_version(env!("CARGO_PKG_VERSION"));

#[derive(clap::Subcommand)]
#[command(subcommand_required(true))]
pub enum Command {
    Copy(crate::cli::term_copy::Options),
    Paste(crate::cli::term_paste::Options),
}

#[derive(clap::Parser)]
pub struct Options {
    #[command(subcommand)]
    pub command: Command,
}

pub fn main(opts: Options) -> Result<()> {
    match opts.command {
        Command::Copy(opts) => term_copy::main(opts),
        Command::Paste(opts) => term_paste::main(opts),
    }
}

pub mod term_copy;
pub mod term_paste;
