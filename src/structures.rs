//! GS-USB protocol structures
//!
//! This module contains the data structures used in the GS-USB protocol
//! for device configuration, bit timing, and state management.

use crate::constants::{
    can_state_name, GS_CAN_STATE_BUS_OFF, GS_CAN_STATE_ERROR_ACTIVE, GS_CAN_STATE_ERROR_PASSIVE,
    GS_CAN_STATE_ERROR_WARNING,
};

/// Device mode configuration
///
/// Used to start or stop the CAN channel with specific mode flags.
#[derive(Debug, Clone, Copy)]
pub struct DeviceMode {
    /// Mode value (0 = reset/stop, 1 = start)
    pub mode: u32,
    /// Mode flags (combination of GS_CAN_MODE_* constants)
    pub flags: u32,
}

impl DeviceMode {
    /// Create a new device mode configuration
    pub fn new(mode: u32, flags: u32) -> Self {
        Self { mode, flags }
    }

    /// Pack into bytes for USB transfer
    pub fn pack(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&self.mode.to_le_bytes());
        buf[4..8].copy_from_slice(&self.flags.to_le_bytes());
        buf
    }
}

impl std::fmt::Display for DeviceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mode: {}\nFlags: 0x{:08x}", self.mode, self.flags)
    }
}

/// CAN bit timing configuration
///
/// Used to configure the bit timing parameters for both
/// nominal (arbitration) phase and data phase (CAN FD).
#[derive(Debug, Clone, Copy)]
pub struct DeviceBitTiming {
    /// Propagation segment (typically 1)
    pub prop_seg: u32,
    /// Phase segment 1
    pub phase_seg1: u32,
    /// Phase segment 2
    pub phase_seg2: u32,
    /// Synchronization jump width
    pub sjw: u32,
    /// Baud rate prescaler
    pub brp: u32,
}

impl DeviceBitTiming {
    /// Create a new bit timing configuration
    pub fn new(prop_seg: u32, phase_seg1: u32, phase_seg2: u32, sjw: u32, brp: u32) -> Self {
        Self {
            prop_seg,
            phase_seg1,
            phase_seg2,
            sjw,
            brp,
        }
    }

    /// Pack into bytes for USB transfer
    pub fn pack(&self) -> [u8; 20] {
        let mut buf = [0u8; 20];
        buf[0..4].copy_from_slice(&self.prop_seg.to_le_bytes());
        buf[4..8].copy_from_slice(&self.phase_seg1.to_le_bytes());
        buf[8..12].copy_from_slice(&self.phase_seg2.to_le_bytes());
        buf[12..16].copy_from_slice(&self.sjw.to_le_bytes());
        buf[16..20].copy_from_slice(&self.brp.to_le_bytes());
        buf
    }
}

impl std::fmt::Display for DeviceBitTiming {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Prop Seg: {}\nPhase Seg 1: {}\nPhase Seg 2: {}\nSJW: {}\nBRP: {}",
            self.prop_seg, self.phase_seg1, self.phase_seg2, self.sjw, self.brp
        )
    }
}

/// Device information
///
/// Contains device metadata including channel count and version information.
#[derive(Debug, Clone, Copy)]
pub struct DeviceInfo {
    /// Reserved byte 1
    pub reserved1: u8,
    /// Reserved byte 2
    pub reserved2: u8,
    /// Reserved byte 3
    pub reserved3: u8,
    /// Interface count (number of CAN channels - 1)
    pub icount: u8,
    /// Firmware version (multiply by 0.1 for actual version)
    pub fw_version: u32,
    /// Hardware version (multiply by 0.1 for actual version)
    pub hw_version: u32,
}

impl DeviceInfo {
    /// Unpack from bytes received via USB
    pub fn unpack(data: &[u8]) -> Self {
        Self {
            reserved1: data[0],
            reserved2: data[1],
            reserved3: data[2],
            icount: data[3],
            fw_version: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            hw_version: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        }
    }

    /// Get the number of CAN channels
    pub fn channel_count(&self) -> u8 {
        self.icount + 1
    }

    /// Get firmware version as a float
    pub fn firmware_version(&self) -> f32 {
        self.fw_version as f32 / 10.0
    }

    /// Get hardware version as a float
    pub fn hardware_version(&self) -> f32 {
        self.hw_version as f32 / 10.0
    }
}

impl std::fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "iCount: {}\nFW Version: {:.1}\nHW Version: {:.1}",
            self.icount,
            self.firmware_version(),
            self.hardware_version()
        )
    }
}

/// Device capability including bit timing constraints
///
/// Supports both classic CAN (BT_CONST) and CAN FD (BT_CONST_EXT) devices.
/// When created from BT_CONST_EXT data, the data phase timing fields are populated.
#[derive(Debug, Clone, Copy)]
pub struct DeviceCapability {
    /// Feature bitfield (combination of GS_CAN_FEATURE_* constants)
    pub feature: u32,
    /// CAN clock frequency in Hz
    pub fclk_can: u32,
    /// Minimum TSEG1 value
    pub tseg1_min: u32,
    /// Maximum TSEG1 value
    pub tseg1_max: u32,
    /// Minimum TSEG2 value
    pub tseg2_min: u32,
    /// Maximum TSEG2 value
    pub tseg2_max: u32,
    /// Maximum SJW value
    pub sjw_max: u32,
    /// Minimum BRP value
    pub brp_min: u32,
    /// Maximum BRP value
    pub brp_max: u32,
    /// BRP increment value
    pub brp_inc: u32,

    // CAN FD data phase timing (optional)
    /// Minimum data phase TSEG1 value
    pub dtseg1_min: Option<u32>,
    /// Maximum data phase TSEG1 value
    pub dtseg1_max: Option<u32>,
    /// Minimum data phase TSEG2 value
    pub dtseg2_min: Option<u32>,
    /// Maximum data phase TSEG2 value
    pub dtseg2_max: Option<u32>,
    /// Maximum data phase SJW value
    pub dsjw_max: Option<u32>,
    /// Minimum data phase BRP value
    pub dbrp_min: Option<u32>,
    /// Maximum data phase BRP value
    pub dbrp_max: Option<u32>,
    /// Data phase BRP increment value
    pub dbrp_inc: Option<u32>,
}

impl DeviceCapability {
    /// Unpack from BT_CONST response (40 bytes, 10 x uint32)
    pub fn unpack(data: &[u8]) -> Self {
        Self {
            feature: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            fclk_can: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            tseg1_min: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            tseg1_max: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
            tseg2_min: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
            tseg2_max: u32::from_le_bytes([data[20], data[21], data[22], data[23]]),
            sjw_max: u32::from_le_bytes([data[24], data[25], data[26], data[27]]),
            brp_min: u32::from_le_bytes([data[28], data[29], data[30], data[31]]),
            brp_max: u32::from_le_bytes([data[32], data[33], data[34], data[35]]),
            brp_inc: u32::from_le_bytes([data[36], data[37], data[38], data[39]]),
            dtseg1_min: None,
            dtseg1_max: None,
            dtseg2_min: None,
            dtseg2_max: None,
            dsjw_max: None,
            dbrp_min: None,
            dbrp_max: None,
            dbrp_inc: None,
        }
    }

    /// Unpack from BT_CONST_EXT response (72 bytes, 18 x uint32)
    pub fn unpack_extended(data: &[u8]) -> Self {
        let mut cap = Self::unpack(data);
        cap.dtseg1_min = Some(u32::from_le_bytes([data[40], data[41], data[42], data[43]]));
        cap.dtseg1_max = Some(u32::from_le_bytes([data[44], data[45], data[46], data[47]]));
        cap.dtseg2_min = Some(u32::from_le_bytes([data[48], data[49], data[50], data[51]]));
        cap.dtseg2_max = Some(u32::from_le_bytes([data[52], data[53], data[54], data[55]]));
        cap.dsjw_max = Some(u32::from_le_bytes([data[56], data[57], data[58], data[59]]));
        cap.dbrp_min = Some(u32::from_le_bytes([data[60], data[61], data[62], data[63]]));
        cap.dbrp_max = Some(u32::from_le_bytes([data[64], data[65], data[66], data[67]]));
        cap.dbrp_inc = Some(u32::from_le_bytes([data[68], data[69], data[70], data[71]]));
        cap
    }

    /// Check if CAN FD data phase timing is available
    pub fn has_fd_timing(&self) -> bool {
        self.dtseg1_min.is_some()
    }

    /// Get clock frequency in MHz
    pub fn clock_mhz(&self) -> f32 {
        self.fclk_can as f32 / 1_000_000.0
    }
}

impl std::fmt::Display for DeviceCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Feature bitfield: 0x{:08x}\n\
             Clock: {} Hz ({:.1} MHz)\n\
             TSEG1: {} - {}\n\
             TSEG2: {} - {}\n\
             SJW (max): {}\n\
             BRP: {} - {} (inc: {})",
            self.feature,
            self.fclk_can,
            self.clock_mhz(),
            self.tseg1_min,
            self.tseg1_max,
            self.tseg2_min,
            self.tseg2_max,
            self.sjw_max,
            self.brp_min,
            self.brp_max,
            self.brp_inc
        )?;

        if self.has_fd_timing() {
            write!(
                f,
                "\nData Phase (CAN FD):\n\
                   DTSEG1: {} - {}\n\
                   DTSEG2: {} - {}\n\
                   DSJW (max): {}\n\
                   DBRP: {} - {} (inc: {})",
                self.dtseg1_min.unwrap(),
                self.dtseg1_max.unwrap(),
                self.dtseg2_min.unwrap(),
                self.dtseg2_max.unwrap(),
                self.dsjw_max.unwrap(),
                self.dbrp_min.unwrap(),
                self.dbrp_max.unwrap(),
                self.dbrp_inc.unwrap()
            )?;
        }

        Ok(())
    }
}

/// CAN device state from GS_USB_BREQ_GET_STATE response
///
/// Contains the current CAN bus state and error counters.
#[derive(Debug, Clone, Copy)]
pub struct DeviceState {
    /// CAN state enum value
    pub state: u32,
    /// RX error counter
    pub rxerr: u32,
    /// TX error counter
    pub txerr: u32,
}

impl DeviceState {
    /// Unpack from GET_STATE response (12 bytes, 3 x uint32)
    pub fn unpack(data: &[u8]) -> Self {
        Self {
            state: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            rxerr: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            txerr: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        }
    }

    /// Get human-readable state name
    pub fn state_name(&self) -> &'static str {
        can_state_name(self.state)
    }

    /// Check if in normal operation (error active state)
    pub fn is_error_active(&self) -> bool {
        self.state == GS_CAN_STATE_ERROR_ACTIVE
    }

    /// Check if in error warning state (TEC/REC > 96)
    pub fn is_error_warning(&self) -> bool {
        self.state == GS_CAN_STATE_ERROR_WARNING
    }

    /// Check if in error passive state (TEC/REC > 127)
    pub fn is_error_passive(&self) -> bool {
        self.state == GS_CAN_STATE_ERROR_PASSIVE
    }

    /// Check if bus is off (TEC > 255)
    pub fn is_bus_off(&self) -> bool {
        self.state == GS_CAN_STATE_BUS_OFF
    }
}

impl std::fmt::Display for DeviceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "State: {}\nRX Error Counter: {}\nTX Error Counter: {}",
            self.state_name(),
            self.rxerr,
            self.txerr
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_mode_pack() {
        let mode = DeviceMode::new(1, 0x100);
        let packed = mode.pack();
        assert_eq!(packed[0..4], [1, 0, 0, 0]);
        assert_eq!(packed[4..8], [0, 1, 0, 0]);
    }

    #[test]
    fn test_device_bit_timing_pack() {
        let timing = DeviceBitTiming::new(1, 12, 2, 1, 6);
        let packed = timing.pack();
        assert_eq!(packed[0..4], [1, 0, 0, 0]); // prop_seg
        assert_eq!(packed[4..8], [12, 0, 0, 0]); // phase_seg1
        assert_eq!(packed[8..12], [2, 0, 0, 0]); // phase_seg2
        assert_eq!(packed[12..16], [1, 0, 0, 0]); // sjw
        assert_eq!(packed[16..20], [6, 0, 0, 0]); // brp
    }

    #[test]
    fn test_device_info_unpack() {
        let data = [0, 0, 0, 1, 20, 0, 0, 0, 10, 0, 0, 0];
        let info = DeviceInfo::unpack(&data);
        assert_eq!(info.icount, 1);
        assert_eq!(info.channel_count(), 2);
        assert_eq!(info.fw_version, 20);
        assert_eq!(info.firmware_version(), 2.0);
        assert_eq!(info.hw_version, 10);
        assert_eq!(info.hardware_version(), 1.0);
    }

    #[test]
    fn test_device_state_unpack() {
        let data = [1, 0, 0, 0, 50, 0, 0, 0, 25, 0, 0, 0];
        let state = DeviceState::unpack(&data);
        assert_eq!(state.state, 1);
        assert_eq!(state.rxerr, 50);
        assert_eq!(state.txerr, 25);
        assert!(state.is_error_warning());
    }
}
