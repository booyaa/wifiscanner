// Copyright 2016 Mark Sta Ana.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your option.
// This file may not be copied, modified, or distributed except
// according to those terms.

// Inspired by Maurice Svay's node-wifiscanner (https://github.com/mauricesvay/node-wifiscanner)

//! A crate to list WiFi hotspots in your area.
//!
//! As of v0.4.x now supports OSX, Linux, and Windows.
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

#[macro_use]
extern crate itertools;
extern crate regex;

#[doc(no_inline)]
use regex::Regex;

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
#[derive(Debug, PartialEq, Eq)]
pub struct Wifi {
    /// mac address
    pub mac: String,
    /// hotspot name
    pub ssid: String,
    pub channel: String,
    /// wifi signal strength in dBm
    pub signal_level: String,
    /// this field is currently empty in the Linux version of the lib
    pub security: String,
}

/// Returns a list of WiFi hotspots in your area - (OSX/MacOS) uses `airport`
#[cfg(target_os = "macos")]
pub fn scan() -> Result<Vec<Wifi>, Error> {
    use std::process::Command;
    let output = Command::new("/System/Library/PrivateFrameworks/Apple80211.\
    framework/Versions/Current/Resources/airport")
        .arg("-s")
        .output()
        .map_err(|_| Error::CommandNotFound)?;

    let data = String::from_utf8_lossy(&output.stdout);

    parse_airport(&data)
}

/// Returns a list of WiFi hotspots in your area - (Linux) uses `iwlist`
#[cfg(target_os = "linux")]
pub fn scan() -> Result<Vec<Wifi>, Error> {
    use std::env;
    use std::process::Command;

    const PATH_ENV: &'static str = "PATH";
    let path_system = "/usr/sbin:/sbin";
    let path = env::var_os(PATH_ENV).map_or(path_system.to_string(), |v| {
        format!("{}:{}", v.to_string_lossy().into_owned(), path_system)
    });

    let output = Command::new("iwlist")
        .env(PATH_ENV, path)
        .arg("scan")
        .output()
        .map_err(|_| Error::CommandNotFound)?;

    let data = String::from_utf8_lossy(&output.stdout);

    parse_iwlist(&data)
}

/// Returns a list of WiFi hotspots in your area - (Windows) uses `netsh`
#[cfg(target_os = "windows")]
pub fn scan() -> Result<Vec<Wifi>, Error> {
    use std::process::Command;
    let output = Command::new("netsh.exe")
        .args(&["wlan", "show", "networks", "mode=Bssid"])
        .output()
        .map_err(|_| Error::CommandNotFound)?;

    let data = String::from_utf8_lossy(&output.stdout);

    parse_netsh(&data)
}

#[allow(dead_code)]
fn parse_airport(network_list: &str) -> Result<Vec<Wifi>, Error> {
    let mut wifis: Vec<Wifi> = Vec::new();
    let mut lines = network_list.lines();
    let headers = lines.next().unwrap();

    let headers_string = String::from(headers);
    // FIXME: Turn these into non panicking Errors (ok_or breaks it)
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

#[allow(dead_code)]
fn parse_iwlist(network_list: &str) -> Result<Vec<Wifi>, Error> {
    let mut wifis: Vec<Wifi> = Vec::new();

    let cell_regex = Regex::new(r"Cell [0-9]{2,} - Address:")
        .map_err(|_| Error::SyntaxRegexError)?;

    let mac_regex =
        Regex::new(r"([0-9a-zA-Z]{1}[0-9a-zA-Z]{1}[:]{1}){5}[0-9a-zA-Z]{1}[0-9a-zA-Z]{1}")
            .map_err(|_| Error::SyntaxRegexError)?;

    for block in cell_regex.split(&network_list) {
        let mut lines = block.lines();

        let mut wifi_mac = String::new();
        let mut wifi_ssid = String::new();
        let mut wifi_channel = String::new();
        let mut wifi_rssi = String::new();
        let wifi_security = String::new(); // FIXME needs implementing

        let mac_matches = mac_regex.captures(lines.next().ok_or(Error::NoValue)?);

        if let Some(matches) = mac_matches {
            if let Some(mac) = matches.at(0) {
                wifi_mac = mac.to_string();
            }
        }

        for line in lines {
            if line.find("ESSID:").is_some() {
                let ssid = line.split(":").nth(1).unwrap_or("").replace("\"", "");
                wifi_ssid = ssid.to_string();
            } else if line.find("Frequency:").is_some() {
                wifi_channel = line.split("Channel")
                    .nth(1)
                    .unwrap_or("")
                    .replace(")", "")
                    .trim()
                    .to_string();
                // println!("Channel: {}", wifi_channel);
            } else if line.find("Signal level").is_some() {
                if line.find("Quality").is_some() {
                    // case1
                    wifi_rssi = line.split("Signal level=")
                        .nth(1)
                        .unwrap_or("")
                        .replace("dBm", "")
                        .trim()
                        .to_string();
                    // println!("Signal level (case1): {}", wifi_rssi);
                } else {
                    let re = Regex::new(r"Signal level=(\d+)/100")
                        .map_err(|_| Error::SyntaxRegexError)?;
                    let value_raw = re.captures(line).ok_or(Error::FailedToParse)?
                        .at(1)
                        .ok_or(Error::NoValue)?;
                    let value = value_raw.parse::<i32>().map_err(|_| Error::FailedToParse)?;
                    let strength_calc = ((100 * value) / 100) / 2 - 100;
                    wifi_rssi = strength_calc.to_string();

                    // println!("Signal level (case3): {}", wifi_rssi);
                }
            }


            // FIXME make less vomit inducing
            if !wifi_ssid.is_empty() && !wifi_mac.is_empty() && !wifi_rssi.is_empty() {
                wifis.push(Wifi {
                    ssid: wifi_ssid.to_string(),
                    mac: wifi_mac.to_string(),
                    channel: wifi_channel.to_string(),
                    signal_level: wifi_rssi.to_string(),
                    security: wifi_security.to_string(),
                });

                wifi_ssid = String::new();
                wifi_mac = String::new();
                wifi_rssi = String::new();
                wifi_channel = String::new();
            }
        } // for
    }
    Ok(wifis)
}

#[allow(dead_code)]
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
                wifi_macs.push(captures.at(0).unwrap());
            } else if line.find("Signal").is_some() {
                let percent = line.split(":").nth(1).unwrap_or("").trim().replace("%", "");
                let percent: i32 = percent.parse().unwrap();
                wifi_rssi.push(percent / 2 - 100);
            } else if line.find("Channel").is_some() {
                wifi_channels.push(line.split(":").nth(1).unwrap_or("").trim().to_string());
            }
        }

        for (mac, channel, rssi) in izip!(wifi_macs, wifi_channels, wifi_rssi) {
            wifis.push(
                Wifi {
                    mac: mac.to_string(),
                    ssid: wifi_ssid.to_string(),
                    channel: channel.to_string(),
                    signal_level: rssi.to_string(),
                    security: wifi_security.to_string(),
                }
            );
        }
    }

    Ok(wifis)
}

#[test]
fn should_parse_iwlist_type_1() {
    let mut expected: Vec<Wifi> = Vec::new();
    expected.push(Wifi {
        mac: "00:35:1A:6F:0F:40".to_string(),
        ssid: "TEST-Wifi".to_string(),
        channel: "6".to_string(),
        signal_level: "-72".to_string(),
        security: "".to_string(),
    });

    expected.push(Wifi {
        mac: "00:F2:8B:8F:58:77".to_string(),
        ssid: "<hidden>".to_string(),
        channel: "11".to_string(),
        signal_level: "-71".to_string(),
        security: "".to_string(),
    });

    // FIXME: should be a better way to create test fixtures
    use std::path::PathBuf;
    let mut path = PathBuf::new();
    path.push("tests");
    path.push("fixtures");
    path.push("iwlist");
    path.push("iwlist01_ubuntu1404.txt");

    let file_path = path.as_os_str();

    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(&file_path).unwrap();

    let mut filestr = String::new();
    let result = file.read_to_string(&mut filestr).unwrap();
    // println!("Read {} bytes", result);

    let result = parse_iwlist(&filestr).unwrap();
    assert_eq!(expected[0], result[0]);
    assert_eq!(expected[1], result[28]);
}

#[test]
fn should_parse_iwlist_type_2() {
    let mut expected: Vec<Wifi> = Vec::new();

    expected.push(Wifi {
        mac: "D4:D1:84:50:76:45".to_string(),
        ssid: "gsy-97796".to_string(),
        channel: "6".to_string(),
        signal_level: "-76".to_string(),
        security: "".to_string(),
    });

    expected.push(Wifi {
        mac: "7C:B7:33:AE:3B:05".to_string(),
        ssid: "visitor-18170".to_string(),
        channel: "9".to_string(),
        signal_level: "-70".to_string(),
        security: "".to_string(),
    });

    // FIXME: should be a better way to create test fixtures
    use std::path::PathBuf;
    let mut path = PathBuf::new();
    path.push("tests");
    path.push("fixtures");
    path.push("iwlist");
    path.push("iwlist02_raspi.txt");

    let file_path = path.as_os_str();

    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(&file_path).unwrap();

    let mut filestr = String::new();
    let result = file.read_to_string(&mut filestr).unwrap();
    // println!("Read {} bytes", result);

    let result = parse_iwlist(&filestr).unwrap();
    assert_eq!(expected[0], result[0]);
    assert_eq!(expected[1], result[2]);
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
    let result = file.read_to_string(&mut filestr).unwrap();
    // println!("Read {} bytes", result);

    let result = parse_airport(&filestr).unwrap();
    let last = result.len() - 1;
    assert_eq!(expected[0], result[0]);
    assert_eq!(expected[1], result[last]);
}

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
        }
    ];

    // Load test fixtures
    let fixture = fs::read_to_string("tests/fixtures/netsh/netsh01_windows81.txt").unwrap();

    let result = parse_netsh(&fixture).unwrap();
    assert_eq!(expected[0], result[0]);
    assert_eq!(expected[1], result[1]);
    assert_eq!(expected[2], result[2]);
    assert_eq!(expected[3], result[3]);
}
