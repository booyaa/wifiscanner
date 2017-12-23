#[doc(no_inline)]
use regex::Regex;
use super::{Wifi, Error};

/// Returns a list of WiFi hotspots in your area - (Linux) uses `iwlist`
pub fn scan() -> Result<Vec<Wifi>, Error> {
    use std::env;
    use std::process::Command;

    const PATH_ENV: &'static str = "PATH";
    let path_system = "/usr/sbin:/sbin";
    let path = env::var_os(PATH_ENV).map_or(path_system.to_string(), |v| {
        format!("{}:{}", v.to_string_lossy().into_owned(), path_system)
    });

    let output = try!(Command::new("iwlist")
                      .env(PATH_ENV, path)
                      .arg("scan")
                      .output()
                      .map_err(|_| Error::CommandNotFound));

    let data = String::from_utf8_lossy(&output.stdout);

    parse_iwlist(&data)
}

#[allow(dead_code)]
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
        let wifi_security = String::new(); // FIXME needs implementing

        let mac_matches = mac_regex.captures(try!(lines.next().ok_or(Error::NoValue)));

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
                    let re = try!(Regex::new(r"Signal level=(\d+)/100")
                                      .map_err(|_| Error::SyntaxRegexError));
                    let value_raw = try!(try!(re.captures(line).ok_or(Error::FailedToParse))
                                             .at(1)
                                             .ok_or(Error::NoValue));
                    let value = try!(value_raw.parse::<i32>().map_err(|_| Error::FailedToParse));
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
