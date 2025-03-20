#![allow(unused_imports)]
#![allow(dead_code)]

mod glikol;

use glikol::{GlicolWrapper, GlicolAudioSource, Req, Rsp};
use cpal::{Host, HostId, host_from_id};
use cpal::traits::HostTrait;
use rodio::OutputStream;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};

fn main() {
    let host = host_from_id(HostId::Jack).unwrap();
    let device = host.default_output_device().unwrap();
    let (_stream, handle) = OutputStream::try_from_device(&device).unwrap();
    let (req_tx, req_rx) = sync_channel(0);
    let (rsp_tx, rsp_rx) = sync_channel(0);
    let mut wrapper = GlicolWrapper::new(req_rx, rsp_tx);
    wrapper.run();
    let src = GlicolAudioSource::new(rsp_rx, req_tx.clone());
    let _ = req_tx.send(Req::Process(r#"o: sin 220"#));
    let _ = handle.play_raw(src);
    thread::sleep(Duration::from_millis(1500));
    let _ = req_tx.send(Req::Stop);
    /*
    wrapper.eval("");
    thread::sleep(Duration::from_millis(1500));
    wrapper.eval(r#"o: sin 220"#);
    thread::sleep(Duration::from_millis(1500));
    */
}
