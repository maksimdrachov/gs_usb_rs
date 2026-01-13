//! Error types for the GS-USB library
//!
//! This module defines the error types used throughout the library
//! for handling USB communication and protocol errors.

use thiserror::Error;

/// Result type alias for GS-USB operations
pub type Result<T> = std::result::Result<T, GsUsbError>;

/// Error types for GS-USB operations
#[derive(Error, Debug)]
pub enum GsUsbError {
    /// USB error from the rusb library
    #[error("USB error: {0}")]
    Usb(#[from] rusb::Error),

    /// No GS-USB device found
    #[error("No GS-USB device found")]
    DeviceNotFound,

    /// Device is not open
    #[error("Device is not open")]
    DeviceNotOpen,

    /// Failed to claim interface
    #[error("Failed to claim USB interface: {0}")]
    ClaimInterface(rusb::Error),

    /// Failed to detach kernel driver
    #[error("Failed to detach kernel driver: {0}")]
    DetachKernelDriver(rusb::Error),

    /// Unsupported bitrate for the device clock
    #[error("Unsupported bitrate {bitrate} for clock {clock_hz} Hz")]
    UnsupportedBitrate { bitrate: u32, clock_hz: u32 },

    /// Unsupported data bitrate for CAN FD
    #[error("Unsupported data bitrate {bitrate} for clock {clock_hz} Hz")]
    UnsupportedDataBitrate { bitrate: u32, clock_hz: u32 },

    /// Device does not support CAN FD
    #[error("Device does not support CAN FD")]
    FdNotSupported,

    /// Device does not support the requested feature
    #[error("Device does not support feature: {0}")]
    FeatureNotSupported(&'static str),

    /// Timeout during read operation
    #[error("Read timeout")]
    ReadTimeout,

    /// Timeout during write operation
    #[error("Write timeout")]
    WriteTimeout,

    /// Invalid response from device
    #[error("Invalid response from device: expected {expected} bytes, got {actual}")]
    InvalidResponse { expected: usize, actual: usize },

    /// Control transfer failed
    #[error("Control transfer failed: {0}")]
    ControlTransfer(rusb::Error),

    /// Bulk transfer failed
    #[error("Bulk transfer failed: {0}")]
    BulkTransfer(rusb::Error),

    /// Device is already started
    #[error("Device is already started")]
    AlreadyStarted,

    /// Device is not started
    #[error("Device is not started")]
    NotStarted,

    /// Invalid channel number
    #[error("Invalid channel number: {channel} (device has {max_channels} channels)")]
    InvalidChannel { channel: u8, max_channels: u8 },

    /// GET_STATE feature not supported
    #[error("Device does not support GET_STATE feature")]
    GetStateNotSupported,
}

impl GsUsbError {
    /// Check if this error is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(
            self,
            GsUsbError::ReadTimeout
                | GsUsbError::WriteTimeout
                | GsUsbError::Usb(rusb::Error::Timeout)
        )
    }

    /// Check if this error is a USB error
    pub fn is_usb_error(&self) -> bool {
        matches!(
            self,
            GsUsbError::Usb(_)
                | GsUsbError::ClaimInterface(_)
                | GsUsbError::DetachKernelDriver(_)
                | GsUsbError::ControlTransfer(_)
                | GsUsbError::BulkTransfer(_)
        )
    }
}
