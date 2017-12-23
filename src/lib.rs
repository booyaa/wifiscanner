// Copyright 2016 Mark Sta Ana.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your option.
// This file may not be copied, modified, or distributed except
// according to those terms.

// Inspired by Maurice Svay's node-wifiscanner (https://github.com/mauricesvay/node-wifiscanner)

//! A crate to list WiFi hotspots in your area.
//!
//! As of v0.3.x now supports OSX and Linux. Windows to follow.
//!
//! # Usage
//!
//! This crate is [on crates.io](https://crates.io/crates/wifiscanner) and can be
//! used by adding `wifiscanner` to the dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! wifiscanner = "0.3.*"
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

#[cfg(target_os = "openbsd")]
#[macro_use]
extern crate nix;

#[cfg(any(target_os = "linux", target_os = "macos"))]
extern crate regex;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "openbsd")]
mod openbsd;

#[cfg(target_os = "linux")]
pub use linux::scan;
#[cfg(target_os = "macos")]
pub use macos::scan;
#[cfg(target_os = "openbsd")]
pub use openbsd::scan;

use std::convert;
use std::string::FromUtf8Error;

#[allow(missing_docs)]
#[derive(Debug, PartialEq)]
pub enum Error {
    SyntaxRegexError,
    CommandNotFound,
    NoMatch,
    FailedToParse,
    NoValue,
    FromUtf8Error,
    DiscoveryError(&'static str),

    #[cfg(target_os = "openbsd")]
    NixError(nix::Error),
}

#[cfg(target_os = "openbsd")]
impl convert::From<nix::Error> for Error {
    fn from(e: nix::Error) -> Self {
        Error::NixError(e)
    }
}

impl convert::From<FromUtf8Error> for Error {
    fn from(_e: FromUtf8Error) -> Self {
        Error::FromUtf8Error
    }
}

/// Wifi struct used to return information about wifi hotspots
#[derive(Debug, PartialEq, Eq)]
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
