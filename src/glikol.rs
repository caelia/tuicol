#![allow(unused_imports)]

use crate::common::{Channel, AudioReq, AudioRsp, CtrlReq, CtrlRsp};
use crate::config::Config;
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
    audio_rx: &Receiver<AudioReq>,
    audio_tx: SyncSender<AudioRsp>,
    ctrl_rx: Receiver<CtrlReq>,
    ctrl_tx: Sender<CtrlRsp>,
}

impl GlicolWrapper {
    pub fn new(config: &Config) -> Self {
        GlicolWrapper {
            engine: Engine::<32>::new(),
            audio_rx: &config.audio_req_rx,
            audio_tx: config.audio_rsp_tx,
            ctrl_rx: config.ctrl_req_rx,
            ctrl_tx: config.ctrl_rsp_tx
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
                            let _ = self.audio_tx.send(AudioRsp::Ok);
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
                    break;
                }
            }
            let audio_msg = self.audio_rx.recv();
            match audio_msg {
                Ok(req) => {
                    match req {
                        AudioReq::NextBlock => {
                            match self.update() {
                                Some(data) => { let _ = self.audio_tx.send(AudioRsp::Data(data)); },
                                None => { let _ = self.audio_tx.send(AudioRsp::NoData); }
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
    rx: Receiver<AudioRsp>,
    tx: SyncSender<AudioReq>
}

impl GlicolAudioSource {
    pub fn new(config: Config) -> Self {
        GlicolAudioSource {
            channel: Channel::L,
            data_l: VecDeque::new(),
            data_r: VecDeque::new(),
            rx: config.audio_rsp_rx,
            tx: config.audio_req_tx
        }
    }
}
impl Iterator for GlicolAudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let result = {
            let result_ = match self.channel {
                Channel::L => self.data_l.pop_front(),
                Channel::R => self.data_r.pop_front()
            };
            match result_ {
                Some(sample) => Some(sample),
                None => {
                    let _ = self.tx.send(AudioReq::NextBlock);
                    let msg = self.rx.recv();
                    match msg {
                        Ok(rsp) => {
                            match rsp {
                                AudioRsp::Data((left, right)) => {
                                    self.data_l = left;
                                    self.data_r = right;
                                },
                                AudioRsp::NoData => (),
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
