// Part of ethercat-rs. Copyright 2018-2019 by the authors.
// This work is dual-licensed under Apache 2.0 and MIT terms.

//! Wrap an EtherCAT master and slave configuration and provide a PLC-like
//! environment for cyclic task execution.

use std::{thread, time::Duration, marker::PhantomData};
use time::precise_time_ns;
use crossbeam_channel::{unbounded, Sender, Receiver};
use mlzlog;
use log::*;

use ethercat::*;

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

    pub fn build_simulator<E: ExternImage, S: Server>(self) -> Result<PlcSimulator<E, S>> {
        mlzlog::init(self.logfile_base, &self.name, false, self.debug_logging, true)?;

        let channels = if let Some(addr) = self.server_addr {
            let (w_from_plc, r_from_plc) = unbounded();
            let (w_to_plc, r_to_plc) = unbounded();
            S::start(&addr, w_to_plc, r_from_plc)?;
            Some((r_to_plc, w_from_plc))
        } else {
            None
        };

        Ok(PlcSimulator {
            server_channel: channels,
            sleep: 1000_000_000 / self.cycle_freq.unwrap_or(1000) as u64,
            _types: PhantomData,
        })
    }

    pub fn build<P: ProcessImage, E: ExternImage, PC: ProcessConfig, S: Server>(self, cfg: PC) -> Result<Plc<P, E, S>> {
        mlzlog::init(self.logfile_base, &self.name, false, self.debug_logging, true)?;

        let channels = if let Some(addr) = self.server_addr {
            let (w_from_plc, r_from_plc) = unbounded();
            let (w_to_plc, r_to_plc) = unbounded();
            S::start(&addr, w_to_plc, r_from_plc)?;
            Some((r_to_plc, w_from_plc))
        } else {
            None
        };

        let mut master = Master::open(self.master_id.unwrap_or(0), MasterAccess::ReadWrite)?;
        master.reserve()?;
        let domain = master.create_domain()?;

        debug!("PLC: EtherCAT master opened");

        // XXX
        // master.sdo_download(1, SdoIndex::new(0x1011, 1), &0x64616F6Cu32)?;
        // master.sdo_download(2, SdoIndex::new(0x1011, 1), &0x64616F6Cu32)?;

        let slave_ids = P::get_slave_ids();
        let slave_pdos = P::get_slave_pdos();
        let slave_regs = P::get_slave_regs();
        let slave_sdos = P::get_slave_sdos(&cfg);
        for (i, (((id, pdos), regs), sdos)) in slave_ids.into_iter()
                                                        .zip(slave_pdos)
                                                        .zip(slave_regs)
                                                        .zip(slave_sdos)
                                                        .enumerate()
        {
            let mut config = master.configure_slave(SlaveAddr::ByPos(i as u16), id)?;
            if let Some(sm_pdos) = pdos {
                for (sm, pdos) in sm_pdos {
                    config.config_sm_pdos(sm, &pdos)?;
                }
            }
            let mut first_byte = 0;
            for (j, (entry, mut expected_position)) in regs.into_iter().enumerate() {
                let pos = config.register_pdo_entry(entry, domain)?;
                if j == 0 {
                    if pos.bit != 0 {
                        panic!("first PDO of slave {} not byte-aligned", i);
                    }
                    first_byte = pos.byte;
                } else {
                    expected_position.byte += first_byte;
                    if pos != expected_position {
                        panic!("slave {} pdo {}: {:?} != {:?}", i, j, pos, expected_position);
                    }
                }
            }

            for (sdo_index, data) in sdos {
                config.add_sdo(sdo_index, &*data)?;
            }

            let cfg_index = config.index();
            drop(config);

            // ensure that the slave is actually present
            if master.get_config_info(cfg_index)?.slave_position.is_none() {
                panic!("slave {} does not match config", i);
            }
        }

        info!("PLC: EtherCAT slaves configured");

        let domain_size = master.domain(domain).size()?;
        if domain_size != P::size() {
            panic!("size: {} != {}", domain_size, P::size());
        }

        master.activate()?;
        info!("PLC: EtherCAT master activated");

        Ok(Plc {
            master: master,
            domain: domain,
            server_channel: channels,
            sleep: 1000_000_000 / self.cycle_freq.unwrap_or(1000) as u64,
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
                data[from..to].copy_from_slice(&values);
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
    master: Master,
    domain: DomainIdx,
    sleep:  u64,
    server_channel: Option<ServerChannels<S::Extra>>,
    _types: PhantomData<(P, E)>,
}

impl<P: ProcessImage, E: ExternImage, S: Server> Plc<P, E, S> {
    pub fn run<F>(&mut self, mut cycle_fn: F)
    where F: FnMut(&mut P, &mut E)
    {
        let mut ext = E::default();
        let mut cycle_start = precise_time_ns();

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
            let now = precise_time_ns();
            cycle_start += self.sleep;
            if cycle_start > now {
                thread::sleep(Duration::from_nanos(cycle_start - now));
            }
        }
    }

    fn single_cycle<F>(&mut self, mut cycle_fn: F, ext: &mut E) -> Result<()>
    where F: FnMut(&mut P, &mut E)
    {
        self.master.receive()?;
        self.master.domain(self.domain).process()?;

        // XXX: check working counters periodically, etc.
        // println!("master state: {:?}", self.master.state());
        // println!("domain state: {:?}", self.master.domain(self.domain).state());

        let data = P::cast(self.master.domain_data(self.domain)?);
        cycle_fn(data, ext);

        self.master.domain(self.domain).queue()?;
        self.master.send()?;
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
        let mut cycle_start = precise_time_ns();

        loop {
            // simulate a cycle
            cycle_fn(&mut ext);

            // data exchange with upper layer
            if let Some(chan) = self.server_channel.as_mut() {
                data_exchange(chan, &mut ext);
            }

            // wait until next cycle
            let now = precise_time_ns();
            cycle_start += self.sleep;
            if cycle_start > now {
                thread::sleep(Duration::from_nanos(cycle_start - now));
            }
        }
    }
}
