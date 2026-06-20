#![deny(clippy::all)]

//! Node.js native bindings for Zenoh, built with NAPI-RS.
//!
//! The surface mirrors `zenoh`'s public API 1:1; only runtime mechanics
//! (async resolution, ownership, JS value marshaling) are adapted.

mod bytes;
mod config;
mod error;
mod qos;
mod sample;
mod session;
mod time;
