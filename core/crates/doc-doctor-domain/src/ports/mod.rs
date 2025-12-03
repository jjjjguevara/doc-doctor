//! Domain Ports
//!
//! Ports define the interfaces between the domain and the outside world.
//! Following Hexagonal Architecture:
//!
//! - **Inbound (Driving)**: Use cases that external actors invoke
//! - **Outbound (Driven)**: Services that the domain needs

pub mod inbound;
pub mod outbound;
