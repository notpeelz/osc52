use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;

pub fn tty() -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
}

fn tcgetattr(tty: &std::fs::File) -> std::io::Result<libc::termios> {
    let mut termios = MaybeUninit::<libc::termios>::zeroed();
    let r = unsafe { libc::tcgetattr(tty.as_raw_fd(), termios.as_mut_ptr()) };
    if r != 0 {
        return Err(std::io::Error::from_raw_os_error(r));
    }
    Ok(unsafe { termios.assume_init() })
}

fn tcsetattr(tty: &std::fs::File, termios: &libc::termios) -> std::io::Result<()> {
    let r = unsafe { libc::tcsetattr(tty.as_raw_fd(), libc::TCSANOW, &raw const *termios) };
    if r != 0 {
        return Err(std::io::Error::from_raw_os_error(r));
    }
    Ok(())
}

pub struct Terminal {
    pub tty: std::fs::File,
    termios: libc::termios,
}

pub struct RawModeGuard<'a> {
    fd: i32,
    termios: libc::termios,
    _term: PhantomData<&'a ()>,
}

impl<'a> RawModeGuard<'a> {
    fn new(tty: &std::fs::File) -> std::io::Result<Self> {
        let termios = tcgetattr(tty)?;
        Ok(Self {
            fd: tty.as_raw_fd(),
            termios,
            _term: PhantomData::default(),
        })
    }

    pub fn restore(&self) -> std::io::Result<()> {
        let r = unsafe { libc::tcsetattr(self.fd, libc::TCSANOW, &raw const self.termios) };
        if r != 0 {
            return Err(std::io::Error::from_raw_os_error(r));
        }

        Ok(())
    }
}

impl<'a> Drop for RawModeGuard<'a> {
    fn drop(&mut self) {
        self.restore().unwrap();
    }
}

impl Terminal {
    pub fn new(tty: std::fs::File) -> std::io::Result<Self> {
        let termios = tcgetattr(&tty)?;
        let term = Self { tty, termios };

        // TODO: use DA1 to response to determine OSC 52 support:
        // https://github.com/neovim/neovim/pull/34860/changes

        Ok(term)
    }

    pub fn set_raw_mode(&self) -> std::io::Result<RawModeGuard<'_>> {
        let mut termios = self.termios.clone();
        unsafe { libc::cfmakeraw(&raw mut termios) };
        tcsetattr(&self.tty, &termios)?;
        RawModeGuard::new(&self.tty)
    }

    pub fn restore_attrs(&self) -> std::io::Result<()> {
        tcsetattr(&self.tty, &self.termios)
    }
}
