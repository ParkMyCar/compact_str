// defmt only works in ELF binaries, and using defmt makes Windows CI fail
#![cfg(not(target_os = "windows"))]

use defmt::Format;

use crate::{CompactString, Drain, ReserveError, ToCompactStringError, Utf16Error};

impl Format for CompactString {
    fn format(&self, fmt: defmt::Formatter) {
        self.as_str().format(fmt)
    }
}

impl Format for Drain<'_> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "Drain({=str})", self.as_str())
    }
}

impl Format for ToCompactStringError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            ToCompactStringError::Reserve(reserve_error) => reserve_error.format(fmt),
            ToCompactStringError::Fmt(core::fmt::Error) => {
                defmt::write!(fmt, "Display::fmt() returned an error")
            }
        }
    }
}

impl Format for ReserveError {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "Cannot allocate memory to hold CompactString")
    }
}

impl Format for Utf16Error {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "invalid utf-16: lone surrogate found")
    }
}
