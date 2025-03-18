#![allow(unused_imports)]
#![allow(dead_code)]

mod glikol;

use glikol::GlicolWrapper;
use cpal::{Host, HostId, host_from_id};
use cpal::traits::HostTrait;
use rodio::OutputStream;
use std::time::Duration;
use std::thread;
// use std::Iter;

fn main() {
    let host = host_from_id(HostId::Jack).unwrap();
    let device = host.default_output_device().unwrap();
    let mut wrapper = GlicolWrapper::new();
    wrapper.eval(r#"o: sin 440"#);
    let (_stream, handle) = OutputStream::try_from_device(&device).unwrap();
    // let (_stream, handle) = OutputStream::try_default().unwrap();
    let _ = handle.play_raw(wrapper);
    thread::sleep(Duration::from_millis(1500));
}
