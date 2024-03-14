// Part of ethercat-rs. Copyright 2018-2024 by the authors.
// This work is dual-licensed under Apache 2.0 and MIT terms.

//! Modbus server allowing access to the PLC "memory" variables.

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::io::{Result, Read, Write, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::thread;
use log::*;
use byteorder::{ByteOrder, BE, LE};
use crossbeam_channel::{unbounded, Sender, Receiver};


#[derive(Debug)]
pub struct Request<T> {
    pub hid: usize,
    pub addr: usize,
    pub count: usize,
    pub write: Option<Vec<u8>>,
    pub extra: T,
}

#[derive(Debug)]
pub enum Response<T> {
    Ok(Request<T>, Vec<u8>),
    Error(Request<T>, u8),
}

pub trait Server {
    type Extra: Debug + Send + 'static;
    fn start(addr: &str, w_to_plc: Sender<Request<Self::Extra>>,
             r_from_plc: Receiver<Response<Self::Extra>>,) -> Result<()>;
}


pub struct NoServer;

impl Server for NoServer {
    type Extra = ();

    fn start(_: &str, _: Sender<Request<()>>, _: Receiver<Response<()>>) -> Result<()> {
        Ok(())
    }
}


pub enum HandlerEvent<T> {
    Request(Request<T>),
    New((usize, Sender<Response<T>>)),
    Finished(usize),
}

pub trait Handler {
    type Extra: Debug + Send + 'static;
    fn new(client: TcpStream, hid: usize, requests: Sender<HandlerEvent<Self::Extra>>,
           replies: Receiver<Response<Self::Extra>>) -> Self;
    fn sender(client: TcpStream, replies: Receiver<Response<Self::Extra>>);
    fn handle(self);
}

pub struct TcpServer<H: Handler> {
    to_plc:   Sender<Request<H::Extra>>,
    from_plc: Receiver<Response<H::Extra>>,
}

impl<H: Handler + Send + 'static> Server for TcpServer<H> {
    type Extra = H::Extra;

    fn start(addr: &str, w_to_plc: Sender<Request<H::Extra>>,
             r_from_plc: Receiver<Response<H::Extra>>,) -> Result<()> {
        let (w_clients, r_clients) = unbounded();
        let tcp_sock = TcpListener::bind(addr)?;

        let srv: Self = TcpServer { to_plc: w_to_plc, from_plc: r_from_plc };

        thread::spawn(move || Self::tcp_listener(tcp_sock, w_clients));
        thread::spawn(move || srv.dispatcher(r_clients));

        Ok(())
    }
}

impl<H: Handler + Send> TcpServer<H> {
    /// Listen for connections on the TCP socket and spawn handlers for it.
    fn tcp_listener(tcp_sock: TcpListener, handler_sender: Sender<HandlerEvent<H::Extra>>) {
        mlzlog::set_thread_prefix("TCP: ");

        info!("listening on {}", tcp_sock.local_addr().unwrap());
        let mut handler_id = 0;

        while let Ok((stream, _)) = tcp_sock.accept() {
            let (w_rep, r_rep) = unbounded();
            let w_req = handler_sender.clone();
            handler_id += 1;
            if let Err(e) = w_req.send(HandlerEvent::New((handler_id, w_rep))) {
                warn!("couldn't send new handler event: {}", e);
            } else {
                thread::spawn(move || H::new(stream, handler_id,
                                             w_req, r_rep).handle());
            }
        }
    }

    fn dispatcher(self, r_clients: Receiver<HandlerEvent<H::Extra>>) {
        mlzlog::set_thread_prefix("Dispatcher: ");

        let mut handlers = BTreeMap::new();

        for event in r_clients {
            match event {
                HandlerEvent::New((id, chan)) => {
                    handlers.insert(id, chan);
                }
                HandlerEvent::Finished(id) => {
                    handlers.remove(&id);
                }
                HandlerEvent::Request(req) => {
                    let hid = req.hid;
                    if let Err(e) = self.to_plc.send(req) {
                        warn!("couldn't send request to PLC: {}", e);
                    } else {
                        let resp = self.from_plc.recv().unwrap();
                        if let Err(e) = handlers[&hid].send(resp) {
                            warn!("couldn't send reply to handler: {}", e);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ModbusExtra {
    tid: u16,
    fc: u8,
}

pub struct ModbusHandler {
    hid:      usize,
    client:   TcpStream,
    requests: Sender<HandlerEvent<ModbusExtra>>,
}

impl Handler for ModbusHandler {
    type Extra = ModbusExtra;

    fn new(client: TcpStream, hid: usize, requests: Sender<HandlerEvent<ModbusExtra>>,
           replies: Receiver<Response<ModbusExtra>>) -> Self {
        let send_client = client.try_clone().expect("could not clone socket");
        thread::spawn(move || ModbusHandler::sender(send_client, replies));
        ModbusHandler { client, hid, requests }
    }

    fn sender(mut client: TcpStream, replies: Receiver<Response<ModbusExtra>>) {
        let mut buf = [0u8; 256];
        mlzlog::set_thread_prefix(format!("{} sender: ", client.peer_addr().unwrap()));

        for response in replies {
            debug!("sending response: {:?}", response);
            let count = match response {
                Response::Ok(req, values) => {
                    BE::write_u16(&mut buf, req.extra.tid);
                    buf[7] = req.extra.fc;
                    match req.extra.fc {
                        3 | 4 => {
                            let nbytes = values.len();
                            buf[8] = nbytes as u8;
                            buf[9..9+nbytes].copy_from_slice(&values);
                            9 + nbytes
                        }
                        6 => {
                            BE::write_u16(&mut buf[8..], req.addr as u16);
                            buf[10..12].copy_from_slice(&values);
                            12
                        }
                        16 => {
                            BE::write_u16(&mut buf[8..], req.addr as u16);
                            BE::write_u16(&mut buf[10..], values.len() as u16 / 2);
                            12
                        }
                        x => panic!("impossible function code {}", x)
                    }
                }
                Response::Error(req, ec) => {
                    BE::write_u16(&mut buf, req.extra.tid);
                    buf[7] = req.extra.fc | 0x80;
                    buf[8] = ec;
                    9
                }
            };
            BE::write_u16(&mut buf[4..], (count - 6) as u16);
            if let Err(err) = client.write_all(&buf[..count]) {
                warn!("write error: {}", err);
                break;
            }
        }
    }

    fn handle(mut self) {
        let mut headbuf = [0u8; 8];
        let mut bodybuf = [0u8; 250];  // max frame size is 255
        let mut errbuf  = [0, 0, 0, 0, 0, 9, 0, 0, 0];

        mlzlog::set_thread_prefix(format!("{}: ", self.client.peer_addr().unwrap()));
        info!("connection accepted");

        loop {
            if let Err(err) = self.client.read_exact(&mut headbuf) {
                if err.kind() != ErrorKind::UnexpectedEof {
                    warn!("error reading request head: {}", err);
                }
                break;
            }
            if headbuf[2..4] != [0, 0] {
                warn!("protocol ID mismatch: {:?}", headbuf);
                break;
            }
            let tid = BE::read_u16(&headbuf);
            let data_len = BE::read_u16(&headbuf[4..6]) as usize;
            if let Err(err) = self.client.read_exact(&mut bodybuf[..data_len - 2]) {
                warn!("error reading request body: {}", err);
                break;
            }
            if headbuf[6] != 0 {
                warn!("invalid slave {}", headbuf[6]);
                continue;
            }
            let fc = headbuf[7];
            let req = match fc {
                3 | 4 => { // read registers
                    if data_len != 6 {
                        warn!("invalid data length for fc {}", fc);
                        continue;
                    }
                    let addr = 2 * BE::read_u16(&bodybuf[..2]) as usize;
                    let count = 2 * BE::read_u16(&bodybuf[2..4]) as usize;
                    Request { hid: self.hid, addr, count, write: None,
                              extra: ModbusExtra { tid, fc } }
                }
                6 => { // write single register
                    if data_len != 6 {
                        warn!("invalid data length for fc {}", fc);
                        continue;
                    }
                    let addr = 2 * BE::read_u16(&bodybuf[..2]) as usize;
                    Request { hid: self.hid, addr, count: 2, write: Some(bodybuf[2..4].to_vec()),
                              extra: ModbusExtra { tid, fc } }
                }
                16 => { // write multiple registers
                    if data_len < 7 {
                        warn!("insufficient data length for fc {}", fc);
                        continue;
                    }
                    let addr = 2 * BE::read_u16(&bodybuf[..2]) as usize;
                    let bytecount = bodybuf[4] as usize;
                    if data_len != 7 + bytecount {
                        warn!("invalid data length for fc {}", fc);
                        continue;
                    }
                    let values = bodybuf[5..5+bytecount].to_vec();
                    Request { hid: self.hid, addr, count: values.len(), write: Some(values),
                              extra: ModbusExtra { tid, fc } }
                }
                _ => {
                    warn!("unknown function code {}", fc);
                    BE::write_u16(&mut errbuf, tid);
                    errbuf[7] = fc | 0x80;
                    errbuf[8] = 1;
                    if let Err(err) = self.client.write_all(&errbuf) {
                        warn!("error writing error response: {}", err);
                        break;
                    }
                    continue;
                }
            };
            debug!("got request: {:?}", req);
            if let Err(e) = self.requests.send(HandlerEvent::Request(req)) {
                warn!("couldn't send request to server: {}", e);
            }
        }
        info!("connection closed");
        if let Err(e) = self.requests.send(HandlerEvent::Finished(self.hid)) {
            warn!("couldn't send finish event to server: {}", e);
        }
    }
}


pub struct SimpleHandler {
    hid:      usize,
    client:   TcpStream,
    requests: Sender<HandlerEvent<bool>>,
}

const SIMPLE_READ:  u32 = 0x7EAD;
const SIMPLE_WRITE: u32 = 0xF71E;
const SIMPLE_ERR:   u32 = 0xE770;

impl Handler for SimpleHandler {
    type Extra = bool;

    fn new(client: TcpStream, hid: usize, requests: Sender<HandlerEvent<bool>>,
           replies: Receiver<Response<bool>>) -> Self {
        let send_client = client.try_clone().expect("could not clone socket");
        thread::spawn(move || SimpleHandler::sender(send_client, replies));
        SimpleHandler { client, hid, requests }
    }

    fn sender(mut client: TcpStream, replies: Receiver<Response<bool>>) {
        let mut buf = [0u8; 12];
        mlzlog::set_thread_prefix(format!("{} sender: ", client.peer_addr().unwrap()));

        for response in replies {
            debug!("sending response: {:?}", response);
            match response {
                Response::Ok(req, values) => {
                    if req.extra {
                        LE::write_u32(&mut buf, SIMPLE_READ);
                        LE::write_u32(&mut buf[4..], req.addr as u32);
                        LE::write_u32(&mut buf[8..], req.count as u32);
                        if let Err(err) = client.write_all(&buf) {
                            warn!("write error: {}", err);
                            break;
                        }
                        if let Err(err) = client.write_all(&values) {
                            warn!("write error: {}", err);
                            break;
                        }
                    } else {
                        LE::write_u32(&mut buf, SIMPLE_WRITE);
                        LE::write_u32(&mut buf[4..], req.addr as u32);
                        LE::write_u32(&mut buf[8..], req.count as u32);
                        if let Err(err) = client.write_all(&buf) {
                            warn!("write error: {}", err);
                            break;
                        }
                    }
                }
                Response::Error(req, ec) => {
                    LE::write_u32(&mut buf, SIMPLE_ERR);
                    LE::write_u32(&mut buf[4..], req.addr as u32);
                    LE::write_u32(&mut buf[4..], ec as u32);
                    if let Err(err) = client.write_all(&buf) {
                        warn!("write error: {}", err);
                        break;
                    }
                }
            }
        }
    }

    fn handle(mut self) {
        let mut headbuf = [0u8; 12];

        mlzlog::set_thread_prefix(format!("{}: ", self.client.peer_addr().unwrap()));
        info!("connection accepted");

        loop {
            if let Err(err) = self.client.read_exact(&mut headbuf) {
                if err.kind() != ErrorKind::UnexpectedEof {
                    warn!("error reading request head: {}", err);
                }
                break;
            }
            let func = LE::read_u32(&headbuf);
            let addr = LE::read_u32(&headbuf[4..]) as usize;
            let count = LE::read_u32(&headbuf[8..]) as usize;
            let req = if func == SIMPLE_READ {
                Request { hid: self.hid, addr, count, write: None, extra: true }
            } else if func == SIMPLE_WRITE {
                let mut bodybuf = Vec::new();
                if let Err(err) = std::io::Write::by_ref(&mut self.client)
                    .take(count as u64).read_to_end(&mut bodybuf)
                {
                    warn!("error reading request body: {}", err);
                    break;
                }
                if bodybuf.len() != count {
                    warn!("error reading request body: connection closed");
                    break;
                }
                Request { hid: self.hid, addr, count, write: Some(bodybuf), extra: false }
            } else {
                warn!("invalid function {}", func);
                continue;
            };
            debug!("got request: {:?}", req);
            if let Err(e) = self.requests.send(HandlerEvent::Request(req)) {
                warn!("couldn't send request to server: {}", e);
            }
        }
        info!("connection closed");
        if let Err(e) = self.requests.send(HandlerEvent::Finished(self.hid)) {
            warn!("couldn't send finish event to server: {}", e);
        }
    }
}
