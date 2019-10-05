#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use self::macos::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use self::linux::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use self::windows::*;

#[cfg(target_os = "windows")]
#[macro_use]
extern crate itertools;
#[cfg(target_os = "windows")]
extern crate regex;

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
compile_error!("wifiscan doesn't compile for this platform yet");
