#![allow(unused_imports)]
#![allow(static_mut_refs)]

// For counting the number of iterations of next()
// static mut COUNTER: u32 = 0;

use crate::common::{Channel, DataReq, DataRsp, CtrlReq, CtrlRsp};
// use crate::config::Config;
use glicol::Engine;
use rodio::Source;
use std::collections::VecDeque;
use std::iter::Iterator;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, SyncSender, Receiver};
// use std::cell::{RefCell, RefMut};

pub struct GlicolWrapper {
    engine: glicol::Engine<32>,
    data_rx: Receiver<DataReq>,
    data_tx: SyncSender<DataRsp>,
    ctrl_rx: Receiver<CtrlReq>,
    ctrl_tx: Sender<CtrlRsp>,
}

impl GlicolWrapper {
    pub fn new(data_tx: SyncSender<DataRsp>,
               data_rx: Receiver<DataReq>,
               ctrl_tx: Sender<CtrlRsp>,
               ctrl_rx: Receiver<CtrlReq> ) -> Self {
        GlicolWrapper {
            engine: Engine::<32>::new(),
            data_tx,
            data_rx,
            ctrl_tx,
            ctrl_rx
        }
    }

    fn eval(&mut self, code: &str) {
        self.engine.update_with_code(code);
    }

    fn update(&mut self) -> Option<(VecDeque<f32>, VecDeque<f32>)> {
        let (bufs, _mystery_data) = self.engine.next_block(vec![]);
        if bufs[0].is_empty() || bufs[1].is_empty() {
            None
        } else {
            // println!("{:?}", bufs[0]);
            // println!("{:?}", bufs[1]);
            Some((VecDeque::from(bufs[0].to_vec()),
                  VecDeque::from(bufs[1].to_vec())))
        }
    }

    pub fn run(&mut self) {
        loop {
            let ctrl_msg = self.ctrl_rx.try_recv();
            match ctrl_msg {
                Ok(req) => {
                    match req {
                        CtrlReq::Process(code) => {
                            self.eval(code);
                            let _ = self.data_tx.send(DataRsp::Ok);
                        },
                        CtrlReq::Stop => {
                            let _ = self.ctrl_tx.send(CtrlRsp::Ok);
                            break;
                        },
                        CtrlReq::Start | CtrlReq::Pause | CtrlReq::Resume => {
                            let _ = self.ctrl_tx.send(CtrlRsp::Ok);
                        }
                    }
                },
                _ => {
                    let _ = self.ctrl_tx.send(CtrlRsp::Error);
                }
            }
            let data_msg = self.data_rx.recv();
            match data_msg {
                Ok(req) => {
                    match req {
                        DataReq::NextBlock => {
                            match self.update() {
                                Some(data) => { let _ = self.data_tx.send(DataRsp::Data(data)); },
                                None => { let _ = self.data_tx.send(DataRsp::NoData); }
                            }
                        },
                    }
                },
                _ => panic!("Error receiving audio request!")
            }
        }
    }
}

pub struct GlicolAudioSource {
    channel: Channel,
    data_l: VecDeque<f32>,
    data_r: VecDeque<f32>,
    tx: SyncSender<DataReq>,
    rx: Receiver<DataRsp>,
}

impl GlicolAudioSource {
    pub fn new(tx: SyncSender<DataReq>, rx: Receiver<DataRsp>) -> Self {
        GlicolAudioSource {
            channel: Channel::L,
            data_l: VecDeque::new(),
            data_r: VecDeque::new(),
            tx,
            rx,
        }
    }
}
impl Iterator for GlicolAudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        /*
        unsafe {
            COUNTER += 1;
            println!("{}", COUNTER);
        }
        */
        let result = {
            let result_ = match self.channel {
                Channel::L => self.data_l.pop_front(),
                Channel::R => self.data_r.pop_front()
            };
            match result_ {
                Some(sample) => Some(sample),
                None => {
                    // println!("MOAR SAMPULZ PLZ!");
                    let _ = self.tx.send(DataReq::NextBlock);
                    let msg = self.rx.recv();
                    // println!("{:?}", msg);
                    match msg {
                        Ok(rsp) => {
                            match rsp {
                                DataRsp::Data((left, right)) => {
                                    self.data_l = left;
                                    self.data_r = right;
                                },
                                DataRsp::NoData => (),
                                _ => ()
                            }
                        },
                        _ => ()
                    }
                    match self.channel {
                        Channel::L => self.data_l.pop_front(),
                        Channel::R => self.data_r.pop_front()
                    }
                } 
            }
        };
        self.channel = match self.channel {
            Channel::L => Channel::R,
            Channel::R => Channel::L
        };
        result
    }
}

impl Source for GlicolAudioSource {
    fn current_frame_len(&self) -> Option<usize> {
       let len = self.data_l.len() + self.data_r.len();     
       if len == 0 {
           None
       } else {
           Some(len)
       }
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
