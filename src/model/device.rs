use serde::{Deserialize, Deserializer};

use crate::wrapper::platformio;

#[derive(Deserialize, Debug)]
pub struct PioDevice {
    pub port: String,
    #[serde(deserialize_with = "parse_input", default)]
    pub description: Option<String>,

    #[serde(deserialize_with = "parse_input", default)]
    pub hwid: Option<String>,
}

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