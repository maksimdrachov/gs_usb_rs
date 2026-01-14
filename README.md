# gs_usb - GS-USB Protocol Implementation for Rust

A Rust implementation of the GS-USB protocol for communicating with CAN bus adapters like candleLight, CANable, and other GS-USB compatible devices.

## Features

- **Classic CAN Support**: Up to 1 Mbps bitrate
- **CAN FD Support**: Up to 10 Mbps data rate with flexible data length
- **Hardware Timestamps**: Microsecond-precision timestamps from the device
- **Multiple Operating Modes**: Normal, listen-only, loopback, and one-shot modes
- **Device State Monitoring**: Error counters and bus state information
- **Cross-Platform**: Works on Linux, macOS, and Windows

## Supported Devices

- GS-USB devices (VID: 0x1D50, PID: 0x606F)
- candleLight (VID: 0x1209, PID: 0x2323)
- CES CANext FD (VID: 0x1CD2, PID: 0x606F)
- ABE CANdebugger FD (VID: 0x16D0, PID: 0x10B8)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gs_usb = "0.1"
```

### System Dependencies

This crate requires libusb to be installed:

**Linux (Debian/Ubuntu):**
```bash
sudo apt-get install libusb-1.0-0-dev
```

**Linux (Fedora):**
```bash
sudo dnf install libusb1-devel
```

**macOS:**
```bash
brew install libusb
```

**Windows:**
Download and install libusb from https://libusb.info/

## Quick Start

```rust
use gs_usb::{GsUsb, GsUsbFrame, GS_CAN_MODE_NORMAL, GS_CAN_MODE_HW_TIMESTAMP};
use std::time::Duration;

fn main() -> gs_usb::Result<()> {
    // Scan for devices
    let devices = GsUsb::scan()?;
    if devices.is_empty() {
        println!("No GS-USB device found");
        return Ok(());
    }

    let mut dev = devices.into_iter().next().unwrap();

    // Configure bitrate (250 kbps)
    dev.set_bitrate(250000)?;

    // Start the device
    dev.start(GS_CAN_MODE_NORMAL | GS_CAN_MODE_HW_TIMESTAMP)?;

    // Send a frame
    let data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
    let frame = GsUsbFrame::with_data(0x7FF, &data);
    dev.send(&frame)?;

    // Read frames
    loop {
        match dev.read(Duration::from_millis(100)) {
            Ok(frame) => println!("RX  {}", frame),
            Err(gs_usb::GsUsbError::ReadTimeout) => continue,
            Err(e) => return Err(e),
        }
    }
}
```

## Examples

Run the examples with:

```bash
# Basic usage example
cargo run --example gs_usb_example

# Device initialization sequence
cargo run --example device_initialization

# CAN FD channel start sequence
cargo run --example channel_start_sequence

# Test all supported bitrates
cargo run --example test_all_bitrates

# Test GET_STATE feature
cargo run --example test_get_state
```

## Supported Bitrates

### Classic CAN (87.5% sample point)

| Bitrate | 80 MHz | 40 MHz |
|---------|--------|--------|
| 10 kbps | ✓ | ✓ |
| 20 kbps | ✓ | ✓ |
| 50 kbps | ✓ | ✓ |
| 100 kbps | ✓ | ✓ |
| 125 kbps | ✓ | ✓ |
| 250 kbps | ✓ | ✓ |
| 500 kbps | ✓ | ✓ |
| 800 kbps | ✓ | ✓ |
| 1 Mbps | ✓ | ✓ |

### CAN FD Data Phase (75% sample point)

| Data Bitrate | 80 MHz | 40 MHz |
|--------------|--------|--------|
| 2 Mbps | ✓ | ✓ |
| 4 Mbps | ✓ | ✓ |
| 5 Mbps | ✓ | ✓ |
| 8 Mbps | ✓ | ✓ |
| 10 Mbps | - | ✓ |

## API Overview

### Device Discovery

```rust
// Scan for all GS-USB devices
let devices = GsUsb::scan()?;

// Find a specific device by bus and address
let device = GsUsb::find(1, 5)?;
```

### Configuration

```rust
// Set bitrate (classic CAN)
dev.set_bitrate(500000)?;

// Set CAN FD data bitrate
dev.set_data_bitrate(5000000)?;

// Set raw timing parameters
dev.set_timing(prop_seg, phase_seg1, phase_seg2, sjw, brp)?;
```

### Operating Modes

```rust
use gs_usb::*;

// Normal mode with hardware timestamps
dev.start(GS_CAN_MODE_NORMAL | GS_CAN_MODE_HW_TIMESTAMP)?;

// Listen-only mode (no ACKs)
dev.start(GS_CAN_MODE_LISTEN_ONLY)?;

// Loopback mode for testing
dev.start(GS_CAN_MODE_LOOP_BACK)?;

// CAN FD mode
dev.start(GS_CAN_MODE_NORMAL | GS_CAN_MODE_FD)?;
```

### Frame Types

```rust
use gs_usb::{GsUsbFrame, CAN_EFF_FLAG, CAN_RTR_FLAG};

// Standard frame (11-bit ID)
let frame = GsUsbFrame::with_data(0x123, &[0x01, 0x02, 0x03]);

// Extended frame (29-bit ID)
let frame = GsUsbFrame::with_data(0x12345678 | CAN_EFF_FLAG, &data);

// Remote transmission request
let frame = GsUsbFrame::with_data(0x123 | CAN_RTR_FLAG, &[]);

// CAN FD frame with BRS (bit rate switch)
let frame = GsUsbFrame::with_fd_data(0x123, &data_64_bytes, true);
```

### Device Information

```rust
// Get device info (channels, firmware version)
let info = dev.device_info()?;
println!("Channels: {}", info.channel_count());
println!("FW: {:.1}", info.firmware_version());

// Get device capabilities
let cap = dev.device_capability()?;
println!("Clock: {:.1} MHz", cap.clock_mhz());
println!("Features: 0x{:08x}", cap.feature);

// Check CAN FD support
if dev.supports_fd()? {
    println!("CAN FD is supported");
}

// Get bus state and error counters
if dev.supports_get_state()? {
    let state = dev.get_state(0)?;
    println!("State: {}", state.state_name());
    println!("RX errors: {}", state.rxerr);
    println!("TX errors: {}", state.txerr);
}
```

## Linux Permissions

To access USB devices without root on Linux, create a udev rule:

```bash
sudo tee /etc/udev/rules.d/99-gs_usb.rules << EOF
# GS-USB devices
SUBSYSTEM=="usb", ATTR{idVendor}=="1d50", ATTR{idProduct}=="606f", MODE="0666"
# candleLight
SUBSYSTEM=="usb", ATTR{idVendor}=="1209", ATTR{idProduct}=="2323", MODE="0666"
# CES CANext FD
SUBSYSTEM=="usb", ATTR{idVendor}=="1cd2", ATTR{idProduct}=="606f", MODE="0666"
# ABE CANdebugger FD
SUBSYSTEM=="usb", ATTR{idVendor}=="16d0", ATTR{idProduct}=="10b8", MODE="0666"
EOF

sudo udevadm control --reload-rules
sudo udevadm trigger
```

## License

MIT License

## Acknowledgments

This is a Rust port of the [Python gs_usb library](https://github.com/jxltom/gs_usb).