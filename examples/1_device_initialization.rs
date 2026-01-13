//! Device Configuration Example
//!
//! This script demonstrates the full device initialization cycle for a gs_usb device,
//! following the protocol flow described in the gs_usb specification:
//!
//! 1. HOST_FORMAT (0) - Set byte order (legacy)
//! 2. DEVICE_CONFIG (5) - Get device capabilities (channels, firmware/hardware version)
//! 3. BT_CONST (4) - Get bit timing constraints
//! 4. BT_CONST_EXT (11) - Get CAN FD constraints (if supported)

use gs_usb::{
    DeviceCapability, DeviceInfo, GsUsb, GsUsbError, GS_CAN_FEATURE_BERR_REPORTING,
    GS_CAN_FEATURE_BT_CONST_EXT, GS_CAN_FEATURE_FD, GS_CAN_FEATURE_GET_STATE,
    GS_CAN_FEATURE_HW_TIMESTAMP, GS_CAN_FEATURE_IDENTIFY, GS_CAN_FEATURE_LISTEN_ONLY,
    GS_CAN_FEATURE_LOOP_BACK, GS_CAN_FEATURE_ONE_SHOT, GS_CAN_FEATURE_PAD_PKTS_TO_MAX_PKT_SIZE,
    GS_CAN_FEATURE_REQ_USB_QUIRK_LPC546XX, GS_CAN_FEATURE_TERMINATION,
    GS_CAN_FEATURE_TRIPLE_SAMPLE, GS_CAN_FEATURE_USER_ID,
};

fn main() {
    env_logger::init();

    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn send_host_format(dev: &mut GsUsb) {
    println!("=== Step 1: HOST_FORMAT ===");
    match dev.send_host_format() {
        Ok(()) => println!("HOST_FORMAT sent successfully (little-endian byte order)"),
        Err(e) => println!("HOST_FORMAT failed (may be optional on this device): {}", e),
    }
    println!();
}

fn get_device_config(dev: &mut GsUsb) -> Result<DeviceInfo, GsUsbError> {
    println!("=== Step 2: DEVICE_CONFIG ===");
    let device_info = dev.device_info()?;

    println!(
        "Number of CAN channels: {} (icount={})",
        device_info.channel_count(),
        device_info.icount
    );
    println!(
        "Firmware version: {:.1} (raw={})",
        device_info.firmware_version(),
        device_info.fw_version
    );
    println!(
        "Hardware version: {:.1} (raw={})",
        device_info.hardware_version(),
        device_info.hw_version
    );
    println!();

    Ok(device_info)
}

fn get_bt_const(dev: &mut GsUsb) -> Result<DeviceCapability, GsUsbError> {
    println!("=== Step 3: BT_CONST (Bit Timing Constraints) ===");
    let capability = dev.device_capability()?;

    println!("Feature bitfield: 0x{:08x}", capability.feature);
    println!(
        "Clock frequency: {:.1} MHz ({} Hz)",
        capability.clock_mhz(),
        capability.fclk_can
    );
    println!(
        "TSEG1 range: {} - {}",
        capability.tseg1_min, capability.tseg1_max
    );
    println!(
        "TSEG2 range: {} - {}",
        capability.tseg2_min, capability.tseg2_max
    );
    println!("SJW max: {}", capability.sjw_max);
    println!(
        "BRP range: {} - {} (increment: {})",
        capability.brp_min, capability.brp_max, capability.brp_inc
    );
    println!();

    // Decode feature flags
    println!("Supported features:");
    let feature_names: [(u32, &str); 14] = [
        (GS_CAN_FEATURE_LISTEN_ONLY, "LISTEN_ONLY"),
        (GS_CAN_FEATURE_LOOP_BACK, "LOOP_BACK"),
        (GS_CAN_FEATURE_TRIPLE_SAMPLE, "TRIPLE_SAMPLE"),
        (GS_CAN_FEATURE_ONE_SHOT, "ONE_SHOT"),
        (GS_CAN_FEATURE_HW_TIMESTAMP, "HW_TIMESTAMP"),
        (GS_CAN_FEATURE_IDENTIFY, "IDENTIFY"),
        (GS_CAN_FEATURE_USER_ID, "USER_ID"),
        (
            GS_CAN_FEATURE_PAD_PKTS_TO_MAX_PKT_SIZE,
            "PAD_PKTS_TO_MAX_PKT_SIZE",
        ),
        (GS_CAN_FEATURE_FD, "FD (CAN FD)"),
        (
            GS_CAN_FEATURE_REQ_USB_QUIRK_LPC546XX,
            "REQ_USB_QUIRK_LPC546XX",
        ),
        (GS_CAN_FEATURE_BT_CONST_EXT, "BT_CONST_EXT"),
        (GS_CAN_FEATURE_TERMINATION, "TERMINATION"),
        (GS_CAN_FEATURE_BERR_REPORTING, "BERR_REPORTING"),
        (GS_CAN_FEATURE_GET_STATE, "GET_STATE"),
    ];

    for (flag, name) in &feature_names {
        if (capability.feature & flag) != 0 {
            println!("  - {}", name);
        }
    }
    println!();

    Ok(capability)
}

fn get_bt_const_ext(
    dev: &mut GsUsb,
    capability: &DeviceCapability,
) -> Result<Option<DeviceCapability>, GsUsbError> {
    println!("=== Step 4: BT_CONST_EXT (CAN FD Timing Constraints) ===");

    if (capability.feature & GS_CAN_FEATURE_BT_CONST_EXT) == 0 {
        println!("BT_CONST_EXT not supported by this device (no CAN FD)");
        println!();
        return Ok(None);
    }

    match dev.device_capability_extended()? {
        Some(cap_ext) => {
            println!("CAN FD Data Phase Timing Constraints:");
            if let (Some(dtseg1_min), Some(dtseg1_max)) = (cap_ext.dtseg1_min, cap_ext.dtseg1_max) {
                println!("  DTSEG1 range: {} - {}", dtseg1_min, dtseg1_max);
            }
            if let (Some(dtseg2_min), Some(dtseg2_max)) = (cap_ext.dtseg2_min, cap_ext.dtseg2_max) {
                println!("  DTSEG2 range: {} - {}", dtseg2_min, dtseg2_max);
            }
            if let Some(dsjw_max) = cap_ext.dsjw_max {
                println!("  DSJW max: {}", dsjw_max);
            }
            if let (Some(dbrp_min), Some(dbrp_max), Some(dbrp_inc)) =
                (cap_ext.dbrp_min, cap_ext.dbrp_max, cap_ext.dbrp_inc)
            {
                println!(
                    "  DBRP range: {} - {} (increment: {})",
                    dbrp_min, dbrp_max, dbrp_inc
                );
            }
            println!();
            Ok(Some(cap_ext))
        }
        None => {
            println!("BT_CONST_EXT request failed");
            println!();
            Ok(None)
        }
    }
}

fn run() -> gs_usb::Result<()> {
    // Find our device
    println!("Scanning for gs_usb devices...");
    let devices = GsUsb::scan()?;
    if devices.is_empty() {
        println!("Can not find gs_usb device");
        return Ok(());
    }

    let mut dev = devices.into_iter().next().unwrap();
    println!("Found device: {}", dev);
    println!();

    // Step 1: Send HOST_FORMAT (set byte order)
    send_host_format(&mut dev);

    // Step 2: Get device configuration
    let device_info = get_device_config(&mut dev)?;

    // Step 3: Get bit timing constraints
    let capability = get_bt_const(&mut dev)?;

    // Step 4: Get extended bit timing constraints (CAN FD) if supported
    let bt_const_ext = get_bt_const_ext(&mut dev, &capability)?;

    println!("{}", "=".repeat(50));
    println!("Device initialization cycle complete!");
    println!("Device has {} CAN channel(s)", device_info.channel_count());
    if bt_const_ext.is_some() {
        println!("CAN FD is supported (BT_CONST_EXT available)");
    } else {
        println!("Classic CAN only (no CAN FD)");
    }

    Ok(())
}
