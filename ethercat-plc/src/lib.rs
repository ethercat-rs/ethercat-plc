// Part of ethercat-rs. Copyright 2018-2024 by the authors.
// This work is dual-licensed under Apache 2.0 and MIT terms.

#![allow(clippy::type_complexity)]

mod plc;
mod image;
mod server;

pub mod beckhoff;
pub mod mlz_spec;

pub use self::plc::{Plc, PlcBuilder, PlcSimulator};
pub use self::image::{ExternImage, ProcessImage, ProcessConfig};
pub use self::server::{Server, NoServer, TcpServer, ModbusHandler, SimpleHandler};
pub use ethercat_derive::{ExternImage, ProcessImage, SlaveProcessImage};
