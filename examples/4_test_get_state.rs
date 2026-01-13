//! Test GET_STATE Example
//!
//! This script demonstrates how to retrieve the CAN bus state and error counters
//! using the GS_USB_BREQ_GET_STATE request.
//!
//! The state information includes:
//! - CAN state (ERROR_ACTIVE, ERROR_WARNING, ERROR_PASSIVE, BUS_OFF, STOPPED, SLEEPING)
//! - RX error counter (REC)
//! - TX error counter (TEC)
//!
//! This is useful for monitoring CAN bus health and diagnosing communication issues.

use std::thread;
use std::time::{Duration, Instant};

use gs_usb::{
    GsUsb, GS_CAN_MODE_HW_TIMESTAMP, GS_CAN_MODE_LOOP_BACK, GS_CAN_MODE_NORMAL,
    GS_CAN_STATE_BUS_OFF, GS_CAN_STATE_ERROR_ACTIVE, GS_CAN_STATE_ERROR_PASSIVE,
    GS_CAN_STATE_ERROR_WARNING,
};

fn main() {
    env_logger::init();

    match run() {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run() -> gs_usb::Result<i32> {
    println!("{}", "=".repeat(60));
    println!("GS-USB GET_STATE Test");
    println!("{}", "=".repeat(60));
    println!();

    // Find device
    println!("Scanning for gs_usb devices...");
    let devices = GsUsb::scan()?;
    if devices.is_empty() {
        println!("ERROR: No gs_usb device found");
        return Ok(1);
    }

    let mut dev = devices.into_iter().next().unwrap();
    println!("Found device: {}", dev);
    println!();

    // Check device capabilities
    let capability = dev.device_capability()?;
    println!("Device clock: {:.1} MHz", capability.clock_mhz());
    println!("Feature flags: 0x{:08x}", capability.feature);

    if !dev.supports_get_state()? {
        println!();
        println!("ERROR: Device does not support GET_STATE feature");
        println!("This feature requires GS_CAN_FEATURE_GET_STATE (bit 13) to be set");
        return Ok(1);
    }

    println!("GET_STATE support: Yes");
    println!();

    // Configure and start device
    println!("{}", "-".repeat(60));
    println!("Configuring CAN interface...");
    println!("{}", "-".repeat(60));

    dev.set_bitrate(500_000)?;
    println!("Bitrate: 500 kbps");

    // Start with loopback mode (so we don't need a real bus)
    let flags = GS_CAN_MODE_NORMAL | GS_CAN_MODE_HW_TIMESTAMP | GS_CAN_MODE_LOOP_BACK;
    dev.start(flags)?;
    println!("Mode: Loopback + HW Timestamp");
    println!("Device started successfully");
    println!();

    // Get and display state
    println!("{}", "-".repeat(60));
    println!("CAN Bus State");
    println!("{}", "-".repeat(60));

    let state = dev.get_state(0)?;

    println!("State: {}", state.state_name());
    println!("RX Error Counter (REC): {}", state.rxerr);
    println!("TX Error Counter (TEC): {}", state.txerr);
    println!();

    // Explain the state
    println!("State explanation:");
    if state.state == GS_CAN_STATE_ERROR_ACTIVE {
        println!("  ERROR_ACTIVE: Normal operation, TEC and REC are below 96");
    } else if state.state == GS_CAN_STATE_ERROR_WARNING {
        println!("  ERROR_WARNING: TEC or REC exceeded 96");
    } else if state.state == GS_CAN_STATE_ERROR_PASSIVE {
        println!("  ERROR_PASSIVE: TEC or REC exceeded 127");
    } else if state.state == GS_CAN_STATE_BUS_OFF {
        println!("  BUS_OFF: TEC exceeded 255, node is off the bus");
    } else {
        println!(
            "  {}: Device is not actively communicating",
            state.state_name()
        );
    }
    println!();

    // Monitor state for a few seconds
    println!("{}", "-".repeat(60));
    println!("Monitoring state for 3 seconds...");
    println!("{}", "-".repeat(60));

    let start_time = Instant::now();
    let mut last_state: Option<(u32, u32, u32)> = None;

    while start_time.elapsed() < Duration::from_secs(3) {
        let state = dev.get_state(0)?;

        // Only print if state changed
        let state_tuple = (state.state, state.rxerr, state.txerr);
        if last_state.map_or(true, |last| last != state_tuple) {
            let elapsed = start_time.elapsed().as_secs_f64();
            println!(
                "[{:5.2}s] State: {:15} REC: {:3}  TEC: {:3}",
                elapsed,
                state.state_name(),
                state.rxerr,
                state.txerr
            );
            last_state = Some(state_tuple);
        }

        thread::sleep(Duration::from_millis(100));
    }

    println!();
    println!("Monitoring complete");
    println!();

    // Stop device
    dev.stop()?;
    println!("Device stopped");

    Ok(0)
}
