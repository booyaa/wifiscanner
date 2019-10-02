// Copyright 2016 Mark Sta Ana.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your option.
// This file may not be copied, modified, or distributed except
// according to those terms.

// Inspired by Maurice Svay's node-wifiscanner (https://github.com/mauricesvay/node-wifiscanner)

//! A crate to list WiFi hotspots in your area.
//!
//! As of v0.4.x now supports OSX and Linux. Windows to follow.
//!
//! # Usage
//!
//! This crate is [on crates.io](https://crates.io/crates/wifiscanner) and can be
//! used by adding `wifiscanner` to the dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! wifiscanner = "0.4.*"
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
//! ```
//! use wifiscanner;
//! println!("{:?}", wifiscanner::scan());
//! ```
//!
//! Alternatively if you've cloned the the Git repo, you can run the above example
//! using: `cargo run --example scan`.

#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    SyntaxRegexError,
    CommandNotFound,
    NoMatch,
    FailedToParse,
    NoValue,
}

/// Wifi struct used to return information about wifi hotspots
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Wifi {
    /// mac address
    pub mac: String,
    /// hotspot name
    pub ssid: String,
    pub channel: String,
    pub signal_level: String,
    /// this field is currently empty in the Linux version of the lib
    pub security: String,
}

/// Returns a list of WiFi hotspots in your area - (OSX/MacOS) uses `airport`
#[cfg(target_os = "macos")]
pub fn scan() -> Result<Vec<Wifi>, Error> {
    use std::process::Command;
    let output = try!(Command::new(
        "/System/Library/PrivateFrameworks/Apple80211.\
         framework/Versions/Current/Resources/airport"
    )
    .arg("-s")
    .output()
    .map_err(|_| Error::CommandNotFound));

    let data = String::from_utf8_lossy(&output.stdout);

    parse_airport(&data)
}

/// Returns a list of WiFi hotspots in your area - (Linux) uses `iw`
#[cfg(target_os = "linux")]
pub fn scan() -> Result<Vec<Wifi>, Error> {
    use std::env;
    use std::process::Command;

    const PATH_ENV: &'static str = "PATH";
    let path_system = "/usr/sbin:/sbin";
    let path = env::var_os(PATH_ENV).map_or(path_system.to_string(), |v| {
        format!("{}:{}", v.to_string_lossy().into_owned(), path_system)
    });

    let output = Command::new("iw")
        .env(PATH_ENV, path.clone())
        .arg("dev")
        .output()
        .map_err(|_| Error::CommandNotFound)?;
    let data = String::from_utf8_lossy(&output.stdout);
    let interface = parse_iw_dev(&data)?;

    let output = Command::new("iw")
        .env(PATH_ENV, path)
        .arg("dev")
        .arg(interface)
        .arg("scan")
        .output()
        .map_err(|_| Error::CommandNotFound)?;
    let data = String::from_utf8_lossy(&output.stdout);
    parse_iw_dev_scan(&data)
}

#[allow(dead_code)]
fn parse_airport(network_list: &str) -> Result<Vec<Wifi>, Error> {
    let mut wifis: Vec<Wifi> = Vec::new();
    let mut lines = network_list.lines();
    let headers = match lines.next() {
        Some(v) => v,
        // return an empty list of WiFi if the network_list is empty
        None => return Ok(vec![]),
    };

    let headers_string = String::from(headers);
    // FIXME: Turn these into non panicking Errors (ok_or breaks it)
    let col_mac = headers_string.find("BSSID").expect("failed to find BSSID");
    let col_rrsi = headers_string.find("RSSI").expect("failed to find RSSI");
    let col_channel = headers_string
        .find("CHANNEL")
        .expect("failed to find CHANNEL");
    let col_ht = headers_string.find("HT").expect("failed to find HT");
    let col_security = headers_string
        .find("SECURITY")
        .expect("failed to find SECURITY");

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

#[allow(dead_code)]
fn parse_iw_dev(interfaces: &str) -> Result<String, Error> {
    interfaces
        .split("\tInterface ")
        .take(2)
        .last()
        .ok_or(Error::NoValue)?
        .split("\n")
        .nth(0)
        .ok_or(Error::NoValue)
        .map(|text| text.to_string())
}

#[allow(dead_code)]
fn parse_iw_dev_scan(network_list: &str) -> Result<Vec<Wifi>, Error> {
    // TODO: implement wifi.security
    let mut wifis: Vec<Wifi> = Vec::new();
    let mut wifi = Wifi::default();
    for line in network_list.split("\n") {
        if let Ok(mac) = extract_value(line, "BSS ", Some("(")) {
            wifi.mac = mac;
        } else if let Ok(signal) = extract_value(line, "\tsignal: ", Some(" dBm")) {
            wifi.signal_level = signal;
        } else if let Ok(channel) = extract_value(line, "\tDS Parameter set: channel ", None) {
            wifi.channel = channel;
        } else if let Ok(ssid) = extract_value(line, "\tSSID: ", None) {
            wifi.ssid = ssid;
        }

        if !wifi.mac.is_empty()
            && !wifi.signal_level.is_empty()
            && !wifi.channel.is_empty()
            && !wifi.ssid.is_empty()
        {
            wifis.push(wifi);
            wifi = Wifi::default();
        }
    }

    Ok(wifis)
}

#[allow(dead_code)]
fn extract_value(
    line: &str,
    pattern_start: &str,
    pattern_end: Option<&str>,
) -> Result<String, Error> {
    let start = pattern_start.len();
    if start < line.len() && &line[0..start] == pattern_start {
        let end = match pattern_end {
            Some(end) => line.find(end).ok_or(Error::NoValue)?,
            None => line.len(),
        };
        Ok(line[start..end].to_string())
    } else {
        Err(Error::NoValue)
    }
}

#[test]
fn should_parse_iw_dev() {
    let expected = "wlp2s0";

    // FIXME: should be a better way to create test fixtures
    use std::path::PathBuf;
    let mut path = PathBuf::new();
    path.push("tests");
    path.push("fixtures");
    path.push("iw");
    path.push("iw_dev_01.txt");

    let file_path = path.as_os_str();

    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(&file_path).unwrap();

    let mut filestr = String::new();
    let _ = file.read_to_string(&mut filestr).unwrap();

    let result = parse_iw_dev(&filestr).unwrap();
    assert_eq!(expected, result);
}

#[test]
fn should_parse_iw_dev_scan() {
    let mut expected: Vec<Wifi> = Vec::new();
    expected.push(Wifi {
        mac: "11:22:33:44:55:66".to_string(),
        ssid: "hello".to_string(),
        channel: "10".to_string(),
        signal_level: "-67.00".to_string(),
        security: "".to_string(),
    });

    expected.push(Wifi {
        mac: "66:77:88:99:aa:bb".to_string(),
        ssid: "hello-world-foo-bar".to_string(),
        channel: "8".to_string(),
        signal_level: "-89.00".to_string(),
        security: "".to_string(),
    });

    // FIXME: should be a better way to create test fixtures
    use std::path::PathBuf;
    let mut path = PathBuf::new();
    path.push("tests");
    path.push("fixtures");
    path.push("iw");
    path.push("iw_dev_scan_01.txt");

    let file_path = path.as_os_str();

    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(&file_path).unwrap();

    let mut filestr = String::new();
    let _ = file.read_to_string(&mut filestr).unwrap();

    let result = parse_iw_dev_scan(&filestr).unwrap();
    assert_eq!(expected[0], result[0]);
    assert_eq!(expected[1], result[5]);
}

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

    // FIXME: should be a better way to create test fixtures
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
    let _ = file.read_to_string(&mut filestr).unwrap();

    let result = parse_airport(&filestr).unwrap();
    let last = result.len() - 1;
    assert_eq!(expected[0], result[0]);
    assert_eq!(expected[1], result[last]);
}
