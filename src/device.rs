//! GS-USB device implementation
//!
//! This module provides the `GsUsb` struct for interfacing with GS-USB compatible
//! CAN adapters, including candleLight, CANable, and similar devices.

use std::time::Duration;

use rusb::{DeviceHandle, GlobalContext};

use crate::constants::*;
use crate::error::{GsUsbError, Result};
use crate::frame::GsUsbFrame;
use crate::structures::{DeviceBitTiming, DeviceCapability, DeviceInfo, DeviceMode, DeviceState};

/// GS-USB device handle
///
/// Provides methods for interacting with GS-USB compatible CAN adapters.
///
/// # Example
///
/// ```no_run
/// use gs_usb::{GsUsb, GsUsbFrame};
///
/// // Scan for devices
/// let devices = GsUsb::scan()?;
/// if devices.is_empty() {
///     println!("No GS-USB device found");
///     return Ok(());
/// }
///
/// let mut dev = devices.into_iter().next().unwrap();
///
/// // Configure bitrate
/// dev.set_bitrate(250000)?;
///
/// // Start the device
/// dev.start(gs_usb::GS_CAN_MODE_NORMAL | gs_usb::GS_CAN_MODE_HW_TIMESTAMP)?;
///
/// // Send a frame
/// let frame = GsUsbFrame::with_data(0x123, &[0x01, 0x02, 0x03, 0x04]);
/// dev.send(&frame)?;
///
/// // Read frames
/// loop {
///     match dev.read(std::time::Duration::from_millis(100)) {
///         Ok(frame) => println!("RX: {}", frame),
///         Err(gs_usb::GsUsbError::ReadTimeout) => continue,
///         Err(e) => return Err(e),
///     }
/// }
/// # Ok::<(), gs_usb::GsUsbError>(())
/// ```
pub struct GsUsb {
    /// USB device handle
    handle: DeviceHandle<GlobalContext>,
    /// Cached device capability
    capability: Option<DeviceCapability>,
    /// Current device flags
    device_flags: u32,
    /// Whether FD mode is enabled
    fd_mode: bool,
    /// Whether the device has been started
    started: bool,
    /// USB bus number
    bus: u8,
    /// USB device address
    address: u8,
    /// Device serial number (cached)
    serial_number: Option<String>,
    /// Last nominal (arbitration) phase bit timing that was set
    last_timing: Option<DeviceBitTiming>,
    /// Last data phase (CAN FD) bit timing that was set
    last_data_timing: Option<DeviceBitTiming>,
}

impl GsUsb {
    /// Create a new GsUsb from a USB device handle
    fn new(handle: DeviceHandle<GlobalContext>, bus: u8, address: u8) -> Self {
        Self {
            handle,
            capability: None,
            device_flags: 0,
            fd_mode: false,
            started: false,
            bus,
            address,
            serial_number: None,
            last_timing: None,
            last_data_timing: None,
        }
    }

    /// Start the GS-USB device
    ///
    /// # Arguments
    /// * `flags` - Mode flags (combination of GS_CAN_MODE_* constants)
    ///
    /// # Example
    /// ```no_run
    /// # use gs_usb::GsUsb;
    /// # let mut dev: GsUsb = todo!();
    /// // Start with hardware timestamps and normal mode
    /// dev.start(gs_usb::GS_CAN_MODE_NORMAL | gs_usb::GS_CAN_MODE_HW_TIMESTAMP)?;
    /// # Ok::<(), gs_usb::GsUsbError>(())
    /// ```
    pub fn start(&mut self, flags: u32) -> Result<()> {
        // Reset to support restart multiple times
        self.handle.reset()?;

        // Detach kernel driver on Linux/Unix
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            if self.handle.kernel_driver_active(0).unwrap_or(false) {
                self.handle
                    .detach_kernel_driver(0)
                    .map_err(GsUsbError::DetachKernelDriver)?;
            }
        }

        // Claim the interface
        self.handle
            .claim_interface(0)
            .map_err(GsUsbError::ClaimInterface)?;

        // Get capability to check supported features
        let capability = self.device_capability()?;

        // Only allow features that the device supports
        let mut flags = flags & capability.feature;

        // Only allow features that this driver supports
        flags &= GS_CAN_MODE_LISTEN_ONLY
            | GS_CAN_MODE_LOOP_BACK
            | GS_CAN_MODE_ONE_SHOT
            | GS_CAN_MODE_HW_TIMESTAMP
            | GS_CAN_MODE_FD;

        self.device_flags = flags;
        self.fd_mode = (flags & GS_CAN_MODE_FD) == GS_CAN_MODE_FD;

        let mode = DeviceMode::new(GS_CAN_MODE_START, flags);
        self.control_out(GS_USB_BREQ_MODE, 0, &mode.pack())?;

        self.started = true;
        Ok(())
    }

    /// Stop the GS-USB device
    pub fn stop(&mut self) -> Result<()> {
        let mode = DeviceMode::new(GS_CAN_MODE_RESET, 0);
        // Ignore errors when stopping (device might already be stopped)
        let _ = self.control_out(GS_USB_BREQ_MODE, 0, &mode.pack());
        self.started = false;
        Ok(())
    }

    /// Set the CAN bitrate
    ///
    /// Configures the nominal (arbitration) bitrate with a sample point of 87.5%.
    ///
    /// # Arguments
    /// * `bitrate` - Bitrate in bits per second (e.g., 250000 for 250 kbps)
    ///
    /// # Supported bitrates
    /// - 10000 (10 kbps)
    /// - 20000 (20 kbps)
    /// - 50000 (50 kbps)
    /// - 100000 (100 kbps)
    /// - 125000 (125 kbps)
    /// - 250000 (250 kbps)
    /// - 500000 (500 kbps)
    /// - 800000 (800 kbps)
    /// - 1000000 (1 Mbps)
    pub fn set_bitrate(&mut self, bitrate: u32) -> Result<()> {
        self.set_bitrate_with_sample_point(bitrate, 87.5)
    }

    /// Set the CAN bitrate with a specific sample point
    ///
    /// # Arguments
    /// * `bitrate` - Bitrate in bits per second
    /// * `sample_point` - Sample point percentage (typically 87.5%)
    pub fn set_bitrate_with_sample_point(&mut self, bitrate: u32, sample_point: f32) -> Result<()> {
        let capability = self.device_capability()?;
        let clock = capability.fclk_can;

        let prop_seg = 1;
        let sjw = 1;

        // Get timing parameters based on clock and sample point
        let timing = match (clock, (sample_point * 10.0) as u32) {
            // 48 MHz clock, 87.5% sample point
            (48_000_000, 875) => match bitrate {
                10_000 => Some((prop_seg, 12, 2, sjw, 300)),
                20_000 => Some((prop_seg, 12, 2, sjw, 150)),
                50_000 => Some((prop_seg, 12, 2, sjw, 60)),
                100_000 => Some((prop_seg, 12, 2, sjw, 30)),
                125_000 => Some((prop_seg, 12, 2, sjw, 24)),
                250_000 => Some((prop_seg, 12, 2, sjw, 12)),
                500_000 => Some((prop_seg, 12, 2, sjw, 6)),
                800_000 => Some((prop_seg, 11, 2, sjw, 4)),
                1_000_000 => Some((prop_seg, 12, 2, sjw, 3)),
                _ => None,
            },
            // 80 MHz clock, 87.5% sample point
            (80_000_000, 875) => match bitrate {
                10_000 => Some((prop_seg, 12, 2, sjw, 500)),
                20_000 => Some((prop_seg, 12, 2, sjw, 250)),
                50_000 => Some((prop_seg, 12, 2, sjw, 100)),
                100_000 => Some((prop_seg, 12, 2, sjw, 50)),
                125_000 => Some((prop_seg, 12, 2, sjw, 40)),
                250_000 => Some((prop_seg, 12, 2, sjw, 20)),
                500_000 => Some((prop_seg, 12, 2, sjw, 10)),
                800_000 => Some((prop_seg, 7, 1, sjw, 10)),
                1_000_000 => Some((prop_seg, 12, 2, sjw, 5)),
                _ => None,
            },
            // 40 MHz clock, 87.5% sample point (CF3 / TCAN4550)
            (40_000_000, 875) => match bitrate {
                10_000 => Some((prop_seg, 12, 2, sjw, 250)),
                20_000 => Some((prop_seg, 12, 2, sjw, 125)),
                50_000 => Some((prop_seg, 12, 2, sjw, 50)),
                100_000 => Some((prop_seg, 12, 2, sjw, 25)),
                125_000 => Some((prop_seg, 12, 2, sjw, 20)),
                250_000 => Some((prop_seg, 12, 2, sjw, 10)),
                500_000 => Some((prop_seg, 12, 2, sjw, 5)),
                800_000 => Some((prop_seg, 7, 1, sjw, 5)),
                1_000_000 => Some((prop_seg, 5, 1, sjw, 5)),
                _ => None,
            },
            _ => None,
        };

        match timing {
            Some((prop_seg, phase_seg1, phase_seg2, sjw, brp)) => {
                self.set_timing(prop_seg, phase_seg1, phase_seg2, sjw, brp)
            }
            None => Err(GsUsbError::UnsupportedBitrate {
                bitrate,
                clock_hz: clock,
            }),
        }
    }

    /// Set raw CAN bit timing parameters
    ///
    /// # Arguments
    /// * `prop_seg` - Propagation segment (typically 1)
    /// * `phase_seg1` - Phase segment 1 (1-15)
    /// * `phase_seg2` - Phase segment 2 (1-8)
    /// * `sjw` - Synchronization jump width (1-4)
    /// * `brp` - Baud rate prescaler (1-1024)
    pub fn set_timing(
        &mut self,
        prop_seg: u32,
        phase_seg1: u32,
        phase_seg2: u32,
        sjw: u32,
        brp: u32,
    ) -> Result<()> {
        let timing = DeviceBitTiming::new(prop_seg, phase_seg1, phase_seg2, sjw, brp);
        self.control_out(GS_USB_BREQ_BITTIMING, 0, &timing.pack())?;
        self.last_timing = Some(timing);
        Ok(())
    }

    /// Set CAN FD data phase bit timing parameters
    pub fn set_data_timing(
        &mut self,
        prop_seg: u32,
        phase_seg1: u32,
        phase_seg2: u32,
        sjw: u32,
        brp: u32,
    ) -> Result<()> {
        let timing = DeviceBitTiming::new(prop_seg, phase_seg1, phase_seg2, sjw, brp);
        self.control_out(GS_USB_BREQ_DATA_BITTIMING, 0, &timing.pack())?;
        self.last_data_timing = Some(timing);
        Ok(())
    }

    /// Get the last nominal (arbitration) phase timing that was set via `set_timing`/`set_bitrate*`
    pub fn last_timing(&self) -> Option<DeviceBitTiming> {
        self.last_timing
    }

    /// Get the last data phase (CAN FD) timing that was set via `set_data_timing`/`set_data_bitrate*`
    pub fn last_data_timing(&self) -> Option<DeviceBitTiming> {
        self.last_data_timing
    }

    /// Set CAN FD data phase bitrate
    ///
    /// Common data bitrates: 2 Mbps, 4 Mbps, 5 Mbps, 8 Mbps, 10 Mbps
    ///
    /// # Arguments
    /// * `bitrate` - Data phase bitrate in bits per second
    pub fn set_data_bitrate(&mut self, bitrate: u32) -> Result<()> {
        self.set_data_bitrate_with_sample_point(bitrate, 75.0)
    }

    /// Set CAN FD data phase bitrate with a specific sample point
    pub fn set_data_bitrate_with_sample_point(
        &mut self,
        bitrate: u32,
        sample_point: f32,
    ) -> Result<()> {
        let capability = self.device_capability()?;

        // Check if device supports CAN FD
        if (capability.feature & GS_CAN_FEATURE_FD) == 0 {
            return Err(GsUsbError::FdNotSupported);
        }

        let clock = capability.fclk_can;
        let prop_seg = 1;
        let sjw = 1;

        // Get timing parameters based on clock
        let timing = match (clock, (sample_point * 10.0) as u32) {
            // 80 MHz clock, 75% sample point
            (80_000_000, 750) => match bitrate {
                2_000_000 => Some((prop_seg, 4, 2, sjw, 5)),
                4_000_000 => Some((prop_seg, 1, 1, sjw, 5)),
                5_000_000 => Some((prop_seg, 4, 2, sjw, 2)),
                8_000_000 => Some((prop_seg, 2, 1, sjw, 2)),
                _ => None,
            },
            // 40 MHz clock, 75-80% sample point (TCAN4550/CF3)
            (40_000_000, 750) => match bitrate {
                2_000_000 => Some((prop_seg, 6, 2, sjw, 2)),
                4_000_000 => Some((prop_seg, 2, 1, sjw, 2)),
                5_000_000 => Some((prop_seg, 4, 2, sjw, 1)),
                8_000_000 => Some((prop_seg, 2, 1, sjw, 1)),
                10_000_000 => Some((prop_seg, 1, 1, sjw, 1)),
                _ => None,
            },
            _ => None,
        };

        match timing {
            Some((prop_seg, phase_seg1, phase_seg2, sjw, brp)) => {
                self.set_data_timing(prop_seg, phase_seg1, phase_seg2, sjw, brp)
            }
            None => Err(GsUsbError::UnsupportedDataBitrate {
                bitrate,
                clock_hz: clock,
            }),
        }
    }

    /// Send a CAN frame
    ///
    /// # Arguments
    /// * `frame` - The CAN frame to send
    pub fn send(&mut self, frame: &GsUsbFrame) -> Result<()> {
        let hw_timestamps = (self.device_flags & GS_CAN_MODE_HW_TIMESTAMP) != 0;
        let data = frame.pack(hw_timestamps, self.fd_mode);

        self.handle
            .write_bulk(GS_USB_ENDPOINT_OUT, &data, Duration::from_millis(1000))
            .map_err(GsUsbError::BulkTransfer)?;

        Ok(())
    }

    /// Read a CAN frame
    ///
    /// # Arguments
    /// * `timeout` - Read timeout duration
    ///
    /// # Returns
    /// The received CAN frame, or an error if timeout or other failure
    pub fn read(&mut self, timeout: Duration) -> Result<GsUsbFrame> {
        let hw_timestamps = (self.device_flags & GS_CAN_MODE_HW_TIMESTAMP) != 0;
        let max_size = GsUsbFrame::frame_size(hw_timestamps, self.fd_mode);

        let mut buf = vec![0u8; max_size];
        let len = match self.handle.read_bulk(GS_USB_ENDPOINT_IN, &mut buf, timeout) {
            Ok(len) => len,
            Err(rusb::Error::Timeout) => return Err(GsUsbError::ReadTimeout),
            Err(e) => return Err(GsUsbError::BulkTransfer(e)),
        };

        // Determine if this is an FD frame by checking the flags byte (offset 10)
        let is_fd_frame = if len >= 11 {
            (buf[10] & GS_CAN_FLAG_FD) != 0
        } else {
            false
        };

        Ok(GsUsbFrame::from_bytes(
            &buf[..len],
            hw_timestamps,
            is_fd_frame,
        ))
    }

    /// Get the USB bus number
    pub fn bus(&self) -> u8 {
        self.bus
    }

    /// Get the USB device address
    pub fn address(&self) -> u8 {
        self.address
    }

    /// Get the device serial number
    pub fn serial_number(&mut self) -> Result<String> {
        if let Some(ref sn) = self.serial_number {
            return Ok(sn.clone());
        }

        let device = self.handle.device();
        let desc = device.device_descriptor()?;

        if desc.serial_number_string_index().is_some() {
            let sn = self
                .handle
                .read_string_descriptor_ascii(desc.serial_number_string_index().unwrap())?;
            self.serial_number = Some(sn.clone());
            Ok(sn)
        } else {
            Ok(String::new())
        }
    }

    /// Get device information (channel count, firmware/hardware version)
    pub fn device_info(&mut self) -> Result<DeviceInfo> {
        let data = self.control_in(GS_USB_BREQ_DEVICE_CONFIG, 0, 12)?;
        Ok(DeviceInfo::unpack(&data))
    }

    /// Get device capability (bit timing constraints, feature flags)
    pub fn device_capability(&mut self) -> Result<DeviceCapability> {
        if let Some(ref cap) = self.capability {
            return Ok(*cap);
        }

        let data = self.control_in(GS_USB_BREQ_BT_CONST, 0, 40)?;
        let cap = DeviceCapability::unpack(&data);
        self.capability = Some(cap);
        Ok(cap)
    }

    /// Get extended device capability (includes CAN FD timing constraints)
    ///
    /// Returns `None` if device doesn't support BT_CONST_EXT
    pub fn device_capability_extended(&mut self) -> Result<Option<DeviceCapability>> {
        let cap = self.device_capability()?;

        // Check if device supports extended capability
        if (cap.feature & GS_CAN_FEATURE_BT_CONST_EXT) == 0 {
            return Ok(None);
        }

        // If we already have extended capability cached, return it
        if let Some(ref cap) = self.capability {
            if cap.has_fd_timing() {
                return Ok(Some(*cap));
            }
        }

        // Fetch extended capability and replace the basic one
        let data = self.control_in(GS_USB_BREQ_BT_CONST_EXT, 0, 72)?;
        let cap = DeviceCapability::unpack_extended(&data);
        self.capability = Some(cap);
        Ok(Some(cap))
    }

    /// Check if device supports CAN FD
    pub fn supports_fd(&mut self) -> Result<bool> {
        let cap = self.device_capability()?;
        Ok((cap.feature & GS_CAN_FEATURE_FD) != 0)
    }

    /// Check if device supports GET_STATE request
    pub fn supports_get_state(&mut self) -> Result<bool> {
        let cap = self.device_capability()?;
        Ok((cap.feature & GS_CAN_FEATURE_GET_STATE) != 0)
    }

    /// Get CAN bus state and error counters
    ///
    /// # Arguments
    /// * `channel` - CAN channel number (default 0)
    pub fn get_state(&mut self, channel: u16) -> Result<DeviceState> {
        if !self.supports_get_state()? {
            return Err(GsUsbError::GetStateNotSupported);
        }

        let data = self.control_in(GS_USB_BREQ_GET_STATE, channel, 12)?;
        Ok(DeviceState::unpack(&data))
    }

    /// Send HOST_FORMAT request (legacy requirement)
    ///
    /// This sets the byte order for the device. Most modern devices
    /// don't require this, but it's included for compatibility.
    pub fn send_host_format(&mut self) -> Result<()> {
        let host_format: [u8; 4] = 0x0000_BEEFu32.to_le_bytes();
        // Ignore errors - this may fail on some devices that don't support it
        let _ = self.control_out(GS_USB_BREQ_HOST_FORMAT, 0, &host_format);
        Ok(())
    }

    /// Perform a control OUT transfer
    fn control_out(&self, request: u8, value: u16, data: &[u8]) -> Result<()> {
        self.handle
            .write_control(
                0x41, // bmRequestType: vendor, host-to-device
                request,
                value,
                0, // wIndex
                data,
                Duration::from_millis(1000),
            )
            .map_err(GsUsbError::ControlTransfer)?;
        Ok(())
    }

    /// Perform a control IN transfer
    fn control_in(&self, request: u8, value: u16, length: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; length];
        let len = self
            .handle
            .read_control(
                0xC1, // bmRequestType: vendor, device-to-host
                request,
                value,
                0, // wIndex
                &mut buf,
                Duration::from_millis(1000),
            )
            .map_err(GsUsbError::ControlTransfer)?;

        if len < length {
            return Err(GsUsbError::InvalidResponse {
                expected: length,
                actual: len,
            });
        }

        Ok(buf)
    }

    /// Check if a USB device is a GS-USB device
    fn is_gs_usb_device(vendor_id: u16, product_id: u16) -> bool {
        matches!(
            (vendor_id, product_id),
            (GS_USB_ID_VENDOR, GS_USB_ID_PRODUCT)
                | (GS_USB_CANDLELIGHT_VENDOR_ID, GS_USB_CANDLELIGHT_PRODUCT_ID)
                | (
                    GS_USB_CES_CANEXT_FD_VENDOR_ID,
                    GS_USB_CES_CANEXT_FD_PRODUCT_ID
                )
                | (
                    GS_USB_ABE_CANDEBUGGER_FD_VENDOR_ID,
                    GS_USB_ABE_CANDEBUGGER_FD_PRODUCT_ID
                )
        )
    }

    /// Scan for GS-USB devices
    ///
    /// Returns a list of all connected GS-USB compatible devices.
    pub fn scan() -> Result<Vec<GsUsb>> {
        let mut devices = Vec::new();

        for device in rusb::devices()?.iter() {
            let desc = match device.device_descriptor() {
                Ok(desc) => desc,
                Err(_) => continue,
            };

            if Self::is_gs_usb_device(desc.vendor_id(), desc.product_id()) {
                let handle = match device.open() {
                    Ok(handle) => handle,
                    Err(_) => continue,
                };

                devices.push(GsUsb::new(handle, device.bus_number(), device.address()));
            }
        }

        Ok(devices)
    }

    /// Find a specific GS-USB device by bus and address
    pub fn find(bus: u8, address: u8) -> Result<Option<GsUsb>> {
        for device in rusb::devices()?.iter() {
            if device.bus_number() != bus || device.address() != address {
                continue;
            }

            let desc = match device.device_descriptor() {
                Ok(desc) => desc,
                Err(_) => continue,
            };

            if Self::is_gs_usb_device(desc.vendor_id(), desc.product_id()) {
                let handle = device.open()?;
                return Ok(Some(GsUsb::new(handle, bus, address)));
            }
        }

        Ok(None)
    }
}

impl std::fmt::Display for GsUsb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let device = self.handle.device();
        if let Ok(desc) = device.device_descriptor() {
            write!(
                f,
                "GS-USB {:04x}:{:04x} (bus {}, addr {})",
                desc.vendor_id(),
                desc.product_id(),
                self.bus,
                self.address
            )
        } else {
            write!(f, "GS-USB (bus {}, addr {})", self.bus, self.address)
        }
    }
}

impl std::fmt::Debug for GsUsb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GsUsb")
            .field("bus", &self.bus)
            .field("address", &self.address)
            .field("started", &self.started)
            .field("fd_mode", &self.fd_mode)
            .field("device_flags", &format_args!("0x{:08x}", self.device_flags))
            .finish()
    }
}

impl Drop for GsUsb {
    fn drop(&mut self) {
        // Try to stop the device when dropped
        let _ = self.stop();
    }
}
