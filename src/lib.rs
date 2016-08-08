// Copyright 2016 Mark Sta Ana.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your option.
// This file may not be copied, modified, or distributed except
// according to those terms.

// Inspired by Maurice Svay's node-wifiscanner (https://github.com/mauricesvay/node-wifiscanner)


//! A crate to list WiFi hotspots in your area.
//!
//! Only support OSX computers, Linux and Windows to follow.
//!
//! # Usage
//!
//! This crate is [on crates.io](https://crates.io/crates/wifiscanner) and can be
//! used by adding `wifiscanner` to the dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! wifiscanner = "0.2"
//! ```
//!
//! and this to your crate root:
//!
//! ```rust
//! extern crate wifiscanner;
//! ```
//!
//! # Example
//!
//! ```rust
//! use wifiscanner;
//! println!("{:?}", wifiscanner::scan());
//! ```

/// Wifi struct used to return information about wifi hotspots
#[derive(Debug,PartialEq,Eq)]
pub struct Wifi {
    pub mac: String,
    pub ssid: String,
    pub channel: String,
    pub signal_level: String,
    pub security: String,
}

/// Returns WiFi hotspots in your area
#[cfg(target_os="macos")]
pub fn scan() -> Result<Vec<Wifi>, String> {
    use std::process::Command;
    let output = match Command::new("/System/Library/PrivateFrameworks/Apple80211.\
    framework/Versions/Current/Resources/airport")
                           .arg("-s")
                           .output() {
        Ok(output) => output,
        Err(_) => return Err("Failed to find airport utility (are you using OSX?)".to_string()),
    };

    let data = String::from_utf8_lossy(&output.stdout);

    parse_airport(&data)
}

fn parse_airport(network_list: &str) -> Result<Vec<Wifi>, String> {
    println!("airport_parse");
    let mut wifis: Vec<Wifi> = Vec::new();
    let mut lines = network_list.lines();
    let headers = lines.next().unwrap();

    let headers_string = String::from(headers);
    // FIXME: Turn these into non panicking Errors
    let col_mac = headers_string.find("BSSID").expect("failed to find BSSID");
    let col_rrsi = headers_string.find("RSSI").expect("failed to find RSSI");
    let col_channel = headers_string.find("CHANNEL").expect("failed to find CHANNEL");
    let col_ht = headers_string.find("HT").expect("failed to find HT");
    let col_security = headers_string.find("SECURITY").expect("failed to find SECURITY");

    for line in lines {
        let ssid = &line[..col_mac].trim();
        let mac = &line[col_mac..col_rrsi].trim();
        let signal_level = &line[col_rrsi..col_channel].trim();
        let channel = &line[col_channel..col_ht].trim();
        let security = &line[col_security..].trim();

        wifis.push(Wifi {
            mac: mac.to_string(),
            ssid: ssid.to_string(),
            channel: channel.to_string(),
            signal_level: signal_level.to_string(),
            security: security.to_string(),
        });
    }

    Ok(wifis)
}

fn parse_iwlist(network_list: &str) -> Result<Vec<Wifi>, String> {
    println!("airport_parse");
    let wifis: Vec<Wifi> = Vec::new();

    Ok(wifis)
}

#[cfg(test)]
#[test]
fn should_parse_iwlist() {
    let expected: Vec<Wifi> = Vec::new();

    use std::path::PathBuf;
    let mut path = PathBuf::new();
    path.push("tests");
    path.push("fixtures");
    path.push("airport");
    path.push("airport01.txt");

    let file_path = path.as_os_str();

    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(&file_path).unwrap();

    let mut filestr = String::new();
    let result = file.read_to_string(&mut filestr).unwrap();
    println!("Read {} bytes", result);

    let result = parse_iwlist(&filestr).unwrap();
    assert_eq!(expected, result);
}

#[cfg(test)]
#[test]
fn should_parse_airport() {
    let mut expected: Vec<Wifi> = Vec::new();
    expected.push(Wifi {
        mac: "00:35:1a:90:56:03".to_string(),
        ssid: "OurTest".to_string(),
        channel: "112".to_string(),
        signal_level: "-70".to_string(),
        security: "WPA2(PSK/AES/AES)".to_string(),
    });

    expected.push(Wifi {
        mac: "00:35:1a:90:56:00".to_string(),
        ssid: "TEST-Wifi".to_string(),
        channel: "1".to_string(),
        signal_level: "-67".to_string(),
        security: "WPA2(PSK/AES/AES)".to_string(),
    });

    use std::path::PathBuf;
    let mut path = PathBuf::new();
    path.push("tests");
    path.push("fixtures");
    path.push("airport");
    path.push("airport01.txt");

    let file_path = path.as_os_str();

    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(&file_path).unwrap();

    let mut filestr = String::new();
    let result = file.read_to_string(&mut filestr).unwrap();
    println!("Read {} bytes", result);

    let result = parse_airport(&filestr).unwrap();
    let last = result.len() - 1;
    assert_eq!(expected[0], result[0]);
    assert_eq!(expected[1], result[last]);
}
