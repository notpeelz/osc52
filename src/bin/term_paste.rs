use std::io::Write;
use std::sync::Arc;

use eyre::{Context, Result};

use term_clipboard::cli;
use term_clipboard::osc52::Osc52TermExt;
use term_clipboard::term::{self, Terminal};

#[derive(clap::Parser)]
#[command(name = cli::NAME, version = cli::VERSION)]
struct Options {
    /// List the offered MIME types instead of pasting
    #[arg(long, short)]
    pub list_types: bool,

    /// Do not append a newline character
    ///
    /// By default the newline character is appended automatically when pasting text MIME types.
    #[arg(short = 'n', long)]
    pub no_newline: bool,

    /// Request the given MIME type instead of inferring the MIME type
    ///
    /// As a special case, specifying "text" will look for a number of plain text types,
    /// prioritizing ones that are known to give UTF-8 text.
    #[arg(
        name = "MIME/TYPE",
        long = "type",
        short = 't',
        conflicts_with = "list_types"
    )]
    pub mime_type: Option<String>,
}

fn main() -> Result<()> {
    let opts = <Options as clap::Parser>::parse();

    let term = Terminal::new(term::tty()?)?;
    let term = drop_guard::guard(Arc::new(term), |term| {
        let _ = term.restore_attrs();
    });

    std::panic::set_hook(Box::new({
        let term = Arc::clone(&term);
        move |info| {
            let _ = term.restore_attrs();
            eprintln!("{info}");
        }
    }));

    let str = {
        let _ = term.set_raw_mode()?;

        let osc52 = term.detect_osc52()?.expect("TODO");
        osc52.read()?
    };

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(str.as_encoded_bytes())?;
    if !opts.no_newline {
        stdout.write_all(b"\n")?;
    }
    stdout.flush()?;

    Ok(())
}
