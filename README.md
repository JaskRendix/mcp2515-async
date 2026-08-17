# ESP32 MCP2515 CAN Interface Library (ESP‑IDF, Async Rust Port)

This project provides a Rust implementation of the MCP2515 CAN controller driver.

It is based on existing C drivers for ESP32 and similar microcontrollers, rewritten to use `no_std` and `embedded-hal-async`.

The goal is to replace blocking SPI operations with an async interface suitable for cooperative embedded runtimes.

---

## Purpose of the Port

The original MCP2515 drivers use blocking SPI calls. Blocking SPI prevents other tasks from progressing while a transfer is active. Systems that share the SPI bus or run multiple concurrent tasks cannot schedule work predictably under this model.

This port provides:

- an async SPI interface  
- a `no_std` Rust API  
- predictable behavior under concurrency  
- compatibility with async executors  
- a testable driver with mockable SPI backends  

The functional behavior of the original driver is preserved. The communication model is updated to support event‑driven embedded systems.

---

## Upstream Source

This async Rust driver is based on the original MCP2515 ESP‑IDF component:

- [https://github.com/Microver-Electronics/mcp2515-esp32-idf](https://github.com/Microver-Electronics/mcp2515-esp32-idf)

---

## Hardware

- **Controller:** Microchip MCP2515 (SPI CAN controller)  
- **Transceivers:** MCP2551, MCP2562, TJA1055  

---

## Rust Setup (Async Port)

The Rust driver uses:

- `embedded-hal-async` for SPI  
- `no_std`  
- a mockable SPI interface for testing  

The driver exposes:

- register access  
- mode configuration  
- frame transmission  
- frame reception  

All SPI operations are async.

---

## Operation Modes

The MCP2515 supports:

- Normal  
- Loopback  
- Listen‑Only  
- Sleep  
- Configuration  

---

## Baud Rates

Supported baud rates include:

- 5 kbps  
- 10 kbps  
- 20 kbps  
- 31.25 kbps  
- 33 kbps  
- 40 kbps  
- 50 kbps  
- 80 kbps  
- 83.3 kbps  
- 95 kbps  
- 100 kbps  
- 125 kbps  
- 200 kbps  
- 250 kbps  
- 500 kbps  
- 1 Mbps  

---

## Clock Speeds

Supported oscillator values:

- 20 MHz  
- 16 MHz  
- 8 MHz  

Default: **16 MHz**.

---

## Frame Format

```c
typedef struct can_frame {
    canid_t can_id;
    __u8    can_dlc;
    __u8    data[CAN_MAX_DLEN] __attribute__((aligned(8)));
} CAN_FRAME_t[1], *CAN_FRAME;
```

The Rust port provides an equivalent structure with fixed‑size data and explicit ID handling.

---

## Sending Frames

### C API

```c
ERROR_t MCP2515_sendMessage(const TXBn_t txbn, const CAN_FRAME frame);
ERROR_t MCP2515_sendMessageAfterCtrlCheck(const CAN_FRAME frame);
```

### Rust API

```rust
let frame = CanFrame::new(0x123, &[1, 2, 3]).unwrap();
mcp2515.send(&frame).await?;
```

This method checks transmit buffers (`TXB0`, `TXB1`, `TXB2`), formats standard or extended identifiers, writes data via SPI, and triggers a Request‑to‑Send instruction.

---

## Receiving Frames

### C API

```c
ERROR_t MCP2515_readMessage(const RXBn_t rxbn, const CAN_FRAME frame);
ERROR_t MCP2515_readMessageAfterStatCheck(const CAN_FRAME frame);
```

### Rust API

#### Polling

```rust
let frame = mcp2515.read().await?;
```

#### Async Interrupt

```rust
let frame = mcp2515.recv().await?;
```

This avoids busy‑loop polling by waiting for the interrupt pin to signal data availability.

---

## Filters and Masks

### C API

```c
ERROR_t MCP2515_setFilterMask(const MASK_t num, const bool ext, const uint32_t ulData);
ERROR_t MCP2515_setFilter(const RXF_t num, const bool ext, const uint32_t ulData);
```

Supported:

- Masks: `MASK0`, `MASK1`  
- Filters: `RXF0` through `RXF5`  
- Standard (11‑bit) and extended (29‑bit) IDs  

---

## Examples

- **`auto_bitrate_detect`** — Demonstrates automated CAN bitrate scanning and dynamic detection by cycling through standard clock/bitrate configurations in Listen-Only mode using a mock SPI backend.  
- **`can_firewall`** — Demonstrates a selective CAN security firewall utilizing hardware acceptance filters and masks to allow specific ID ranges while blocking others, tested with a mock SPI backend.  
- **`can_stress_test`** — Demonstrates high-frequency TX frame flooding combined with real-time error counter monitoring (TEC/REC) using a mock SPI backend.  
- **`gateway_bridge`** — Demonstrates advanced configuration, including hardware acceptance filter masks and target IDs, combined with mock SPI register transactions.
- **`latency_benchmark`** — Demonstrates driver performance measurement, timing poll-and-read operations using a mock SPI backend.  
- **`loopback`** — Demonstrates driver initialization, clock/bitrate configuration, and mode switching.  
- **`loopback_full`** — Demonstrates a complete end‑to‑end workflow including initialization, bitrate configuration, mode switching, and mock SPI transmit/receive frame handling.  
- **`sniffer_error_monitor`** — Demonstrates a diagnostic CAN sniffer and error monitor that tracks error counters (TEC/REC) and bus status using a mock SPI backend.  

Run examples:

```bash
cargo run --example loopback
```
