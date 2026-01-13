//! GS-USB protocol constants
//!
//! This module contains all the constants used in the GS-USB protocol,
//! including mode flags, feature flags, CAN frame flags, and state definitions.

// ============================================================================
// GS-USB Mode Flags (used in DeviceMode.flags)
// ============================================================================

/// Normal operation mode
pub const GS_CAN_MODE_NORMAL: u32 = 0;
/// Listen-only mode (no ACKs sent)
pub const GS_CAN_MODE_LISTEN_ONLY: u32 = 1 << 0;
/// Loopback mode (for testing)
pub const GS_CAN_MODE_LOOP_BACK: u32 = 1 << 1;
/// Triple sample mode
pub const GS_CAN_MODE_TRIPLE_SAMPLE: u32 = 1 << 2;
/// One-shot mode (no retransmission)
pub const GS_CAN_MODE_ONE_SHOT: u32 = 1 << 3;
/// Hardware timestamp mode
pub const GS_CAN_MODE_HW_TIMESTAMP: u32 = 1 << 4;
/// Identify mode (blink LED)
pub const GS_CAN_MODE_IDENTIFY: u32 = 1 << 5;
/// User ID mode
pub const GS_CAN_MODE_USER_ID: u32 = 1 << 6;
/// Pad packets to max packet size
pub const GS_CAN_MODE_PAD_PKTS_TO_MAX_PKT_SIZE: u32 = 1 << 7;
/// CAN FD mode
pub const GS_CAN_MODE_FD: u32 = 1 << 8;
/// Bus error reporting
pub const GS_CAN_MODE_BERR_REPORTING: u32 = 1 << 12;

// ============================================================================
// GS-USB Device Feature Flags (from BT_CONST response)
// ============================================================================

/// Device supports listen-only mode
pub const GS_CAN_FEATURE_LISTEN_ONLY: u32 = 1 << 0;
/// Device supports loopback mode
pub const GS_CAN_FEATURE_LOOP_BACK: u32 = 1 << 1;
/// Device supports triple sample mode
pub const GS_CAN_FEATURE_TRIPLE_SAMPLE: u32 = 1 << 2;
/// Device supports one-shot mode
pub const GS_CAN_FEATURE_ONE_SHOT: u32 = 1 << 3;
/// Device supports hardware timestamps
pub const GS_CAN_FEATURE_HW_TIMESTAMP: u32 = 1 << 4;
/// Device supports identify (LED blink)
pub const GS_CAN_FEATURE_IDENTIFY: u32 = 1 << 5;
/// Device supports user ID
pub const GS_CAN_FEATURE_USER_ID: u32 = 1 << 6;
/// Device supports packet padding
pub const GS_CAN_FEATURE_PAD_PKTS_TO_MAX_PKT_SIZE: u32 = 1 << 7;
/// Device supports CAN FD
pub const GS_CAN_FEATURE_FD: u32 = 1 << 8;
/// Device requires USB quirk for LPC546XX
pub const GS_CAN_FEATURE_REQ_USB_QUIRK_LPC546XX: u32 = 1 << 9;
/// Device supports extended bit timing constants
pub const GS_CAN_FEATURE_BT_CONST_EXT: u32 = 1 << 10;
/// Device supports termination control
pub const GS_CAN_FEATURE_TERMINATION: u32 = 1 << 11;
/// Device supports bus error reporting
pub const GS_CAN_FEATURE_BERR_REPORTING: u32 = 1 << 12;
/// Device supports GET_STATE request
pub const GS_CAN_FEATURE_GET_STATE: u32 = 1 << 13;

// ============================================================================
// CAN ID Flags (in CAN frame identifier)
// ============================================================================

/// Extended frame format flag (29-bit ID)
pub const CAN_EFF_FLAG: u32 = 0x8000_0000;
/// Remote transmission request flag
pub const CAN_RTR_FLAG: u32 = 0x4000_0000;
/// Error message frame flag
pub const CAN_ERR_FLAG: u32 = 0x2000_0000;

// ============================================================================
// CAN ID Masks
// ============================================================================

/// Standard frame format mask (11-bit ID)
pub const CAN_SFF_MASK: u32 = 0x0000_07FF;
/// Extended frame format mask (29-bit ID)
pub const CAN_EFF_MASK: u32 = 0x1FFF_FFFF;
/// Error mask (omit EFF, RTR, ERR flags)
pub const CAN_ERR_MASK: u32 = 0x1FFF_FFFF;

/// Number of bits in standard frame ID
pub const CAN_SFF_ID_BITS: u8 = 11;
/// Number of bits in extended frame ID
pub const CAN_EFF_ID_BITS: u8 = 29;

// ============================================================================
// CAN Payload Definitions
// ============================================================================

/// Maximum DLC for classic CAN
pub const CAN_MAX_DLC: u8 = 8;
/// Maximum data length for classic CAN
pub const CAN_MAX_DLEN: usize = 8;

/// Maximum DLC for CAN FD
pub const CANFD_MAX_DLC: u8 = 15;
/// Maximum data length for CAN FD
pub const CANFD_MAX_DLEN: usize = 64;

// ============================================================================
// GS-USB Frame Flags (in gs_host_frame.flags field)
// ============================================================================

/// RX overflow occurred
pub const GS_CAN_FLAG_OVERFLOW: u8 = 1 << 0;
/// CAN FD frame
pub const GS_CAN_FLAG_FD: u8 = 1 << 1;
/// Bit rate switch (FD frame transmitted at data bitrate)
pub const GS_CAN_FLAG_BRS: u8 = 1 << 2;
/// Error state indicator
pub const GS_CAN_FLAG_ESI: u8 = 1 << 3;

// ============================================================================
// DLC to Length Conversion for CAN FD
// ============================================================================

/// DLC to data length conversion table for CAN FD
pub const CANFD_DLC_TO_LEN: [usize; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 20, 24, 32, 48, 64];

// ============================================================================
// GS-USB Frame Constants
// ============================================================================

/// CAN ID length in bytes
pub const CAN_IDLEN: usize = 4;

/// Echo ID for transmitted frames
pub const GS_USB_ECHO_ID: u32 = 0;
/// Echo ID value for received frames (from CAN bus)
pub const GS_USB_RX_ECHO_ID: u32 = 0xFFFF_FFFF;

// ============================================================================
// Frame Sizes
// ============================================================================

/// Classic CAN frame size (without timestamp)
pub const GS_USB_FRAME_SIZE: usize = 20;
/// Classic CAN frame size (with hardware timestamp)
pub const GS_USB_FRAME_SIZE_HW_TIMESTAMP: usize = 24;

/// CAN FD frame size (without timestamp): 12-byte header + 64-byte data
pub const GS_USB_FRAME_SIZE_FD: usize = 76;
/// CAN FD frame size (with hardware timestamp)
pub const GS_USB_FRAME_SIZE_FD_HW_TIMESTAMP: usize = 80;

// ============================================================================
// CAN State Enum (from GS_USB_BREQ_GET_STATE)
// ============================================================================

/// Normal operation
pub const GS_CAN_STATE_ERROR_ACTIVE: u32 = 0;
/// TEC/REC > 96
pub const GS_CAN_STATE_ERROR_WARNING: u32 = 1;
/// TEC/REC > 127
pub const GS_CAN_STATE_ERROR_PASSIVE: u32 = 2;
/// TEC > 255
pub const GS_CAN_STATE_BUS_OFF: u32 = 3;
/// Device stopped
pub const GS_CAN_STATE_STOPPED: u32 = 4;
/// Device sleeping
pub const GS_CAN_STATE_SLEEPING: u32 = 5;

/// Get human-readable name for CAN state
pub fn can_state_name(state: u32) -> &'static str {
    match state {
        GS_CAN_STATE_ERROR_ACTIVE => "ERROR_ACTIVE",
        GS_CAN_STATE_ERROR_WARNING => "ERROR_WARNING",
        GS_CAN_STATE_ERROR_PASSIVE => "ERROR_PASSIVE",
        GS_CAN_STATE_BUS_OFF => "BUS_OFF",
        GS_CAN_STATE_STOPPED => "STOPPED",
        GS_CAN_STATE_SLEEPING => "SLEEPING",
        _ => "UNKNOWN",
    }
}

// ============================================================================
// USB Vendor/Product IDs
// ============================================================================

/// GS-USB default vendor ID
pub const GS_USB_ID_VENDOR: u16 = 0x1D50;
/// GS-USB default product ID
pub const GS_USB_ID_PRODUCT: u16 = 0x606F;

/// Candlelight vendor ID
pub const GS_USB_CANDLELIGHT_VENDOR_ID: u16 = 0x1209;
/// Candlelight product ID
pub const GS_USB_CANDLELIGHT_PRODUCT_ID: u16 = 0x2323;

/// CES CANext FD vendor ID
pub const GS_USB_CES_CANEXT_FD_VENDOR_ID: u16 = 0x1CD2;
/// CES CANext FD product ID
pub const GS_USB_CES_CANEXT_FD_PRODUCT_ID: u16 = 0x606F;

/// ABE CANdebugger FD vendor ID
pub const GS_USB_ABE_CANDEBUGGER_FD_VENDOR_ID: u16 = 0x16D0;
/// ABE CANdebugger FD product ID
pub const GS_USB_ABE_CANDEBUGGER_FD_PRODUCT_ID: u16 = 0x10B8;

// ============================================================================
// GS-USB Control Request Codes
// ============================================================================

/// Set host byte order (legacy)
pub const GS_USB_BREQ_HOST_FORMAT: u8 = 0;
/// Set bit timing
pub const GS_USB_BREQ_BITTIMING: u8 = 1;
/// Set/start mode
pub const GS_USB_BREQ_MODE: u8 = 2;
/// Get bus errors
pub const GS_USB_BREQ_BERR: u8 = 3;
/// Get bit timing constants
pub const GS_USB_BREQ_BT_CONST: u8 = 4;
/// Get device configuration
pub const GS_USB_BREQ_DEVICE_CONFIG: u8 = 5;
/// Get timestamp
pub const GS_USB_BREQ_TIMESTAMP: u8 = 6;
/// Identify device (blink LED)
pub const GS_USB_BREQ_IDENTIFY: u8 = 7;
/// Get user ID
pub const GS_USB_BREQ_GET_USER_ID: u8 = 8;
/// Set user ID
pub const GS_USB_BREQ_SET_USER_ID: u8 = 9;
/// Set data phase bit timing (CAN FD)
pub const GS_USB_BREQ_DATA_BITTIMING: u8 = 10;
/// Get extended bit timing constants (CAN FD)
pub const GS_USB_BREQ_BT_CONST_EXT: u8 = 11;
/// Set termination
pub const GS_USB_BREQ_SET_TERMINATION: u8 = 12;
/// Get termination
pub const GS_USB_BREQ_GET_TERMINATION: u8 = 13;
/// Get CAN state
pub const GS_USB_BREQ_GET_STATE: u8 = 14;

// ============================================================================
// GS-USB Mode Values
// ============================================================================

/// Reset/stop mode
pub const GS_CAN_MODE_RESET: u32 = 0;
/// Start mode
pub const GS_CAN_MODE_START: u32 = 1;

// ============================================================================
// USB Endpoints
// ============================================================================

/// Bulk OUT endpoint (host to device)
pub const GS_USB_ENDPOINT_OUT: u8 = 0x02;
/// Bulk IN endpoint (device to host)
pub const GS_USB_ENDPOINT_IN: u8 = 0x81;
