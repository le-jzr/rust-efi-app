
use core;
use core::convert::TryFrom;

pub mod console;
pub mod boot_services;

pub struct Guid(u32, u16, u16, [u8; 8]);

// TODO
#[derive(Copy, Clone, Debug)]
pub struct Status {
    code: usize,
}

impl Status {
    fn success() -> Status {
        Status{code: 0}
    }

    fn is_success(&self) -> bool {
        self.code == 0
    }
}

// TODO
pub struct Error {
    code: usize,
}

impl Error {
    pub fn invalid_parameter() -> Error {
        // FIXME
        Error { code: 2 }
    }
}

#[repr(u8)]
pub enum Color {
    Black = 0x00,
    Blue = 0x01,
    Green = 0x02,
    Cyan = 0x03,
    Red = 0x04,
    Magenta = 0x05,
    Brown = 0x06,
    LightGray = 0x07,
    DarkGray = 0x08,
    LightBlue = 0x09,
    LightGreen = 0x0a,
    LightCyan = 0x0b,
    LightRed = 0x0c,
    LightMagenta = 0x0d,
    Yellow = 0x0e,
    White = 0x0f,
}

impl TryFrom<u8> for Color {
    type Error = ();

    fn try_from(val: u8) -> core::result::Result<Color, Self::Error> {
        if val > (Color::White as u8) {
            return Err(());
        }
        Ok(unsafe{ core::mem::transmute(val) })
    }
}

pub type Result<T> = core::result::Result<T, Error>;

pub(super) fn status_to_status<N: Into<u64>>(status: N) -> Result<Status> {
    let status: u64 = status.into();

    if core::mem::size_of::<N>() == 8 {
        if status & 0x8000000000000000 != 0 {
            return Err(Error{code: (status ^ 0x8000000000000000) as _});
        }
    }
    if core::mem::size_of::<N>() == 4 {
        if status & 0x80000000 != 0 {
            return Err(Error{code: (status ^ 0x80000000) as _});
        }
    }

    return Ok(Status{code: status as _});
}

pub(super) fn status_to_result<T, N: Into<u64>>(status: N, val: T) -> Result<T> {
    let status: u64 = status.into();

    if core::mem::size_of::<N>() == 8 {
        if status & 0x8000000000000000 != 0 {
            return Err(Error{code: (status ^ 0x8000000000000000) as _});
        }
    }
    if core::mem::size_of::<N>() == 4 {
        if status & 0x80000000 != 0 {
            return Err(Error{code: (status ^ 0x80000000) as _});
        }
    }

    return Ok(val);
}
