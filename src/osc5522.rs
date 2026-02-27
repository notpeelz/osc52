use std::{
    io::Write,
    sync::{Arc, LazyLock},
};

use eyre::{Context, Result};
use regex::bytes::Regex;

use crate::{read_append_ext::ReadAppendExt, term::Terminal};

pub struct Osc5522 {
    term: Arc<Terminal>,
}

pub trait Osc5522TermExt {
    fn detect_osc5522(&self) -> Result<Option<Osc5522>>;
}

impl Osc5522TermExt for Arc<Terminal> {
    fn detect_osc5522(&self) -> Result<Option<Osc5522>> {
        static PATTERN: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(?-u)\x1B\[\?5522;(?<code>.*)\$y").unwrap());

        let _ = self.set_raw_mode()?;

        let mut tty = &self.tty;
        tty.write_all(b"\x1B[?5522$p")?;

        let mut buf = Vec::new();
        let mut n = 0_usize;
        let data = loop {
            n += tty.read_append(&mut buf, 128)?;
            let buf = &buf[..n];

            if let Some(s) = PATTERN
                .captures_iter(buf)
                .map(|c| c.name("code").unwrap().as_bytes())
                .next()
            {
                break s;
            }
        };
        let data = str::from_utf8(&data)?;
        let code = data
            .parse::<u32>()
            .wrap_err("DECRQM response code is not a number")?;

        if code == 0 || code == 4 {
            return Ok(None);
        }

        Ok(Some(Osc5522::new(self.clone())))
    }
}

impl Osc5522 {
    pub fn new(term: Arc<Terminal>) -> Self {
        Self { term }
    }
}
