//! GS-USB Example
//!
//! This example demonstrates basic usage of the GS-USB library:
//! - Scanning for devices
//! - Configuring bitrate
//! - Starting the device
//! - Sending and receiving CAN frames

use std::time::{Duration, Instant};

use gs_usb::{
    GsUsb, GsUsbError, GsUsbFrame, CAN_EFF_FLAG, CAN_ERR_FLAG, CAN_RTR_FLAG,
    GS_CAN_MODE_HW_TIMESTAMP, GS_CAN_MODE_NORMAL,
};

fn main() {
    // Initialize logging
    env_logger::init();

    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> gs_usb::Result<()> {
    // Find our device
    let devices = GsUsb::scan()?;
    if devices.is_empty() {
        println!("Can not find gs_usb device");
        return Ok(());
    }

    let mut dev = devices.into_iter().next().unwrap();
    println!("Found device: {}", dev);

    // Configuration
    dev.set_bitrate(250000)?;
    println!("Bitrate set to 250 kbps");

    // Start device
    dev.start(GS_CAN_MODE_NORMAL | GS_CAN_MODE_HW_TIMESTAMP)?;
    println!("Device started");

    // Prepare frames
    let data: [u8; 8] = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];

    let frames = [
        // Standard frame format with data
        GsUsbFrame::with_data(0x7FF, &data),
        // Standard frame format without data
        GsUsbFrame::with_data(0x7FF, &[]),
        // Error frame
        GsUsbFrame::with_data(0x7FF | CAN_ERR_FLAG, &data),
        // Extended frame format with data
        GsUsbFrame::with_data(0x12345678 | CAN_EFF_FLAG, &data),
        // Extended frame format without data
        GsUsbFrame::with_data(0x12345678 | CAN_EFF_FLAG, &[]),
        // Remote transmission request (standard)
        GsUsbFrame::with_data(0x7FF | CAN_RTR_FLAG, &[]),
        // Remote transmission request (extended)
        GsUsbFrame::with_data(0x12345678 | CAN_RTR_FLAG | CAN_EFF_FLAG, &[]),
        // RTR with data (data is ignored for RTR)
        GsUsbFrame::with_data(0x7FF | CAN_RTR_FLAG, &data),
    ];

    println!("\nStarting CAN communication (press Ctrl+C to stop)...\n");

    // Read all the time and send message each second
    let mut next_send_time = Instant::now();
    let mut frame_index = 0;

    loop {
        // Check for keyboard interrupt (Ctrl+C)
        // This is handled by the signal handler, but we check for errors

        // Try to read a frame with 1ms timeout
        match dev.read(Duration::from_millis(1)) {
            Ok(frame) => {
                println!("RX  {}", frame);
            }
            Err(GsUsbError::ReadTimeout) => {
                // No frame available, continue
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }

        // Send a frame every second
        if Instant::now() >= next_send_time {
            next_send_time = Instant::now() + Duration::from_secs(1);

            let frame = &frames[frame_index];
            frame_index = (frame_index + 1) % frames.len();

            match dev.send(frame) {
                Ok(()) => {
                    println!("TX  {}", frame);
                }
                Err(e) => {
                    eprintln!("Send error: {}", e);
                }
            }
        }
    }

    Ok(())
}
