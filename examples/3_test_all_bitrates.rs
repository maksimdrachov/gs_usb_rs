//! Test All Bitrates Example
//!
//! This script tests all supported Classic CAN and CAN FD bitrates by:
//! 1. Configuring the CAN interface with loopback mode
//! 2. Sending a test frame
//! 3. Verifying that 2 frames are received (1 echo + 1 loopback RX) with correct payload
//!
//! Classic CAN bitrates tested (40MHz clock):
//! - 10k, 20k, 50k, 100k, 125k, 250k, 500k, 1M
//!
//! CAN FD bitrate combinations tested (40MHz clock):
//! - Arbitration: 125k, 250k, 500k, 1M
//! - Data: 2M, 5M, 8M, 10M

use std::time::{Duration, Instant};

use gs_usb::{
    GsUsb, GsUsbError, GsUsbFrame, GS_CAN_MODE_FD, GS_CAN_MODE_HW_TIMESTAMP, GS_CAN_MODE_LOOP_BACK,
    GS_CAN_MODE_NORMAL,
};

// Test configuration
const TEST_CAN_ID: u32 = 0x123;
const TEST_DATA_CLASSIC: [u8; 8] = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
const READ_TIMEOUT_MS: u64 = 1000;

// Classic CAN bitrates to test
const CLASSIC_CAN_BITRATES: [u32; 8] = [
    10_000, 20_000, 50_000, 100_000, 125_000, 250_000, 500_000, 1_000_000,
];

// CAN FD arbitration bitrates
const FD_ARBITRATION_BITRATES: [u32; 4] = [125_000, 250_000, 500_000, 1_000_000];

// CAN FD data bitrates
const FD_DATA_BITRATES: [u32; 4] = [2_000_000, 5_000_000, 8_000_000, 10_000_000];

struct TestResult {
    name: String,
    passed: bool,
    error_message: String,
    echo_received: bool,
    rx_received: bool,
    echo_data_correct: bool,
    rx_data_correct: bool,
}

impl TestResult {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            error_message: String::new(),
            echo_received: false,
            rx_received: false,
            echo_data_correct: false,
            rx_data_correct: false,
        }
    }
}

fn verify_frame_data(frame: &GsUsbFrame, expected_data: &[u8]) -> bool {
    let expected_len = expected_data.len();
    let actual_data = frame.data();
    actual_data.len() >= expected_len && &actual_data[..expected_len] == expected_data
}

fn run_single_test(
    dev: &mut GsUsb,
    test_name: &str,
    is_fd: bool,
    expected_data: &[u8],
) -> TestResult {
    let mut result = TestResult::new(test_name);

    // Create test frame
    let tx_frame = if is_fd {
        GsUsbFrame::with_fd_data(TEST_CAN_ID, expected_data, true)
    } else {
        GsUsbFrame::with_data(TEST_CAN_ID, expected_data)
    };

    // Send frame
    if let Err(e) = dev.send(&tx_frame) {
        result.error_message = format!("Failed to send frame: {}", e);
        return result;
    }

    // Read frames (expecting 2: echo + loopback RX)
    let mut frames_received = Vec::new();
    let start_time = Instant::now();
    while frames_received.len() < 2 && start_time.elapsed() < Duration::from_secs(2) {
        match dev.read(Duration::from_millis(READ_TIMEOUT_MS)) {
            Ok(frame) => frames_received.push(frame),
            Err(GsUsbError::ReadTimeout) => continue,
            Err(_) => break,
        }
    }

    // Analyze received frames
    for frame in &frames_received {
        if frame.is_echo_frame() {
            result.echo_received = true;
            result.echo_data_correct = verify_frame_data(frame, expected_data);
        } else {
            result.rx_received = true;
            result.rx_data_correct = verify_frame_data(frame, expected_data);
        }
    }

    // Determine pass/fail
    if !result.echo_received {
        result.error_message = "Echo frame not received".to_string();
    } else if !result.rx_received {
        result.error_message = "Loopback RX frame not received".to_string();
    } else if !result.echo_data_correct {
        result.error_message = "Echo frame data mismatch".to_string();
    } else if !result.rx_data_correct {
        result.error_message = "Loopback RX frame data mismatch".to_string();
    } else {
        result.passed = true;
    }

    result
}

fn test_classic_can_bitrate(dev: &mut GsUsb, bitrate: u32) -> TestResult {
    let test_name = format!("Classic CAN {}k", bitrate / 1000);

    // Configure bitrate
    if let Err(e) = dev.set_bitrate(bitrate) {
        let mut result = TestResult::new(&test_name);
        result.error_message = format!("Failed to set bitrate {}: {}", bitrate, e);
        return result;
    }

    // Start device with loopback
    let flags = GS_CAN_MODE_NORMAL | GS_CAN_MODE_HW_TIMESTAMP | GS_CAN_MODE_LOOP_BACK;
    if let Err(e) = dev.start(flags) {
        let mut result = TestResult::new(&test_name);
        result.error_message = format!("Failed to start device: {}", e);
        return result;
    }

    // Run test
    let result = run_single_test(dev, &test_name, false, &TEST_DATA_CLASSIC);

    // Stop device
    let _ = dev.stop();

    result
}

fn test_canfd_bitrate(dev: &mut GsUsb, arb_bitrate: u32, data_bitrate: u32) -> TestResult {
    let test_name = format!(
        "CAN FD {}k / {}M",
        arb_bitrate / 1000,
        data_bitrate / 1_000_000
    );

    // Configure arbitration bitrate
    if let Err(e) = dev.set_bitrate(arb_bitrate) {
        let mut result = TestResult::new(&test_name);
        result.error_message = format!("Failed to set arbitration bitrate {}: {}", arb_bitrate, e);
        return result;
    }

    // Configure data bitrate
    if let Err(e) = dev.set_data_bitrate(data_bitrate) {
        let mut result = TestResult::new(&test_name);
        result.error_message = format!("Failed to set data bitrate {}: {}", data_bitrate, e);
        return result;
    }

    // Start device with loopback and FD mode
    let flags =
        GS_CAN_MODE_NORMAL | GS_CAN_MODE_HW_TIMESTAMP | GS_CAN_MODE_LOOP_BACK | GS_CAN_MODE_FD;
    if let Err(e) = dev.start(flags) {
        let mut result = TestResult::new(&test_name);
        result.error_message = format!("Failed to start device: {}", e);
        return result;
    }

    // Test data for FD (64 bytes)
    let test_data_fd: Vec<u8> = (0..64).collect();

    // Run test
    let result = run_single_test(dev, &test_name, true, &test_data_fd);

    // Stop device
    let _ = dev.stop();

    result
}

fn print_result(result: &TestResult, verbose: bool) {
    let status = if result.passed {
        "✓ PASS"
    } else {
        "✗ FAIL"
    };
    println!("  {}  {}", status, result.name);
    if !result.passed && !result.error_message.is_empty() {
        println!("         Error: {}", result.error_message);
    }
    if verbose && !result.passed {
        println!(
            "         Echo received: {}, data OK: {}",
            result.echo_received, result.echo_data_correct
        );
        println!(
            "         RX received: {}, data OK: {}",
            result.rx_received, result.rx_data_correct
        );
    }
}

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
    println!("GS-USB Bitrate Test Suite");
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

    // Check device capabilities
    let capability = dev.device_capability()?;
    println!("Device clock: {:.1} MHz", capability.clock_mhz());
    println!("Feature flags: 0x{:08x}", capability.feature);
    println!(
        "CAN FD support: {}",
        if dev.supports_fd()? { "Yes" } else { "No" }
    );
    println!();

    let mut results = Vec::new();

    // Test Classic CAN bitrates
    println!("{}", "-".repeat(60));
    println!("Testing Classic CAN Bitrates");
    println!("{}", "-".repeat(60));

    for &bitrate in &CLASSIC_CAN_BITRATES {
        let result = test_classic_can_bitrate(&mut dev, bitrate);
        print_result(&result, false);
        results.push(result);
    }

    // Test CAN FD bitrates (if supported)
    if dev.supports_fd()? {
        println!();
        println!("{}", "-".repeat(60));
        println!("Testing CAN FD Bitrates");
        println!("{}", "-".repeat(60));

        for &arb_bitrate in &FD_ARBITRATION_BITRATES {
            for &data_bitrate in &FD_DATA_BITRATES {
                let result = test_canfd_bitrate(&mut dev, arb_bitrate, data_bitrate);
                print_result(&result, false);
                results.push(result);
            }
        }
    } else {
        println!();
        println!("Skipping CAN FD tests (device does not support CAN FD)");
    }

    // Summary
    println!();
    println!("{}", "=".repeat(60));
    println!("Test Summary");
    println!("{}", "=".repeat(60));

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();
    let total = results.len();

    println!("Total tests: {}", total);
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!();

    if failed > 0 {
        println!("Failed tests:");
        for r in &results {
            if !r.passed {
                println!("  - {}: {}", r.name, r.error_message);
            }
        }
        println!();
    }

    if failed == 0 {
        println!("All tests PASSED! ✓");
        Ok(0)
    } else {
        println!("Some tests FAILED! ✗ ({}/{})", failed, total);
        Ok(1)
    }
}
