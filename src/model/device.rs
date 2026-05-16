//! Represents connected hardware devices and serial ports.
//!
//! This module handles querying the host machine for connected microcontrollers
//! and parsing the JSON output provided by PlatformIO's device list tool.

use serde::{Deserialize, Deserializer};
use serde::ser::{SerializeStruct, Serialize, Serializer};

use crate::wrapper::platformio;

/// Represents a serial device or port detected on the host machine.
#[derive(Deserialize, Debug)]
pub struct PioDevice {
    /// The physical port the device is connected to (e.g., "/dev/ttyUSB0" or "COM3").
    pub port: String,

    /// A description of the device.
    #[serde(deserialize_with = "parse_input", default)]
    pub description: Option<String>,

    /// The Hardware ID (e.g., USB VID/PID).
    #[serde(deserialize_with = "parse_input", default)]
    pub hwid: Option<String>,
}

impl Serialize for PioDevice {
    /// Custom serialization logic for `PioDevice`.
    ///
    /// This ensures that when the device list is exported (e.g., for the CLI JSON output),
    /// missing `description` or `hwid` fields are converted back to the `"n/a"` string 
    /// format to maintain consistency with PlatformIO's native console output.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hwid = match &self.hwid {
            Some(id) => id.as_str(),
            None => "n/a",
        };
        let description = match &self.description {
            Some(d) => d.as_str(),
            None => "n/a",
        };

        let mut state = serializer.serialize_struct("PioDevice", 3)?;
        state.serialize_field("port", &self.port)?;
        state.serialize_field("hwid", &hwid)?;
        state.serialize_field("description", description)?;
        state.end()
    }
}

/// Retrieves a list of currently connected microcontrollers.
///
/// Filters the complete port list to return only devices that have a valid Hardware ID,
/// which generally filters out disconnected or virtual ports.
///
/// # Errors
/// Returns an error if querying PlatformIO fails or if the JSON cannot be parsed.
pub fn get_connected_device_list() -> Result<Vec<PioDevice>, String> {
    let ports = get_port_list()?;
    let mut connected: Vec<PioDevice> = Vec::new();

    for port in ports {
        match port.hwid {
            Some(_) => {
                connected.push(port);
            },
            None => (),
        };
    }

    Ok(connected)
}

/// Retrieves the raw list of all detected serial ports.
///
/// Calls the underlying `platformio` wrapper to execute `pio device list`,
/// captures its JSON output, and deserializes it into a vector of `PioDevice`s.
///
/// # Errors
/// Returns an error if querying PlatformIO fails or if the JSON cannot be parsed.
pub fn get_port_list() -> Result<Vec<PioDevice>, String> {
    let output = platformio::get_devices()?;
    let json = String::from_utf8_lossy(&output);
    let devices: Vec<PioDevice> = match serde_json::from_str(&json) {
        Ok(res) => res,
        Err(_) => {
            return Err(String::from("Failed to parse device list JSON."));
        }
    };
    Ok(devices)
}

/// A custom serde deserialization helper for PlatformIO fields.
///
/// PlatformIO often outputs `"n/a"` for missing device descriptions or hardware IDs.
/// This function translates the literal `"n/a"` string into Rust's native `None` type 
/// during the JSON parsing phase.
fn parse_input<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(match opt {
        Some(s) if s == "n/a" => None,
        other => other,
    })
}


//
// Unit Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pio_device_deserialization() {
        let json_data = r#"
        [
            {
                "port": "/dev/ttyUSB0",
                "description": "USB Serial",
                "hwid": "USB VID:PID=1A86:7523"
            },
            {
                "port": "/dev/ttyS0",
                "description": "n/a",
                "hwid": "n/a"
            }
        ]
        "#;
        
        let devices: Vec<PioDevice> = serde_json::from_str(json_data).unwrap();
        assert_eq!(devices.len(), 2);
        
        // The first device should retain its valid String data
        assert_eq!(devices[0].port, "/dev/ttyUSB0");
        assert_eq!(devices[0].description, Some("USB Serial".to_string()));
        assert_eq!(devices[0].hwid, Some("USB VID:PID=1A86:7523".to_string()));

        // The second device should have "n/a" correctly parsed into None
        assert_eq!(devices[1].port, "/dev/ttyS0");
        assert_eq!(devices[1].description, None);
        assert_eq!(devices[1].hwid, None);
    }
}