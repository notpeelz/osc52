use std::ffi::OsString;
use std::io::Write;
use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStringExt;

use eyre::Result;
use regex::bytes::Regex;

use crate::base64;
use crate::lock_ignore_poison_ext::LockResultExt;
use crate::read_append_ext::ReadAppendExt;

pub fn tty() -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
}

pub struct Terminal {
    tty: std::sync::Mutex<std::fs::File>,
    termios: libc::termios,
}

impl Terminal {
    pub fn new(tty: std::fs::File) -> std::io::Result<Self> {
        let mut termios = MaybeUninit::<libc::termios>::zeroed();
        let r = unsafe { libc::tcgetattr(tty.as_raw_fd(), termios.as_mut_ptr()) };
        if r != 0 {
            return Err(std::io::Error::from_raw_os_error(r));
        }

        Ok(Self {
            tty: std::sync::Mutex::new(tty),
            termios: unsafe { termios.assume_init() },
        })
    }

    pub fn set_raw_mode(&self) -> std::io::Result<()> {
        let tty = self.tty();
        let mut termios = self.termios.clone();
        unsafe { libc::cfmakeraw(&raw mut termios) };
        let r = unsafe { libc::tcsetattr(tty.as_raw_fd(), libc::TCSANOW, &raw const termios) };
        if r != 0 {
            return Err(std::io::Error::from_raw_os_error(r));
        }
        Ok(())
    }

    pub fn restore_attrs(&self) -> std::io::Result<()> {
        let tty = self.tty();
        let r = unsafe { libc::tcsetattr(tty.as_raw_fd(), libc::TCSANOW, &raw const self.termios) };
        if r != 0 {
            return Err(std::io::Error::from_raw_os_error(r));
        }
        Ok(())
    }

    pub fn tty(&self) -> std::sync::MutexGuard<'_, std::fs::File> {
        self.tty.lock().ignore_poison()
    }

    pub fn osc52_read(&self) -> Result<OsString> {
        // TODO: reuse regex
        let pattern =
            Regex::new(r"(?-u)\x1B]52;\w?;(?<str>[A-Za-z0-9+/=]*)[^A-Za-z0-9+/=]").unwrap();

        let mut tty = self.tty();
        tty.write_all(b"\x1B]52;;?\x1B\\")?;

        let mut buf = Vec::new();
        let mut n = 0_usize;
        let data = loop {
            n += tty.read_append(&mut buf, 4096)?;
            let buf = &buf[..n];

            if let Some(s) = pattern
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

    pub fn osc52_write(&self, str: &[u8]) -> Result<()> {
        let str = base64::encode(str)?;
        let mut tty = self.tty();
        tty.write_all(b"\x1B]52;;")?;
        tty.write_all(str.as_bytes())?;
        tty.write_all(b"\x1B\\")?;
        Ok(())
    }

    // TODO: osc5522
}
