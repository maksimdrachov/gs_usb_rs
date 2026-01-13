//! GS-USB Protocol Implementation for Rust
//!
//! This crate provides a Rust implementation of the GS-USB protocol for communicating
//! with CAN bus adapters like candleLight, CANable, and other GS-USB compatible devices.
//!
//! # Features
//!
//! - Support for classic CAN (up to 1 Mbps)
//! - Support for CAN FD (up to 10 Mbps data rate)
//! - Hardware timestamps
//! - Multiple operating modes (normal, listen-only, loopback, one-shot)
//! - Device state and error counter monitoring
//!
//! # Example
//!
//! ```no_run
//! use gs_usb::{GsUsb, GsUsbFrame, GS_CAN_MODE_NORMAL, GS_CAN_MODE_HW_TIMESTAMP};
//! use std::time::Duration;
//!
//! fn main() -> gs_usb::Result<()> {
//!     // Scan for devices
//!     let devices = GsUsb::scan()?;
//!     if devices.is_empty() {
//!         println!("No GS-USB device found");
//!         return Ok(());
//!     }
//!
//!     let mut dev = devices.into_iter().next().unwrap();
//!
//!     // Configure bitrate (250 kbps)
//!     dev.set_bitrate(250000)?;
//!
//!     // Start the device
//!     dev.start(GS_CAN_MODE_NORMAL | GS_CAN_MODE_HW_TIMESTAMP)?;
//!
//!     // Send a frame
//!     let data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
//!     let frame = GsUsbFrame::with_data(0x7FF, &data);
//!     dev.send(&frame)?;
//!
//!     // Read frames
//!     loop {
//!         match dev.read(Duration::from_millis(100)) {
//!             Ok(frame) => println!("RX  {}", frame),
//!             Err(gs_usb::GsUsbError::ReadTimeout) => continue,
//!             Err(e) => return Err(e),
//!         }
//!     }
//! }
//! ```
//!
//! # Supported Devices
//!
//! - GS-USB devices (VID: 0x1D50, PID: 0x606F)
//! - candleLight (VID: 0x1209, PID: 0x2323)
//! - CES CANext FD (VID: 0x1CD2, PID: 0x606F)
//! - ABE CANdebugger FD (VID: 0x16D0, PID: 0x10B8)

pub mod constants;
pub mod device;
pub mod error;
pub mod frame;
pub mod structures;

// Re-export main types at crate root
pub use constants::{
    // CAN ID flags
    CAN_EFF_FLAG,
    // CAN ID masks
    CAN_EFF_MASK,
    CAN_ERR_FLAG,
    CAN_ERR_MASK,
    CAN_RTR_FLAG,
    CAN_SFF_MASK,
    // Feature flags
    GS_CAN_FEATURE_BERR_REPORTING,
    GS_CAN_FEATURE_BT_CONST_EXT,
    GS_CAN_FEATURE_FD,
    GS_CAN_FEATURE_GET_STATE,
    GS_CAN_FEATURE_HW_TIMESTAMP,
    GS_CAN_FEATURE_IDENTIFY,
    GS_CAN_FEATURE_LISTEN_ONLY,
    GS_CAN_FEATURE_LOOP_BACK,
    GS_CAN_FEATURE_ONE_SHOT,
    GS_CAN_FEATURE_PAD_PKTS_TO_MAX_PKT_SIZE,
    GS_CAN_FEATURE_REQ_USB_QUIRK_LPC546XX,
    GS_CAN_FEATURE_TERMINATION,
    GS_CAN_FEATURE_TRIPLE_SAMPLE,
    GS_CAN_FEATURE_USER_ID,
    // Frame flags
    GS_CAN_FLAG_BRS,
    GS_CAN_FLAG_ESI,
    GS_CAN_FLAG_FD,
    GS_CAN_FLAG_OVERFLOW,
    // Mode flags
    GS_CAN_MODE_BERR_REPORTING,
    GS_CAN_MODE_FD,
    GS_CAN_MODE_HW_TIMESTAMP,
    GS_CAN_MODE_IDENTIFY,
    GS_CAN_MODE_LISTEN_ONLY,
    GS_CAN_MODE_LOOP_BACK,
    GS_CAN_MODE_NORMAL,
    GS_CAN_MODE_ONE_SHOT,
    GS_CAN_MODE_PAD_PKTS_TO_MAX_PKT_SIZE,
    GS_CAN_MODE_TRIPLE_SAMPLE,
    GS_CAN_MODE_USER_ID,
    // CAN state constants
    GS_CAN_STATE_BUS_OFF,
    GS_CAN_STATE_ERROR_ACTIVE,
    GS_CAN_STATE_ERROR_PASSIVE,
    GS_CAN_STATE_ERROR_WARNING,
    GS_CAN_STATE_SLEEPING,
    GS_CAN_STATE_STOPPED,
};

pub use device::GsUsb;
pub use error::{GsUsbError, Result};
pub use frame::GsUsbFrame;
pub use structures::{DeviceBitTiming, DeviceCapability, DeviceInfo, DeviceMode, DeviceState};
