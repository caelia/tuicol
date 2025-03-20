#![allow(unused_imports)]
#![allow(dead_code)]

mod glikol;

use glikol::glicol_wrapper;
use cpal::{Host, HostId, host_from_id};
use cpal::traits::HostTrait;
use rodio::OutputStream;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    let host = host_from_id(HostId::Jack).unwrap();
    let device = host.default_output_device().unwrap();
    let (_stream, handle) = OutputStream::try_from_device(&device).unwrap();
    let mut wrapper = glicol_wrapper();
    match Arc::get_mut(&mut wrapper) {
        Some(mtx) => {
            match mtx.get_mut() {
                Ok(wrpr) => {
                    wrpr.eval(r#"o: sin 220"#);
                },
                _ => ()
            }
        },
        None => ()
    }
    match Arc::get_mut(&mut wrapper) {
        Some(mtx) => {
            match mtx.get_mut() {
                Ok(wrpr) => {
                    let _ = handle.play_raw(wrpr);
                    thread::sleep(Duration::from_millis(1500));
                },
                _ => ()
            }
        },
        None => ()
    }
    /*
    wrapper.eval("");
    thread::sleep(Duration::from_millis(1500));
    wrapper.eval(r#"o: sin 220"#);
    thread::sleep(Duration::from_millis(1500));
    */
}
