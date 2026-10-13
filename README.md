# MCP2515 Async Rust Driver

A `no_std` Rust driver for the MCP2515 SPI CAN controller. It uses `embedded-hal-async` for SPI operations and is based on the [ESP-IDF MCP2515 driver](https://github.com/Microver-Electronics/mcp2515-esp32-idf).

The MCP2515 is an external chip, not part of the ESP32. You’ll need an MCP2515 board, a compatible CAN transceiver, and an ESP32 Rust platform with the SPI and GPIO support required by your setup.

## Features

- Register access and mode configuration
- CAN frame transmission and reception
- Standard and extended CAN identifiers
- Acceptance masks (`MASK0`, `MASK1`) and filters (`RXF0`–`RXF5`)
- Async SPI operations
- Polling receive and interrupt-based receive, when async GPIO support is configured
- Mock SPI support for testing

## Operating Modes

Normal, Loopback, Listen-Only, Sleep, and Configuration.

## Oscillator and Bitrate Configuration

Supported oscillator settings are 8, 16, and 20 MHz. Configure the driver to match the oscillator on your board. Supported bitrates depend on the oscillator and the driver’s timing configuration; check the configurations in the code before selecting a bitrate.

## Sending a Frame

```rust
let frame = CanFrame::new(0x123, &[1, 2, 3])?;
mcp2515.send(&frame).await?;
```

Classical CAN payloads can contain up to 8 bytes. Standard identifiers are 11-bit; extended identifiers are 29-bit.

## Receiving a Frame

```rust
let frame = mcp2515.read().await?;
```

The driver also provides `recv().await` for interrupt-driven reception, provided the interrupt pin and async GPIO support are configured for your platform.

## Documentation

- MCP2515 Datasheet (Microchip): https://ww1.microchip.com/downloads/en/DeviceDoc/MCP2515-Stand-Alone-CAN-Controller-with-SPI-20001801J.pdf
- MCP2515 Product Page: https://www.microchip.com/en-us/product/MCP2515
