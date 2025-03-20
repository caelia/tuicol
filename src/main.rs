#![allow(unused_imports)]
#![allow(dead_code)]

mod common;
mod glikol;
// mod config;

use glikol::{GlicolWrapper, GlicolAudioSource};
use common::{CtrlReq, CtrlRsp};
// use config::Config;
use cpal::{Host, HostId, host_from_id};
use cpal::traits::HostTrait;
use rodio::OutputStream;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, sync_channel, Sender, SyncSender, Receiver};

fn main() {
    // let cfg = Config::default();

    let (data_req_tx, data_req_rx) = sync_channel(1);
    let (data_rsp_tx, data_rsp_rx) = sync_channel(1);
    let (ctrl_req_tx, ctrl_req_rx) = channel();
    let (ctrl_rsp_tx, ctrl_rsp_rx) = channel();
        
    let (tx, _rx) = (ctrl_req_tx, ctrl_rsp_rx);
    let mut wrapper = GlicolWrapper::new(data_rsp_tx, data_req_rx, ctrl_rsp_tx, ctrl_req_rx);
    let _ = tx.send(CtrlReq::Process(r#"o: sin 440"#));
    thread::spawn(move || {
        wrapper.run();
    });

    let src = GlicolAudioSource::new(data_req_tx, data_rsp_rx);

    // let (tx, rx) = (ctrl_req_tx, ctrl_rsp_rx);

    let host = host_from_id(HostId::Jack).unwrap();
    let device = host.default_output_device().unwrap();
    let (_stream, handle) = OutputStream::try_from_device(&device).unwrap();

    // let _ = tx.send(CtrlReq::Process(r#"o: sin 220"#));
    // thread::sleep(Duration::from_millis(100));
    thread::spawn(move || {
        let _ = handle.play_raw(src);
    });
    thread::sleep(Duration::from_millis(1500));

    let _ = tx.send(CtrlReq::Process(r#""#));
    thread::sleep(Duration::from_millis(1500));
    let _ = tx.send(CtrlReq::Process(r#"o: saw 220"#));
    thread::sleep(Duration::from_millis(1500));

    let _ = tx.send(CtrlReq::Stop);
    /*
    wrapper.eval("");
    thread::sleep(Duration::from_millis(1500));
    wrapper.eval(r#"o: sin 220"#);
    thread::sleep(Duration::from_millis(1500));
    */
}
