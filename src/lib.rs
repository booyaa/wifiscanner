// Copyright 2016 Mark Sta Ana.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your option.
// This file may not be copied, modified, or distributed except
// according to those terms.

// Inspired by Maurice Svay's node-wifiscanner (https://github.com/mauricesvay/node-wifiscanner)

//! A crate to list WiFi hotspots in your area.
//!
//! As of v0.5.x now supports macOS, Linux and Windows.
//!
//! # Usage
//!
//! This crate is on [crates.io](https://crates.io/crates/wifiscanner) and can be
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


//TODO need to find a way to move these out of lib and into sys or better still windows module
#[cfg(target_os = "windows")]
#[macro_use]
extern crate itertools;
#[cfg(target_os = "windows")]
extern crate regex;

mod sys;

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
    /// wifi signal strength in dBm
    pub signal_level: String,
    /// this field is currently empty in the Linux version of the lib
    pub security: String,
}

/// Returns a list of WiFi hotspots in your area.
/// Uses `airport` on macOS and `iw` on Linux.
pub fn scan() -> Result<Vec<Wifi>, Error> {
    crate::sys::scan()
}
