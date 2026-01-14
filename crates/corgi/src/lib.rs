//! 🦀 Rust-based UDP RPC library
//!
//! # Introduction
//!
//! This library provides an RPC framework that allows you to build applications
//! without dealing with routing or low-level protocol implementation details.
//!
//! The **function signature itself acts as the RPC identifier**, making it
//! possible to determine which function should be invoked when an incoming call
//! is received.
//!
//! Internally, the framework uses **UDP** to prioritize **low latency and high
//! throughput**. Because of this design choice, the library is especially suited
//! for performance-critical systems where speed is more important than delivery
//! guarantees.
//!
//! ## ⚠️ Important note
//!
//! UDP does **not guarantee packet delivery, ordering, or duplication**.
//! This library should therefore be used only in domains where occasional packet
//! loss is acceptable, or where reliability is handled at a higher layer.
//!
//! Typical use cases include low-latency systems, internal services, and
//! performance-sensitive workloads.pub mod codec;
//!
//! # Example of usage
//!
//! ```no_run
//! use corgi::{rpc_fn, Container};
//!
//! #[rpc_fn]
//! async fn hello_world(name: String) -> String {
//!     format!("Hello, {}!", name)
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let container = Container::default()
//!         .register(&*__CORGI_RPC_hello_world);
//!
//!     // Start UDP listener / event loop here
//!
//!     Ok(())
//! }
//! ```
pub mod codec;
pub mod container;
pub mod protocol;

pub use container::Container;
pub use corgi_macros::rpc_fn;
