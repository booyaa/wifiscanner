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

extern crate regex;
use regex::Regex;


#[derive(Debug,PartialEq,Eq)]
pub enum Error {
    SyntaxRegexError,
    CommandNotFound,
}
// impl From<regex::Error> for Error {
//     fn from(err: regex::Error) -> Self {
//         Error::SyntaxRegexError(err)
//     }
// }

/// Wifi struct used to return information about wifi hotspots
#[derive(Debug,PartialEq,Eq)]
pub struct Wifi {
    pub mac: String,
    pub ssid: String,
    pub channel: String,
    pub signal_level: String,
    pub security: String,
}

/// Returns WiFi hotspots in your area (OSX/MacOS)
#[cfg(target_os="macos")]
// pub fn scan() -> Result<Vec<Wifi>, String> {
pub fn scan() -> Result<Vec<Wifi>, Error> {
    use std::process::Command;

    // let output = match Command::new("/System/Library/PrivateFrameworks/Apple80211.\
    // framework/Versions/Current/Resources/airport")
    //                        .arg("-s")
    //                        .output() {
    //     Ok(output) => output,
    //     Err(_) => return Err("Failed to find airport utility (are you using OSX?)".to_string()),
    // };
    let output = try!(Command::new("/System/Library/PrivateFrameworks/Apple80211.\
    framework/Versions/Current/Resources/airport")
                          .arg("-s")
                          .output()
                          .map_err(|_| Error::CommandNotFound));


    let data = String::from_utf8_lossy(&output.stdout);

    parse_airport(&data)
}

/// Returns WiFi hotspots in your area (Linux)
#[cfg(target_os="linux")]
pub fn scan() -> Result<Vec<Wifi>, Error> {
    use std::process::Command;
    let output = try!(Command::new("/usr/bin/iwlist")
                          .arg("scan")
                          .output()
                          .map_err(|_| Error::CommandNotFound));

    let data = String::from_utf8_lossy(&output.stdout);

    parse_iwlist(&data)
}



// fn parse_airport(network_list: &str) -> Result<Vec<Wifi>, String> {
fn parse_airport(network_list: &str) -> Result<Vec<Wifi>, Error> {
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

fn parse_iwlist(network_list: &str) -> Result<Vec<Wifi>, Error> {
    let mut wifis: Vec<Wifi> = Vec::new();

    let cell_regex = try!(Regex::new(r"Cell [0-9]{2,} - Address:")
                              .map_err(|_| Error::SyntaxRegexError));

    let mac_regex =
        try!(Regex::new(r"([0-9a-zA-Z]{1}[0-9a-zA-Z]{1}[:]{1}){5}[0-9a-zA-Z]{1}[0-9a-zA-Z]{1}")
                 .map_err(|_| Error::SyntaxRegexError));


    for block in cell_regex.split(&network_list) {
        let mut lines = block.lines();

        let mut wifi_mac = String::new();
        let mut wifi_ssid = String::new();
        let mut wifi_channel = String::new();
        let mut wifi_rssi = String::new();
        let wifi_security = String::new();


        let mac_matches = mac_regex.captures(lines.next().unwrap());
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
            } else if line.find("Signal level").is_some() {
                if line.find("Quality").is_some() {
                    // case1
                    wifi_rssi = line.split("Signal level=")
                                    .nth(1)
                                    .unwrap_or("")
                                    .replace("dBm", "")
                                    .trim()
                                    .to_string();
                } else {
                    // case 3 (raspi)
                    println!("rssi: case 3 pending");
                }
            }

            // FIXME make less vomit inducing
            if !wifi_ssid.is_empty() && !wifi_mac.is_empty() && !wifi_rssi.is_empty() {
                println!("ssid: {} mac: {} ", wifi_ssid, wifi_mac);

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

#[cfg(test)]
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
    println!("Read {} bytes", result);

    let result = parse_iwlist(&filestr).unwrap();
    assert_eq!(expected[0], result[0]);
    assert_eq!(expected[1], result[2]);
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
