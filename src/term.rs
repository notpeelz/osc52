use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;

pub fn tty() -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
}

pub struct Terminal {
    pub tty: std::fs::File,
    termios: libc::termios,
}

impl Terminal {
    pub fn new(tty: std::fs::File) -> std::io::Result<Self> {
        let mut termios = MaybeUninit::<libc::termios>::zeroed();
        let r = unsafe { libc::tcgetattr(tty.as_raw_fd(), termios.as_mut_ptr()) };
        if r != 0 {
            return Err(std::io::Error::from_raw_os_error(r));
        }

        let term = Self {
            tty,
            termios: unsafe { termios.assume_init() },
        };

        // TODO: use DA1 to response to determine OSC 52 support:
        // https://github.com/neovim/neovim/pull/34860/changes

        Ok(term)
    }

    pub fn set_raw_mode(&self) -> std::io::Result<()> {
        let mut termios = self.termios.clone();
        unsafe { libc::cfmakeraw(&raw mut termios) };
        let r = unsafe { libc::tcsetattr(self.tty.as_raw_fd(), libc::TCSANOW, &raw const termios) };
        if r != 0 {
            return Err(std::io::Error::from_raw_os_error(r));
        }
        Ok(())
    }

    pub fn restore_attrs(&self) -> std::io::Result<()> {
        let r = unsafe {
            libc::tcsetattr(self.tty.as_raw_fd(), libc::TCSANOW, &raw const self.termios)
        };
        if r != 0 {
            return Err(std::io::Error::from_raw_os_error(r));
        }
        Ok(())
    }
}
