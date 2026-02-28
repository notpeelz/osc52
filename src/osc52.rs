use std::{
    ffi::OsString,
    io::Write,
    os::unix::ffi::OsStringExt,
    sync::{Arc, LazyLock},
};

use eyre::Result;
use regex::bytes::Regex;

use crate::{base64, read_append_ext::ReadAppendExt, term::Terminal};

pub struct Osc52 {
    term: Arc<Terminal>,
}

pub trait Osc52TermExt {
    fn detect_osc52(&self) -> Result<Option<Osc52>>;
}

impl Osc52TermExt for Arc<Terminal> {
    fn detect_osc52(&self) -> Result<Option<Osc52>> {
        // TODO: use DA1 to response to determine OSC 52 support
        Ok(Some(Osc52::new(self.clone())))
    }
}

impl Osc52 {
    pub fn new(term: Arc<Terminal>) -> Self {
        Self { term }
    }

    pub fn read(&self) -> Result<OsString> {
        static PATTERN: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"(?-u)\x1B\]52;\w?;(?<str>[A-Za-z0-9+/=]*)[^A-Za-z0-9+/=]").unwrap()
        });

        let mut tty = &self.term.tty;
        tty.write_all(b"\x1B]52;;?\x1B\\")?;
        tty.flush()?;

        let mut buf = Vec::new();
        let mut n = 0_usize;
        let data = loop {
            n += tty.read_append(&mut buf, 4096)?;
            let buf = &buf[..n];

            if let Some(s) = PATTERN
                .captures_iter(buf)
                .map(|c| c.name("str").unwrap().as_bytes())
                .next()
            {
                break s;
            }
        };
        let data = str::from_utf8(&data)?;
        let data = base64::decode(data)?;
        Ok(OsString::from_vec(data))
    }

    pub fn write(&self, str: &[u8]) -> Result<()> {
        let str = base64::encode(str)?;
        let mut tty = &self.term.tty;
        tty.write_all(b"\x1B]52;;")?;
        tty.write_all(str.as_bytes())?;
        tty.write_all(b"\x1B\\")?;
        tty.flush()?;
        Ok(())
    }
}
