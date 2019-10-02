use crate::{Error, Wifi};

/// Returns a list of WiFi hotspots in your area - (OSX/MacOS) uses `airport`
pub(crate) fn scan() -> Result<Vec<Wifi>, Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

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
        let mut path = PathBuf::new();
        path.push("tests");
        path.push("fixtures");
        path.push("airport");
        path.push("airport01.txt");

        let file_path = path.as_os_str();

        let mut file = File::open(&file_path).unwrap();

        let mut filestr = String::new();
        let _ = file.read_to_string(&mut filestr).unwrap();

        let result = parse_airport(&filestr).unwrap();
        let last = result.len() - 1;
        assert_eq!(expected[0], result[0]);
        assert_eq!(expected[1], result[last]);
    }
}
