## Examples

Each example is a standalone program in the `examples/` directory. Examples that use a mock SPI backend demonstrate driver logic with simulated register transactions; they do not communicate with physical CAN hardware.

### `auto_bitrate_detect`

Shows a bitrate-detection flow that cycles through clock and bitrate configurations in Listen-Only mode. Uses mock SPI, so it does not detect traffic on a real CAN bus.

### `can_firewall`

Shows how to configure MCP2515 acceptance filters and masks to accept selected CAN IDs. Uses mock SPI; it is an example of filter configuration, not a complete security firewall.

### `can_stress_test`

Demonstrates repeated transmit requests and error-counter monitoring using mock SPI. It does not generate real CAN bus traffic.

### `gateway_bridge`

Shows configuration of acceptance masks and filters for selected CAN IDs, using mock SPI register transactions. It does not bridge traffic between physical CAN buses.

### `latency_benchmark`

Times poll-and-read operations in the example’s mock SPI setup. Results do not represent hardware or CAN-bus performance.

### `loopback`

Shows driver initialization, oscillator and bitrate configuration, and mode switching.

### `loopback_full`

Walks through initialization, bitrate configuration, mode switching, and simulated frame transmission and reception using mock SPI.

### `sniffer_error_monitor`

Demonstrates reading CAN error counters (`TEC` and `REC`) and bus-status information through mock SPI. It does not monitor a physical CAN bus.

### Running an example

From the project root, run:

```bash
cargo run --example loopback
```

Replace `loopback` with the example name you want to run.
