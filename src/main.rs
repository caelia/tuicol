#![allow(unused_imports)]
#![allow(dead_code)]

mod common;
mod glikol;
mod config;

use glikol::{GlicolWrapper, GlicolAudioSource};
use common::{CtrlReq, CtrlRsp};
use config::Config;
use cpal::{Host, HostId, host_from_id};
use cpal::traits::HostTrait;
use rodio::OutputStream;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};

fn main() {
    let cfg = Config::default();
    let host = host_from_id(HostId::Jack).unwrap();
    let device = host.default_output_device().unwrap();
    let (_stream, handle) = OutputStream::try_from_device(&device).unwrap();
    let mut wrapper = GlicolWrapper::new(&cfg);
    thread::spawn(move || {
        wrapper.run();
    });
    let src = GlicolAudioSource::new(&cfg);
    let tx = cfg.ctrl_req_tx;
    let rx = cfg.ctrl_rsp_rx;
    let _ = tx.send(CtrlReq::Process(r#"o: sin 220"#));
    thread::spawn(move || {
        let astat = handle.play_raw(src);
        println!("astat: {:?}", astat);
    });
    thread::sleep(Duration::from_millis(1500));
    let _ = tx.send(CtrlReq::Stop);
    /*
    wrapper.eval("");
    thread::sleep(Duration::from_millis(1500));
    wrapper.eval(r#"o: sin 220"#);
    thread::sleep(Duration::from_millis(1500));
    */
}
