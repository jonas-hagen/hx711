//! A platform agnostic driver to interface with the HX711 (load cell amplifier and ADC)
//!
//! This driver was built using [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal/0.2

#![deny(missing_docs)]
#![no_std]
#![feature(never_type)]

extern crate embedded_hal as hal;

extern crate nb;

use hal::digital::v2::InputPin;
use hal::digital::v2::OutputPin;

use core::convert::Infallible;

/// Maximum ADC value
pub const MAX_VALUE: i32 = (1 << 23) - 1;

/// Minimum ADC value
pub const MIN_VALUE: i32 = 1 << 23;

/// HX711 driver
pub struct Hx711<IN, OUT> {
    dout: IN,
    pd_sck: OUT,
    mode: Mode,
}

/// Error type for Input and Output errors on digital pins.
/// For some HALs, the digital input and output pins can never fail.
/// If you use the driver with such a crate, you can use `.into_ok()` on all results
/// instead of `.unwrap()` or `.expect()`.
#[derive(Debug)]
pub enum Error<EIN, EOUT> {
    /// Error while reading a digital pin
    Input(EIN),
    /// Error while writing a digital pin
    Output(EOUT),
}

/// For some hardware crates, the digital input and output pins can never fail.
/// This implementation enables the use of `.into_ok()`.
impl Into<!> for Error<!, !> {
    fn into(self) -> ! {
        panic!()
    }
}

/// For some hardware crates, the digital input and output pins can never fail.
/// This implementation enables the use of `.into_ok()`.
impl Into<!> for Error<Infallible, Infallible> {
    fn into(self) -> ! {
        panic!()
    }
}

impl<IN, OUT, EIN, EOUT> Hx711<IN, OUT>
where
    IN: InputPin<Error = EIN>,
    OUT: OutputPin<Error = EOUT>,
{
    /// Creates a new driver from Input and Outut pins
    pub fn new(dout: IN, mut pd_sck: OUT) -> Result<Self, Error<EIN, EOUT>> {
        pd_sck.set_low().map_err(Error::Output)?;
        let mut hx711 = Hx711 {
            dout,
            pd_sck,
            mode: Mode::ChAGain128,
        };
        hx711.reset()?;
        Ok(hx711)
    }

    /// Set the mode (channel and gain).
    pub fn set_mode(&mut self, mode: Mode) -> nb::Result<(), Error<EIN, EOUT>> {
        self.mode = mode;
        self.retrieve().and(Ok(()))
    }

    /// Reset the chip. Mode is Channel A Gain 128 after reset.
    pub fn reset(&mut self) -> Result<(), Error<EIN, EOUT>> {
        self.pd_sck.set_high().map_err(Error::Output)?;
        for _ in 1..3 {
            self.dout.is_high().map_err(Error::Input)?;
        }
        self.pd_sck.set_low().map_err(Error::Output)?;
        Ok(())
    }

    /// Retrieve the latest conversion value if available
    pub fn retrieve(&mut self) -> nb::Result<i32, Error<EIN, EOUT>> {
        self.pd_sck.set_low().map_err(Error::Output)?;
        if self.dout.is_high().map_err(Error::Input)? {
            // Conversion not ready yet
            return Err(nb::Error::WouldBlock);
        }

        let mut count: i32 = 0;
        for _ in 0..24 {
            // Read 24 bits
            count <<= 1;
            self.pd_sck.set_high().map_err(Error::Output)?;
            self.pd_sck.set_low().map_err(Error::Output)?;

            if self.dout.is_high().map_err(Error::Input)? {
                count += 1;
            }
        }

        // Continue to set mode for next conversion
        let n_reads = self.mode as u16;
        for _ in 0..n_reads {
            self.pd_sck.set_high().map_err(Error::Output)?;
            self.pd_sck.set_low().map_err(Error::Output)?;
        }

        Ok(i24_to_i32(count))
    }
}

/// The HX711 can run in three modes:
#[derive(Copy, Clone)]
pub enum Mode {
    /// Chanel A with factor 128 gain
    ChAGain128 = 1,
    /// Chanel B with factor 64 gain
    ChBGain32 = 2,
    /// Chanel B with factor 32 gain
    ChBGain64 = 3,
}

/// Convert 24 bit signed integer to i32
fn i24_to_i32(x: i32) -> i32 {
    if x >= 0x800000 {
        x | !0xFFFFFF
    } else {
        x
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn convert() {
        assert_eq!(i24_to_i32(0x000001), 1);
        assert_eq!(i24_to_i32(0x000002), 2);
        assert_eq!(i24_to_i32(0xFFFFFF), -1);
        assert_eq!(i24_to_i32(0xFFFFF3), -13);
        assert_eq!(i24_to_i32(0xF00000), -1048576);
        assert_eq!(i24_to_i32(0x800000), -8388608);
        assert_eq!(i24_to_i32(0x7FFFFF), 8388607);
    }
}
