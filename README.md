[![Build Status](https://travis-ci.org/jonas-hagen/hx711.svg?branch=master)](https://travis-ci.org/jonas-hagen/hx711)

# `HX711`

> A platform agnostic driver to interface with the HX711 (load cell amplifier and 24 bit ADC)

## What works

- Resetting the chip
- Setting the mode (gain and channel)
- Reading conversion results (blocking and non blocking)

Tested with STM32F103. Pull requests for other hardware welcome!

Because the interface works by toggling of GPIO pins, timing is important. Example for linux (Raspberry Pi) does not work reliably due to timing issues, but it should work good enough for quick and easy testing.

## Examples

See here: https://github.com/jonas-hagen/hx711-examples

## Changelog

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

