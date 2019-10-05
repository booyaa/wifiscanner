#[macro_use]
extern crate itertools;
extern crate regex;

use regex::Regex;

use crate::{Error, Wifi};

/// Returns a list of WiFi hotspots in your area - (Windows) uses `netsh`
pub fn scan() -> Result<Vec<Wifi>, Error> {
    use std::process::Command;
    let output = Command::new("netsh.exe")
        .args(&["wlan", "show", "networks", "mode=Bssid"])
        .output()
        .map_err(|_| Error::CommandNotFound)?;

    let data = String::from_utf8_lossy(&output.stdout);

    parse_netsh(&data)
}

fn parse_netsh(network_list: &str) -> Result<Vec<Wifi>, Error> {
    let mut wifis = Vec::new();

    // Regex for matching split, SSID and MAC, since these aren't pulled directly
    let split_regex = Regex::new("\nSSID").map_err(|_| Error::SyntaxRegexError)?;
    let ssid_regex = Regex::new("^ [0-9]* : ").map_err(|_| Error::SyntaxRegexError)?;
    let mac_regex = Regex::new("[a-fA-F0-9:]{17}").map_err(|_| Error::SyntaxRegexError)?;

    for block in split_regex.split(network_list) {
        let mut wifi_macs = Vec::new();
        let mut wifi_ssid = String::new();
        let mut wifi_channels = Vec::new();
        let mut wifi_rssi = Vec::new();
        let mut wifi_security = String::new();

        for line in block.lines() {
            if ssid_regex.is_match(line) {
                wifi_ssid = line.split(":").nth(1).unwrap_or("").trim().to_string();
            } else if line.find("Authentication").is_some() {
                wifi_security = line.split(":").nth(1).unwrap_or("").trim().to_string();
            } else if line.find("BSSID").is_some() {
                let captures = mac_regex.captures(line).unwrap();
                wifi_macs.push(captures.get(0).unwrap());
            } else if line.find("Signal").is_some() {
                let percent = line.split(":").nth(1).unwrap_or("").trim().replace("%", "");
                let percent: i32 = percent.parse().unwrap();
                wifi_rssi.push(percent / 2 - 100);
            } else if line.find("Channel").is_some() {
                wifi_channels.push(line.split(":").nth(1).unwrap_or("").trim().to_string());
            }
        }

        for (mac, channel, rssi) in izip!(wifi_macs, wifi_channels, wifi_rssi) {
            wifis.push(Wifi {
                mac: mac.as_str().to_string(),
                ssid: wifi_ssid.to_string(),
                channel: channel.to_string(),
                signal_level: rssi.to_string(),
                security: wifi_security.to_string(),
            });
        }
    }

    Ok(wifis)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_parse_netsh() {
        use std::fs;

        // Note: formula for % to dBm is (% / 100) - 100
        let expected = vec![
            Wifi {
                mac: "ab:cd:ef:01:23:45".to_string(),
                ssid: "Vodafone Hotspot".to_string(),
                channel: "6".to_string(),
                signal_level: "-92".to_string(),
                security: "Open".to_string(),
            },
            Wifi {
                mac: "ab:cd:ef:01:23:45".to_string(),
                ssid: "Vodafone Hotspot".to_string(),
                channel: "6".to_string(),
                signal_level: "-73".to_string(),
                security: "Open".to_string(),
            },
            Wifi {
                mac: "ab:cd:ef:01:23:45".to_string(),
                ssid: "EdaBox".to_string(),
                channel: "11".to_string(),
                signal_level: "-82".to_string(),
                security: "WPA2-Personal".to_string(),
            },
            Wifi {
                mac: "ab:cd:ef:01:23:45".to_string(),
                ssid: "FRITZ!Box 2345 Cable".to_string(),
                channel: "1".to_string(),
                signal_level: "-50".to_string(),
                security: "WPA2-Personal".to_string(),
            },
        ];

        // Load test fixtures
        let fixture = fs::read_to_string("tests/fixtures/netsh/netsh01_windows81.txt").unwrap();

        let result = parse_netsh(&fixture).unwrap();
        assert_eq!(expected[0], result[0]);
        assert_eq!(expected[1], result[1]);
        assert_eq!(expected[2], result[2]);
        assert_eq!(expected[3], result[3]);
    }
}
