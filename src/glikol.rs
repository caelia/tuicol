#![allow(unused_imports)]

use glicol::Engine;
use rodio::Source;
use std::collections::VecDeque;
use std::iter::Iterator;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{SyncSender, Receiver};
// use std::cell::{RefCell, RefMut};

#[derive(Clone)]
enum Channel {
    L,
    R
}

pub enum State {
    Stopped,
    Paused,
    Running
}

pub enum Req {
    NextBlock,
    Stop,
    Start,
    Pause,
    Resume,
    Process(&'static str)
}

pub enum Rsp {
    Data((VecDeque<f32>, VecDeque<f32>)),
    NoData,
    Ok,
    Error    
}

pub struct GlicolWrapper {
    engine: glicol::Engine<32>,
    rx: Receiver<Req>,
    tx: SyncSender<Rsp>,
}

impl GlicolWrapper {
    pub fn new(rx: Receiver<Req>, tx: SyncSender<Rsp>) -> Self {
        GlicolWrapper {
            engine: Engine::<32>::new(),
            rx,
            tx
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
            let msg = self.rx.recv();
            match msg {
                Ok(req) => {
                    match req {
                        Req::NextBlock => {
                            match self.update() {
                                Some(data) => { let _ = self.tx.send(Rsp::Data(data)); },
                                None => { let _ = self.tx.send(Rsp::NoData); }
                            }
                        },
                        Req::Process(code) => {
                            self.eval(code);
                            let _ = self.tx.send(Rsp::Ok);
                        },
                        Req::Stop => {
                            let _ = self.tx.send(Rsp::Ok);
                            break;
                        },
                        Req::Start | Req::Pause | Req::Resume => {
                            let _ = self.tx.send(Rsp::Ok);
                        }
                    }
                },
                _ => {
                    let _ = self.tx.send(Rsp::Error);
                    break;
                }
            }
        }
    }
}

pub struct GlicolAudioSource {
    channel: Channel,
    data_l: VecDeque<f32>,
    data_r: VecDeque<f32>,
    rx: Receiver<Rsp>,
    tx: SyncSender<Req>
}

impl GlicolAudioSource {
    pub fn new(rx: Receiver<Rsp>, tx: SyncSender<Req>) -> Self {
        GlicolAudioSource {
            channel: Channel::L,
            data_l: VecDeque::new(),
            data_r: VecDeque::new(),
            rx,
            tx
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
                    let _ = self.tx.send(Req::NextBlock);
                    let msg = self.rx.recv();
                    match msg {
                        Ok(rsp) => {
                            match rsp {
                                Rsp::Data((left, right)) => {
                                    self.data_l = left;
                                    self.data_r = right;
                                },
                                Rsp::NoData => (),
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
