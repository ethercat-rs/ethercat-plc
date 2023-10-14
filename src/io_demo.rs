// Part of ethercat-rs. Copyright 2018-2023 by the authors.
// This work is dual-licensed under Apache 2.0 and MIT terms.

use ethercat_plc::{PlcBuilder, ProcessImage, ExternImage, TcpServer, ModbusHandler};
use ethercat_plc::beckhoff::*;

#[repr(C, packed)]
#[derive(ProcessImage)]
struct Image {
    coupler: EK1100,
    ios:     EL1859,
}

#[repr(C)]
#[derive(Default, ExternImage)]
struct Extern {
    magic: f32,
}

fn main() {
    let mut plc = PlcBuilder::new("plc")
        .cycle_freq(2)
        .with_server("0.0.0.0:5020")
        .logging_cfg(None, false)
        .build::<Image, Extern, _, TcpServer<ModbusHandler>>(()).unwrap();

    plc.run(|img, _| {
        img.ios.output ^= 1;
        println!("{}", img.ios.input);
    });
}
