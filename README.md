# The `ethercat-plc` crate

[![Apache 2.0 licensed](https://img.shields.io/badge/license-Apache2.0-blue.svg)](./LICENSE-APACHE)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE-MIT)
[![crates.io](http://meritbadge.herokuapp.com/ethercat-plc)](https://crates.io/crates/ethercat-plc)
[![docs](https://docs.rs/ethercat-plc/badge.svg)](https://docs.rs/ethercat-plc)

## About

**Note: the crates in this repository are experimental.**

The `ethercat-plc` crate builds on the [`ethercat`
crate](https://github.com/ethercat-rs/ethercat) and tries to provide building
blocks for writing PLC like applications in Rust, talking to EtherCAT slaves.

## Building

The main EtherCAT functionality builds on the IgH/Etherlab [EtherCAT Master for
Linux](https://etherlab.org/en/ethercat/).

The IgH repository is located at <https://gitlab.com/etherlab.org/ethercat>.
Please switch to the ``stable-1.5`` branch in the checkout.

In order to build the raw wrapper crate `ethercat-sys`, you need to set the
environment variable `ETHERCAT_PATH` to the location of a checkout of the IgH
Etherlab repository, *after running `configure` there*.

The minimum tested Rust version is 1.63.0.

## Licensing

This crate uses the dual MIT/Apache-2 license commonly used for Rust crates.
