// Part of ethercat-rs. Copyright 2018-2024 by the authors.
// This work is dual-licensed under Apache 2.0 and MIT terms.

//! Wrap an EtherCAT master and slave configuration and provide a PLC-like
//! environment for cyclic task execution.

use std::{thread, time::{Instant, Duration}, marker::PhantomData};
use anyhow::{bail, Context};
use crossbeam_channel::{unbounded, Sender, Receiver};
use log::*;
use ethercat as ec;

use crate::image::{ProcessImage, ExternImage, ProcessConfig};
use crate::server::{Server, Request, Response};

#[derive(Default)]
pub struct PlcBuilder {
    name: String,
    master_id: Option<u32>,
    cycle_freq: Option<u32>,
    server_addr: Option<String>,
    logfile_base: Option<String>,
    debug_logging: bool,
}

impl PlcBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            .. Self::default()
        }
    }

    pub fn master_id(mut self, id: u32) -> Self {
        self.master_id = Some(id);
        self
    }

    pub fn cycle_freq(mut self, freq: u32) -> Self {
        self.cycle_freq = Some(freq);
        self
    }

    pub fn with_server(mut self, addr: impl Into<String>) -> Self {
        self.server_addr = Some(addr.into());
        self
    }

    pub fn logging_cfg(mut self, logfile_base: Option<String>, debug_logging: bool) -> Self {
        self.logfile_base = logfile_base;
        self.debug_logging = debug_logging;
        self
    }

    pub fn build_simulator<E: ExternImage, S: Server>(self) -> anyhow::Result<PlcSimulator<E, S>> {
        mlzlog::init(self.logfile_base, &self.name,
                     mlzlog::Settings { show_appname: false,
                                        debug: self.debug_logging,
                                        ..Default::default() })
            .context("setting up logging")?;

        let channels = if let Some(addr) = self.server_addr {
            let (w_from_plc, r_from_plc) = unbounded();
            let (w_to_plc, r_to_plc) = unbounded();
            S::start(&addr, w_to_plc, r_from_plc)
                .context("starting external server")?;
            Some((r_to_plc, w_from_plc))
        } else {
            None
        };

        Ok(PlcSimulator {
            server_channel: channels,
            sleep: 1_000_000_000 / self.cycle_freq.unwrap_or(1000) as u64,
            _types: PhantomData,
        })
    }

    pub fn build<P: ProcessImage, E: ExternImage, PC: ProcessConfig,
                 S: Server>(self, cfg: PC) -> anyhow::Result<Plc<P, E, S>> {
        mlzlog::init(self.logfile_base, &self.name,
                     mlzlog::Settings { show_appname: false,
                                        debug: self.debug_logging,
                                        ..Default::default() })
            .context("setting up logging")?;

        let channels = if let Some(addr) = self.server_addr {
            let (w_from_plc, r_from_plc) = unbounded();
            let (w_to_plc, r_to_plc) = unbounded();
            S::start(&addr, w_to_plc, r_from_plc)
                .context("starting external server")?;
            Some((r_to_plc, w_from_plc))
        } else {
            None
        };

        let mut master = ec::Master::open(self.master_id.unwrap_or(0),
                                          ec::MasterAccess::ReadWrite)
            .context("opening Ethercat master")?;
        master.reserve()?;
        let domain = master.create_domain()
            .context("creating Ethercat domain")?;

        debug!("PLC: EtherCAT master opened");

        let slave_ids = P::get_slave_ids();
        let slave_pdos = P::get_slave_pdos();
        let slave_regs = P::get_slave_regs();
        let slave_sdos = P::get_slave_sdos(&cfg);
        let slave_wd_dcs = P::get_slave_wd_dc();
        for (i, ((((id, pdos), regs), sdos), wd_dc)) in slave_ids.into_iter()
                                                        .zip(slave_pdos)
                                                        .zip(slave_regs)
                                                        .zip(slave_sdos)
                                                        .zip(slave_wd_dcs)
                                                        .enumerate()
        {
            let mut config = master.configure_slave(ec::SlaveAddr::ByPos(i as u16), id)
                                   .with_context(|| format!("configuring slave {} with id {:?}", i, id))?;
            if let Some(sm_pdos) = pdos {
                for (sm, pdos) in sm_pdos {
                    config.config_sm_pdos(sm, &pdos)
                          .with_context(|| format!("configuring slave {} with pdos for sync \
                                                    manager {:?}", i, sm))?;
                }
            }
            let mut first_byte = 0;
            for (j, (entry, mut expected_position)) in regs.into_iter().enumerate() {
                let pos = config.register_pdo_entry(entry, domain)
                                .with_context(|| format!("registering slave {} pdo {:?}", i, entry))?;
                if j == 0 {
                    if pos.bit != 0 {
                        bail!("first PDO of slave {} not byte-aligned", i);
                    }
                    first_byte = pos.byte;
                } else {
                    expected_position.byte += first_byte;
                    if pos != expected_position {
                        bail!("slave {} pdo {}: {:?} != {:?}", i, j, pos, expected_position);
                    }
                }
            }

            for (sdo_index, data) in sdos {
                config.add_sdo(sdo_index, data)
                      .with_context(|| format!("adding slave {} sdo {:?}", i, sdo_index))?;
            }

            if let Some((div, int)) = wd_dc.0 {
                config.config_watchdog(div, int)
                      .with_context(|| format!("configuring slave {} watchdog", i))?;
            }

            if let Some((act, cyc0, sh0, cyc1, sh1)) = wd_dc.1 {
                config.config_dc(act, cyc0, sh0, cyc1, sh1)
                      .with_context(|| format!("configuring slave {} dist. clock", i))?;
            }

            let cfg_index = config.index();

            // ensure that the slave is actually present
            if master.get_config_info(cfg_index)?.slave_position.is_none() {
                bail!("slave {} does not match config", i);
            }
        }

        info!("PLC: EtherCAT slaves configured");

        let domain_size = master.domain(domain).size()?;
        if domain_size != P::size() {
            bail!("domain size mismatch: real {} != assumed {}", domain_size, P::size());
        }

        master.set_application_time(1)
            .context("setting application time")?;  // 0 is not good
        master.activate()
            .context("activating master")?;
        info!("PLC: EtherCAT master activated");

        Ok(Plc {
            master,
            domain,
            server_channel: channels,
            sleep: 1_000_000_000 / self.cycle_freq.unwrap_or(1000) as u64,
            _types: PhantomData,
        })
    }
}

pub type ServerChannels<X> = (Receiver<Request<X>>, Sender<Response<X>>);

pub fn data_exchange<E: ExternImage, X: std::fmt::Debug>(chan: &mut ServerChannels<X>, ext: &mut E) {
    while let Ok(mut req) = chan.0.try_recv() {
        let mut done = false;
        debug!("PLC sim got request: {:?}", req);
        let data = ext.cast();
        let resp = if req.addr + req.count > E::size() {
            Response::Error(req, 2)
        } else {
            let from = req.addr;
            let to = from + req.count;
            if let Some(ref mut values) = req.write {
                // write request
                data[from..to].copy_from_slice(values);
                let values = req.write.take().unwrap();
                // let a PLC cycle run after a write request
                done = true;
                Response::Ok(req, values)
            } else {
                // read request
                Response::Ok(req, data[from..to].to_vec())
            }
        };
        debug!("PLC sim response: {:?}", resp);
        if let Err(e) = chan.1.send(resp) {
            warn!("could not send back response: {}", e);
        }
        if done {
            break;
        }
    }
}


pub struct Plc<P, E, S: Server> {
    master: ec::Master,
    domain: ec::DomainIdx,
    sleep:  u64,
    server_channel: Option<ServerChannels<S::Extra>>,
    _types: PhantomData<(P, E)>,
}

impl<P: ProcessImage, E: ExternImage, S: Server> Plc<P, E, S> {
    pub fn run<F>(&mut self, mut cycle_fn: F)
    where F: FnMut(&mut P, &mut E)
    {
        let mut ext = E::default();
        let mut cycle_start = Instant::now();

        loop {
            // process data exchange + logic
            if let Err(e) = self.single_cycle(&mut cycle_fn, &mut ext) {
                // XXX: logging unconditionally here is bad, could repeat endlessly
                warn!("error in cycle: {}", e);
            }

            // external data exchange
            if let Some(chan) = self.server_channel.as_mut() {
                data_exchange(chan, &mut ext);
            }

            // wait until next cycle
            let now = Instant::now();
            cycle_start += Duration::from_nanos(self.sleep);
            if cycle_start > now {
                thread::sleep(cycle_start - now);
            }
        }
    }

    fn single_cycle<F>(&mut self, mut cycle_fn: F, ext: &mut E) -> anyhow::Result<()>
    where F: FnMut(&mut P, &mut E)
    {
        self.master.receive()
            .context("receiving Ethercat data")?;
        self.master.domain(self.domain).process()
            .context("processing domain data")?;

        // XXX: check working counters periodically, etc.
        // println!("master state: {:?}", self.master.state());
        // println!("domain state: {:?}", self.master.domain(self.domain).state());

        let data = P::cast(self.master.domain_data(self.domain)?);
        cycle_fn(data, ext);

        self.master.domain(self.domain).queue()
            .context("queueing new domain data")?;
        self.master.send()
            .context("sending Ethercat data")?;
        Ok(())
    }
}


/// An object similar to Plc, but not connected to an Ethercat master.
pub struct PlcSimulator<E, S: Server> {
    sleep: u64,
    server_channel: Option<(Receiver<Request<S::Extra>>, Sender<Response<S::Extra>>)>,
    _types: PhantomData<E>,
}

impl<E: ExternImage, S: Server> PlcSimulator<E, S> {
    pub fn run<F>(&mut self, mut cycle_fn: F)
    where F: FnMut(&mut E)
    {
        let mut ext = E::default();
        let mut cycle_start = Instant::now();

        loop {
            // simulate a cycle
            cycle_fn(&mut ext);

            // data exchange with upper layer
            if let Some(chan) = self.server_channel.as_mut() {
                data_exchange(chan, &mut ext);
            }

            // wait until next cycle
            let now = Instant::now();
            cycle_start += Duration::from_nanos(self.sleep);
            if cycle_start > now {
                thread::sleep(cycle_start - now);
            }
        }
    }
}
