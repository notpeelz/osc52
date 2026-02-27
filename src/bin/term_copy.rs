use std::ffi::OsString;
use std::io::Cursor;
use std::os::unix::ffi::OsStringExt;
use std::sync::Arc;

use eyre::{Context, Result};
use term_clipboard::cli::{self, reset_sigpipe};
use term_clipboard::osc52::Osc52TermExt;
use term_clipboard::term::{self, Terminal};

#[derive(clap::Parser)]
#[command(name = cli::NAME, version = cli::VERSION)]
struct Options {
    /// Override the inferred MIME type for the content
    #[arg(
        name = "MIME/TYPE",
        long = "type",
        short = 't',
        conflicts_with = "clear"
    )]
    pub mime_type: Option<String>,

    /// Trim the trailing newline character before copying
    ///
    /// This flag is only applied for text MIME types.
    #[arg(long, short = 'n', conflicts_with = "clear")]
    pub trim_newline: bool,

    /// Text to copy
    ///
    /// If not specified, term-copy will use data from stdin.
    #[arg(name = "TEXT TO COPY", conflicts_with = "clear")]
    pub text: Vec<OsString>,

    /// Clear the clipboard instead of copying
    #[arg(long, short)]
    pub clear: bool,
}

// Copied from https://github.com/YaLTeR/wl-clipboard-rs/blob/2f6a8852665bd1891a3f3ffa204e62b0f588ef95/src/utils.rs#L30
fn is_text(mime_type: &str) -> bool {
    match mime_type {
        "TEXT" | "STRING" | "UTF8_STRING" => true,
        x if x.starts_with("text/") => true,
        // Common script and markup types.
        x if x.contains("json")
            | x.ends_with("script")
            | x.ends_with("xml")
            | x.ends_with("yaml")
            | x.ends_with("csv")
            | x.ends_with("ini") =>
        {
            true
        }
        _ => false,
    }
}

fn main() -> Result<()> {
    reset_sigpipe();

    let mut opts = <Options as clap::Parser>::parse();

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

    let osc52 = term.detect_osc52()?.expect("TODO");

    if opts.clear {
        let _ = term.set_raw_mode()?;
        osc52.write(&[])?;
        return Ok(());
    }

    let mut mime_type = opts.mime_type;

    let mut data = if opts.text.is_empty() {
        let mut contents = Cursor::new(Vec::new());
        std::io::copy(&mut std::io::stdin(), &mut contents)
            .wrap_err("failed to copy data from stdin")?;
        contents.into_inner()
    } else {
        mime_type = mime_type.or(Some("text/plain".into()));
        let mut iter = opts.text.drain(..);
        let mut str = iter.next().unwrap();
        for s in iter {
            str.push(" ");
            str.push(s);
        }
        str.into_vec()
    };

    if let Some(mime_type) = &mime_type {
        if opts.trim_newline && is_text(&mime_type) && data.last().copied() == Some(b'\n') {
            data.pop();
        }
    }

    let _ = term.set_raw_mode()?;
    osc52.write(&data)?;

    Ok(())
}
