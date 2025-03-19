#![allow(unused_imports)]
#![allow(dead_code)]

mod glikol;

use glikol::GlicolWrapper;
use cpal::{Host, HostId, host_from_id};
use cpal::traits::HostTrait;
use rodio::OutputStream;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
// use std::Iter;

fn main() {
    let host = host_from_id(HostId::Jack).unwrap();
    let device = host.default_output_device().unwrap();
    let (_stream, handle) = OutputStream::try_from_device(&device).unwrap();
    let wrapper_ptr: &'static Arc<Mutex<GlicolWrapper>> = Arc::new(Mutex::new(GlicolWrapper::new()));
    /*
    wrapper.eval(r#"
        ~gate: speed 2.0
        >> seq 60 _60 _~a 48;
        ~a: choose 48 48 48 72 0 0 0
        ~amp: ~gate >> envperc 0.001 0.1;
        ~pit: ~gate
        >> mul ##Math.pow(2, (60-69)/12) * 440#
        // mix js to get 261.63
        ~lead: saw ~pit >> mul ~amp >> lpf ~mod 5.0
        >> meta `
            output = input.map(|x|x*0.1);
            output
        ` // rhai script, same as "mul 0.1"
        ~mod: sin 0.2 >> mul 1300 >> add 1500;
        out: ~lead >> add ~drum >> plate 0.1 // optinal semicolon
        ~drum: speed 4.0 >> seq 60 >> sp \808bd;
        "#);
        */
    // let (_stream, handle) = OutputStream::try_default().unwrap();
    /*
    if let Some(wrapper_mtx) = Arc::get_mut(&mut wrapper_ptr) {
        let wrapper = wrapper_mtx.get_mut().unwrap();
        let _ = handle.play_raw(wrapper);
    }
    if let Some(wrapper_mtx) = Arc::get_mut(&mut wrapper_ptr) {
        let wrapper = wrapper_mtx.get_mut().unwrap();
        wrapper.eval(r#"o: sin 440"#);
        thread::sleep(Duration::from_millis(1500));
    }
    if let Some(wrapper_mtx) = Arc::get_mut(&mut wrapper_ptr) {
        let wrapper = wrapper_mtx.get_mut().unwrap();
        wrapper.eval(r#""#);
        thread::sleep(Duration::from_millis(1500));
    }
    if let Some(wrapper_mtx) = Arc::get_mut(&mut wrapper_ptr) {
        let wrapper = wrapper_mtx.get_mut().unwrap();
        wrapper.eval(r#"o: saw 220"#);
        thread::sleep(Duration::from_millis(1500));
    }
    */
    /*
    wrapper.eval("");
    thread::sleep(Duration::from_millis(1500));
    wrapper.eval(r#"o: sin 220"#);
    thread::sleep(Duration::from_millis(1500));
    */
}
