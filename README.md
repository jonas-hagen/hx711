[![Build Status](https://travis-ci.org/jonas-hagen/hx711.svg?branch=master)](https://travis-ci.org/jonas-hagen/hx711)
[![crates.io](https://img.shields.io/crates/v/hx711.svg)](https://crates.io/crates/hx711)
[![docs.rs](https://docs.rs/hx711/badge.svg)](https://docs.rs/hx711)

# `HX711`

> A platform agnostic driver to interface with the HX711 (load cell amplifier and 24 bit ADC)

## What works

- Resetting the chip
- Setting the mode (gain and channel)
- Reading conversion results (blocking and non blocking)
- Entering low-power mode and wake up

Tested with STM32F103. Pull requests for other hardware welcome!

Because the interface works by toggling of GPIO pins, some care has to be taken. See below.

## Examples

See here: https://github.com/jonas-hagen/hx711-examples

Incomplete appetizer:
```rust
let mut val: i32 = 0;

let dout = gpioa.pa6.into_floating_input(&mut gpioa.crl);
let pd_sck = gpioa.pa7.into_push_pull_output(&mut gpioa.crl);

let mut hx711 = Hx711::new(Delay::new(cp.SYST, clocks), dout, pd_sck).into_ok();

// Obtain the tara value
writeln!(tx, "Obtaining tara ...").unwrap();
const N: i32 = 8;
for _ in 0..N {
    val += block!(hx711.retrieve()).into_ok(); // or unwrap, see features below
}
let tara = val / N;
writeln!(tx, "Tara:   {}", tara).unwrap();
```

## Optional features

### `never_type`

The `never_type` feature can optionally be enabled when using nightly rust.

For some HALs, the digital input and output pins can never fail.
When using the driver with such a crate, one can use `.into_ok()` on all results instead of `.unwrap()` or `.expect()`.

Example with e.g. STM32f1xx embedded hal:
```rust
// Without never_type (stable rust):
// We know that this never fails
let weight = block!(hx711.retrieve()).unwrap())

// With never_type (nightly rust)
// It is obvious that this is always ok
let weight = block!(hx711.retrieve()).into_ok()
```

## Bit-banging and delays

The protocol is implemented using the GPIO interface because the HX711 needs a specific number of clock cycles to set the operation mode (25, 26 or 27 cycles). 
So, **beware of interrupts** during readout!
The delays between state changes for clocking only need to be 0.1 µs (per HX711 specs), allowing to clock through all 24 cycles in approximately 5 µs.
But the current embedded HAL does not support delays smaller than 1 µs (see [embedded-hal #63](https://github.com/rust-embedded/embedded-hal/issues/63) for the discussion).
With a delay of 1 µs, the readout takes at least 48 µs.
Depending on clock speed of the device, this increases to 300 µs in total (tested with STM32F103 at 8 MHz) or more.
In a control loop, this might be undesirable and keeping the time closer to the 5 µs is compelling.

To tweak the performance, an alternative implementation of the delay trait can be used. For example:

```rust
use embedded_hal::blocking::delay::DelayUs;

pub struct BusyDelay {
    nops_per_us: u32,
}

impl BusyDelay {
    pub fn new(loops_per_us: u32) -> Self {
        BusyDelay{loops_per_us: loops_per_us as u32}
    }
}

impl DelayUs<u32> for BusyDelay {
    fn delay_us(&mut self, us: u32) {
        for _ in 0..us {
            cortex_m::asm::delay(self.nops_per_us)
        }
    }
}
```

Or even:

```rust
pub struct NoDelay();

impl NoDelay {
    pub fn new() -> Self {
        NoDelay()
    }
}

impl DelayUs<u32> for NoDelay {
    fn delay_us(&mut self, _us: u32) {
    }
}
```

Some random notes on this topic:
* Maybe add a `nodelay` feature?
* It would be interesting to try to use SPI with MOSI as clock and MISO as data line, not using the actual SPI clock.
* How to make this work on linux? Any ideas?

## Changelog

### v0.7

- Fix naming of mode Enum. This is a breaking change if the mode `ChBGain64` is used (which does not exist in hardware). Thanks *joelsa*!

### v0.6

- Update nb requirement to 1.0.0
- Add `get_mode` function. Thanks *m-ou-se*!

### v0.5

- Hide usage of `never_type` behind feature for usage on stable rust

### v0.4

- Add delays.
- Add `enable()` and `disable()` functions to enter and leave low-power mode.

### v0.3

- Add custom error type. Some HALs implement the digital interface in a way that it cannot fail. In this case, the Error type of Input and Output pins are Infallible. With a custom Error type we can allow the use of `.into_ok()` when using such HAL implementations.

### v0.2

- Update to `embedded-hal` digital pin v2 API. Thanks to *mmou*!

### v0.1

- First version.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

