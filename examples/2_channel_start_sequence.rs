//! Channel Start Sequence Example (CAN FD)
//!
//! This script demonstrates the CAN FD channel start sequence for a gs_usb device,
//! following the protocol flow described in the gs_usb specification:
//!
//! 1. BITTIMING (1) - Set nominal bit timing (1 Mbps arbitration phase)
//! 2. DATA_BITTIMING (10) - Set CAN FD data phase timing (5 Mbps)
//! 3. MODE (2) - Start channel with FD + HW_TIMESTAMP flags
//!
//! This example configures:
//! - Arbitration (nominal) bitrate: 1 Mbps
//! - Data bitrate: 5 Mbps

use std::time::{Duration, Instant};

use gs_usb::{
    GsUsb, GsUsbError, GsUsbFrame, GS_CAN_MODE_FD, GS_CAN_MODE_LOOP_BACK, GS_CAN_MODE_NORMAL,
    GS_CAN_MODE_ONE_SHOT,
};

fn main() {
    env_logger::init();

    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
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

    // Check device capabilities
    let capability = dev.device_capability()?;
    println!("Device clock: {:.1} MHz", capability.clock_mhz());
    println!("Feature flags: 0x{:08x}", capability.feature);

    if !dev.supports_fd()? {
        println!("ERROR: Device does not support CAN FD!");
        println!("This example requires a CAN FD capable device.");
        return Ok(());
    }

    println!("Device supports CAN FD");
    println!();

    // Get extended capabilities for CAN FD timing info
    if let Some(cap_ext) = dev.device_capability_extended()? {
        println!("=== CAN FD Timing Constraints ===");
        if let (Some(dtseg1_min), Some(dtseg1_max)) = (cap_ext.dtseg1_min, cap_ext.dtseg1_max) {
            println!("Data phase TSEG1: {} - {}", dtseg1_min, dtseg1_max);
        }
        if let (Some(dtseg2_min), Some(dtseg2_max)) = (cap_ext.dtseg2_min, cap_ext.dtseg2_max) {
            println!("Data phase TSEG2: {} - {}", dtseg2_min, dtseg2_max);
        }
        if let Some(dsjw_max) = cap_ext.dsjw_max {
            println!("Data phase SJW max: {}", dsjw_max);
        }
        if let (Some(dbrp_min), Some(dbrp_max)) = (cap_ext.dbrp_min, cap_ext.dbrp_max) {
            println!("Data phase BRP: {} - {}", dbrp_min, dbrp_max);
        }
        println!();
    }

    // Step 1: Set nominal (arbitration) bit timing - 1 Mbps
    println!("=== Step 1: BITTIMING (Nominal Phase) ===");
    println!("Setting arbitration bitrate to 1 Mbps...");
    match dev.set_bitrate(1_000_000) {
        Ok(()) => println!("Nominal bitrate set successfully"),
        Err(e) => {
            println!("ERROR: Failed to set nominal bitrate: {}", e);
            println!("Your device clock may not be supported.");
            return Ok(());
        }
    }
    println!();

    // Step 2: Set data phase bit timing - 5 Mbps
    println!("=== Step 2: DATA_BITTIMING (Data Phase) ===");
    println!("Setting data bitrate to 5 Mbps...");
    match dev.set_data_bitrate(5_000_000) {
        Ok(()) => println!("Data bitrate set successfully"),
        Err(e) => {
            println!("ERROR: Failed to set data bitrate: {}", e);
            println!("Your device clock may not support 5 Mbps.");
            println!("Try 2 Mbps or 4 Mbps instead.");
            return Ok(());
        }
    }
    println!();

    // Step 3: Start channel with FD mode enabled
    println!("=== Step 3: MODE (Start Channel) ===");
    let flags = GS_CAN_MODE_NORMAL | GS_CAN_MODE_FD | GS_CAN_MODE_ONE_SHOT | GS_CAN_MODE_LOOP_BACK;
    println!("Starting channel with flags: 0x{:04x}", flags);
    println!("  - GS_CAN_MODE_NORMAL");
    println!("  - GS_CAN_MODE_ONE_SHOT");
    println!("  - GS_CAN_MODE_FD");
    println!("  - GS_CAN_MODE_LOOP_BACK");
    dev.start(flags)?;
    println!("Channel started successfully!");
    println!();

    println!("{}", "=".repeat(50));
    println!("CAN FD Channel Start Sequence Complete!");
    println!("{}", "=".repeat(50));
    println!();
    println!("Configuration:");
    println!("  Arbitration bitrate: 1 Mbps");
    println!("  Data bitrate: 5 Mbps");
    println!("  FD mode: Enabled");
    println!("  Loopback mode: Enabled");
    println!();

    // Demonstrate sending a CAN FD frame
    println!("=== Sending Test CAN FD Frame ===");
    // Create a CAN FD frame with 64 bytes of data (requires FD mode)
    let test_data: Vec<u8> = (0..64).collect();
    let fd_frame = GsUsbFrame::with_fd_data(0x123, &test_data, true);
    println!("TX  {}", fd_frame);

    match dev.send(&fd_frame) {
        Ok(()) => println!("Frame sent successfully!"),
        Err(e) => println!("Failed to send frame: {}", e),
    }
    println!();

    // Listen for incoming frames for a few seconds
    println!("=== Listening for CAN FD Frames (5 seconds) ===");
    println!("(Connect to a CAN FD bus to see received frames)");
    println!();

    let end_time = Instant::now() + Duration::from_secs(5);
    let mut frame_count = 0;
    let mut echo_count = 0;
    let mut rx_count = 0;

    while Instant::now() < end_time {
        match dev.read(Duration::from_millis(100)) {
            Ok(frame) => {
                frame_count += 1;
                if frame.is_echo_frame() {
                    // Echo frame = TX confirmation from device (our transmitted frame)
                    echo_count += 1;
                    println!("ECHO  {}", frame);
                } else {
                    // RX frame = frame received from CAN bus
                    rx_count += 1;
                    println!("RX    {}", frame);
                }
            }
            Err(GsUsbError::ReadTimeout) => {
                // No frame available, continue waiting
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }

    println!();
    println!(
        "Total frames: {} (echo: {}, rx: {})",
        frame_count, echo_count, rx_count
    );
    println!();

    // Stop the device
    println!("Stopping device...");
    dev.stop()?;
    println!("Device stopped.");

    Ok(())
}
