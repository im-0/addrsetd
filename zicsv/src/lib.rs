//! Parser library for Zapret-Info CSV lists.
//! Supports dumps from <https://github.com/zapret-info/z-i> and its mirrors.
//!
//! Source code: <https://github.com/im-0/addrsetd>

#![cfg_attr(feature = "unstable", warn(unreachable_pub))]
#![forbid(unsafe_code)]
#![warn(unused_results)]

extern crate chrono;
extern crate csv;
extern crate encoding;

#[macro_use]
extern crate failure;

extern crate ipnet;

#[cfg(feature = "serialization")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "serialization")]
extern crate serde;

extern crate url;
#[cfg(feature = "serialization")]
extern crate url_serde;

#[cfg(feature = "serialization")]
mod ipnet_serde;

mod types;
pub use types::*;
