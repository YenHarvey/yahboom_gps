//! [![github]](https://github.com/YenHarvey/yahboom_gps)&ensp;[![crates-io]](https://crates.io/crates/yahboom_gps)&ensp;[![docs-rs]](https://docs.rs/yahboom_gps)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//! <br>
//! 
//! # yahboom_gps
//!
//! `yahboom_gps` is a Rust library for initializing and reading GPS data from a Yahboom GPS module via a serial port.
//!
//! ## Example
//!
//! Here is a complete example demonstrating how to use the `yahboom_gps` library to initialize the GPS module, read GPS messages, and parse GPS data.
//!
//! ```rust
//! use yahboom_gps::{gps_init, read_complete_gps_message, parse_gps_data};
//! use anyhow::Result;
//!
//! fn main() -> Result<()> {
//!     // Initialize the GPS module
//!     let mut port = gps_init("COM3", 9600)?;
//!
//!     // Continuously read and parse GPS messages
//!     while let Ok(Some(message)) = read_complete_gps_message(&mut port) {
//!         // Parse the GPS data
//!         let parsed_data = parse_gps_data(&message);
//!
//!         // Print the parsed GPS data
//!         println!("Parsed GPS Data: {}", serde_json::to_string_pretty(&parsed_data)?);
//!         println!("--- End of message ---");
//!     }
//!
//!     Ok(())
//! }
//! ```
//! 
//! Successful output should look like this:
//! ```json
//! Parsed GPS Data: {
//!    "BDGSA": {
//!      "fix_type": "1",
//!      "hdop": "2.5",
//!      "mode": "A",
//!      "pdop": "251.5",
//!      "vdop": "0.5*13"
//!    },
//!    "BDGSV": {
//!      "message_number": "1",
//!      "num_of_messages": "1",
//!      "satellites_in_view": "00*168"
//!    },
//!    "GNGGA": {
//!      "altitude": "",
//!      "fix_quality": "0",
//!      "height_of_geoid": "",
//!      "horizontal_dilution": "1.5",
//!      "latitude": "",
//!      "longitude": "",
//!      "num_of_satellites": "00",
//!      "time": ""
//!    },
//!    "GNGLL": {
//!      "latitude": "",
//!      "longitude": "",
//!      "status": "V",
//!      "time": ""
//!    },
//!    "GNRMC": {
//!      "date": "",
//!      "latitude": "",
//!      "longitude": "",
//!      "speed": "",
//!      "status": "V",
//!      "time": "",
//!      "variation": ""
//!    },
//!    "GNVTG": {
//!      "speed_kmph": "",
//!      "speed_knots": "",
//!      "track_degrees_magnetic": "",
//!      "track_degrees_true": ""
//!    },
//!    "GNZDA": {
//!      "day": "",
//!      "local_zone_hours": "",
//!      "local_zone_minutes": "*6",
//!      "month": "",
//!      "time": "",
//!      "year": ""
//!    },
//!    "GPGSA": {
//!      "fix_type": "1",
//!      "hdop": "2.5",
//!      "mode": "A",
//!      "pdop": "5.0",
//!      "vdop": "2.0*02"
//!    },
//!    "GPGSV": {
//!      "message_number": "1",
//!      "num_of_messages": "1",
//!      "satellites_in_view": "0*79"
//!    },
//!    "GPTXT": {
//!      "text": "01,01,01,ANTENNA OPEN*25"
//!    }
//!  }
//!  --- End of message ---
//! ```
use anyhow::Result;
use serde_json::{json, Value};
use serialport::{self, SerialPort};
use std::io::Read;
use std::str;
use std::time::Duration;

/// Initializes the GPS module
///
/// # Arguments
///
/// * `path` - The path to the serial port
/// * `baud_rate` - The baud rate of the serial port
///
/// # Returns
///
/// A `Result` containing a `Box<dyn SerialPort>` if successful
///
/// # Example
///
/// ```rust
/// use yahboom_gps::gps_init;
/// let port = gps_init("COM3", 9600).unwrap();
/// ```
pub fn gps_init(path: &str, baud_rate: u32) -> Result<Box<dyn SerialPort>> {
    let port = serialport::new(path, baud_rate)
        .timeout(Duration::from_millis(1000))
        .open()?;
    Ok(port)
}

/// Reads a complete GPS message from the serial port
///
/// # Arguments
///
/// * `port` - A mutable reference to the serial port
///
/// # Returns
///
/// A `Result` containing an `Option<Vec<u8>>` if successful
///
/// # Example
///
/// ```rust
/// use yahboom_gps::read_complete_gps_message;
/// let mut port = Box::new(serialport::new("COM3", 9600).open().unwrap());
/// let result = read_complete_gps_message(&mut port).unwrap();
/// if let Some(message) = result {
///     println!("Received GPS message: {:?}", message);
/// }
/// ```
///
/// # Note
///
/// This function reads data from the serial port until a complete GPS message is received.
/// The function returns `None` if no message is received.
pub fn read_complete_gps_message(port: &mut Box<dyn SerialPort>) -> Result<Option<Vec<u8>>> {
    let mut buffer = vec![0; 1024];
    let mut data_accumulator = Vec::new();
    let mut current_message = Vec::new();
    let mut collecting = false;

    loop {
        match port.read(&mut buffer) {
            Ok(bytes) if bytes > 0 => {
                data_accumulator.extend_from_slice(&buffer[..bytes]);

                while let Some(index) = data_accumulator.iter().position(|&x| x == b'\n') {
                    let line = &data_accumulator[..index + 1];
                    let sentence = String::from_utf8_lossy(line);

                    if sentence.starts_with("$GNGGA") {
                        current_message.clear();
                        collecting = true;
                    }

                    if collecting {
                        current_message.extend_from_slice(line);
                    }

                    if sentence.starts_with("$GPTXT") && collecting {
                        data_accumulator.drain(..index + 1);
                        return Ok(Some(current_message.clone())); // Return the complete message
                    }

                    data_accumulator.drain(..index + 1);
                }
            }
            Ok(_) => continue,
            Err(e) => {
                eprintln!("Error reading from serial port: {:?}", e);
                return Err(e.into());
            }
        }
    }
}

/// Parses GPS data from an NMEA sentence
///
/// # Arguments
///
/// * `nmea_data` - A slice of bytes containing the NMEA sentence
///
/// # Returns
///
/// A `Value` containing the parsed GPS data
///
/// # Example
///
/// ```rust
/// use yahboom_gps::parse_gps_data;
/// let nmea_data = b"$GNGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47";
/// let result = parse_gps_data(nmea_data);
/// println!("Parsed GPS Data: {}", result);
/// ```
///
/// # Note
///
/// This function parses the NMEA sentence and returns a JSON object containing the GPS data.
pub fn parse_gps_data(nmea_data: &[u8]) -> Value {
    let data_str = str::from_utf8(nmea_data).unwrap_or_default();
    let mut result = json!({});

    for line in data_str.lines() {
        if let Some(sentence) = line.split_once(',') {
            let (sentence_type, rest) = sentence;
            let sentence_type = sentence_type.trim_start_matches('$');

            let fields: Vec<&str> = rest.split(',').collect();

            match sentence_type {
                "GNGGA" => {
                    result[sentence_type] = json!({
                        "time": fields.get(0).unwrap_or(&""),
                        "latitude": fields.get(1).unwrap_or(&""),
                        "longitude": fields.get(3).unwrap_or(&""),
                        "fix_quality": fields.get(5).unwrap_or(&""),
                        "num_of_satellites": fields.get(6).unwrap_or(&""),
                        "horizontal_dilution": fields.get(7).unwrap_or(&""),
                        "altitude": fields.get(8).unwrap_or(&""),
                        "height_of_geoid": fields.get(10).unwrap_or(&"")
                    });
                }
                "GNGLL" => {
                    result[sentence_type] = json!({
                        "latitude": fields.get(0).unwrap_or(&""),
                        "longitude": fields.get(1).unwrap_or(&""),
                        "time": fields.get(4).unwrap_or(&""),
                        "status": fields.get(5).unwrap_or(&"")
                    });
                }
                "GPGSA" | "BDGSA" => {
                    result[sentence_type] = json!({
                        "mode": fields.get(0).unwrap_or(&""),
                        "fix_type": fields.get(1).unwrap_or(&""),
                        "pdop": fields.get(14).unwrap_or(&""),
                        "hdop": fields.get(15).unwrap_or(&""),
                        "vdop": fields.get(16).unwrap_or(&"")
                    });
                }
                "GPGSV" | "BDGSV" => {
                    result[sentence_type] = json!({
                        "num_of_messages": fields.get(0).unwrap_or(&""),
                        "message_number": fields.get(1).unwrap_or(&""),
                        "satellites_in_view": fields.get(2).unwrap_or(&"")
                    });
                }
                "GNRMC" => {
                    result[sentence_type] = json!({
                        "time": fields.get(0).unwrap_or(&""),
                        "status": fields.get(1).unwrap_or(&""),
                        "latitude": fields.get(2).unwrap_or(&""),
                        "longitude": fields.get(4).unwrap_or(&""),
                        "speed": fields.get(6).unwrap_or(&""),
                        "date": fields.get(8).unwrap_or(&""),
                        "variation": fields.get(9).unwrap_or(&"")
                    });
                }
                "GNVTG" => {
                    result[sentence_type] = json!({
                        "track_degrees_true": fields.get(0).unwrap_or(&""),
                        "track_degrees_magnetic": fields.get(2).unwrap_or(&""),
                        "speed_knots": fields.get(4).unwrap_or(&""),
                        "speed_kmph": fields.get(6).unwrap_or(&"")
                    });
                }
                "GNZDA" => {
                    result[sentence_type] = json!({
                        "time": fields.get(0).unwrap_or(&""),
                        "day": fields.get(1).unwrap_or(&""),
                        "month": fields.get(2).unwrap_or(&""),
                        "year": fields.get(3).unwrap_or(&""),
                        "local_zone_hours": fields.get(4).unwrap_or(&""),
                        "local_zone_minutes": fields.get(5).unwrap_or(&"")
                    });
                }
                "GPTXT" => {
                    result[sentence_type] = json!({
                        "text": rest
                    });
                }
                _ => {}
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fn_read_complete_gps_message() {
        let mut port = Box::new(serialport::new("COM3", 9600).open().unwrap());
        let result = read_complete_gps_message(&mut port).unwrap();
        assert!(result.is_some());
    }
}


