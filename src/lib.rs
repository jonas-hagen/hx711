//! A platform agnostic driver to interface with the HX711 (load cell amplifier and ADC)
//!
//! This driver was built using [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal/1.0.0
//! Patched by Jesse Bud @ Scylla Digital PTY LTD (Australia)

#![deny(missing_docs)]
#![no_std]
#![cfg_attr(feature = "never_type", feature(never_type))]
#![cfg_attr(all(feature = "never_type", test), feature(unwrap_infallible))]

extern crate embedded_hal as hal;

extern crate nb;



#[cfg(feature = "never_type")]
use core::convert::Infallible;
use hal::delay::DelayNs;
use hal::digital::{InputPin, OutputPin};

/// Maximum ADC value
pub const MAX_VALUE: i32 = (1 << 23) - 1;

/// Minimum ADC value
pub const MIN_VALUE: i32 = 1 << 23;

/// When PD_SCK pin changes from low to high and stays at high for logner than 60 us, HX711 enters power down mode. When PD_SCK returns low, the chip will reset and enter normal operation mode.
#[allow(dead_code)]
const TIME_TO_SLEEP: u32 = 70;

/// DOUT falling edge to PD_SCK rising edge (T1)
const TIME_BEFORE_READOUT: u32 = 1;

/// PD_SCK high time (T3)
const TIME_SCK_HIGH: u32 = 1;

/// PD_SCK low time (T4)
const TIME_SCK_LOW: u32 = 1;

/// HX711 driver
pub struct Hx711<D, IN, OUT> {
    delay: D,
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
#[cfg(feature = "never_type")]
impl Into<!> for Error<!, !> {
    fn into(self) -> ! {
        panic!()
    }
}

/// For some hardware crates, the digital input and output pins can never fail.
/// This implementation enables the use of `.into_ok()`.
#[cfg(feature = "never_type")]
impl Into<!> for Error<Infallible, Infallible> {
    fn into(self) -> ! {
        panic!()
    }
}

impl<D, IN, OUT, EIN, EOUT> Hx711<D, IN, OUT>
where
    D: DelayNs,
    IN: InputPin<Error = EIN>,
    OUT: OutputPin<Error = EOUT>,
{
    /// Creates a new driver from Input and Outut pins
    pub fn new(delay: D, dout: IN, mut pd_sck: OUT) -> Result<Self, Error<EIN, EOUT>> {
        pd_sck.set_low().map_err(Error::Output)?;
        let mut hx711 = Hx711 {
            delay,
            dout,
            pd_sck,
            mode: Mode::ChAGain128,
        };
        hx711.reset()?;
        Ok(hx711)
    }

    /// Get the mode (channel and gain).
    pub fn get_mode(&self) -> Mode {
        self.mode
    }

    /// Set the mode (channel and gain).
    pub fn set_mode(&mut self, mode: Mode) -> nb::Result<(), Error<EIN, EOUT>> {
        self.mode = mode;
        self.retrieve().and(Ok(()))
    }
    /// Put the chip in power down state.
    pub fn disable(&mut self) -> Result<(), Error<EIN, EOUT>> {
        self.pd_sck.set_high().map_err(Error::Output)?;
        self.delay.delay_us(TIME_TO_SLEEP);
        Ok(())
    }

    /// Wake the chip up and set mode.
    pub fn enable(&mut self) -> Result<(), Error<EIN, EOUT>> {
        self.pd_sck.set_low().map_err(Error::Output)?;
        self.delay.delay_us(TIME_SCK_LOW);
        nb::block! {self.set_mode(self.mode)}
    }

    /// Reset the chip.
    pub fn reset(&mut self) -> Result<(), Error<EIN, EOUT>> {
        self.disable()?;
        self.enable()
    }

    /// Retrieve the latest conversion value if available
    pub fn retrieve(&mut self) -> nb::Result<i32, Error<EIN, EOUT>> {
        self.pd_sck.set_low().map_err(Error::Output)?;
        if self.dout.is_high().map_err(Error::Input)? {
            // Conversion not ready yet
            return Err(nb::Error::WouldBlock);
        }
        self.delay.delay_us(TIME_BEFORE_READOUT);

        let mut count: i32 = 0;
        for _ in 0..24 {
            // Read 24 bits
            count <<= 1;
            self.pd_sck.set_high().map_err(Error::Output)?;
            self.delay.delay_us(TIME_SCK_HIGH);
            self.pd_sck.set_low().map_err(Error::Output)?;

            if self.dout.is_high().map_err(Error::Input)? {
                count += 1;
            }
            self.delay.delay_us(TIME_SCK_LOW);
        }

        // Continue to set mode for next conversion
        let n_reads = self.mode as u16;
        for _ in 0..n_reads {
            self.pd_sck.set_high().map_err(Error::Output)?;
            self.delay.delay_us(TIME_SCK_HIGH);
            self.pd_sck.set_low().map_err(Error::Output)?;
            self.delay.delay_us(TIME_SCK_LOW);
        }

        Ok(i24_to_i32(count))
    }
}

/// The HX711 can run in three modes (see Table 3 in Datasheet):
#[derive(Copy, Clone)]
pub enum Mode {
    /// Chanel A with factor 128 gain
    ChAGain128 = 1,
    /// Chanel B with factor 32 gain
    ChBGain32 = 2,
    /// Chanel A with factor 64 gain
    ChAGain64 = 3,
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

    #[cfg(feature = "never_type")]
    use core::convert::Infallible;

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

    #[test]
    #[cfg(feature = "never_type")]
    fn infallible_into_ok() {
        let this_is_ok: Result<usize, Error<Infallible, Infallible>> = Ok(77);
        assert_eq!(this_is_ok.into_ok(), 77);
    }

    #[test]
    #[cfg(feature = "never_type")]
    fn never_fail_into_ok() {
        let this_is_ok: Result<usize, Error<!, !>> = Ok(77);
        assert_eq!(this_is_ok.into_ok(), 77);
    }
}
