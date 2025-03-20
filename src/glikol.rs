#![allow(unused_imports)]

use glicol::Engine;
use rodio::Source;
use std::io::BufReader;
use std::collections::VecDeque;
use std::iter::Iterator;
use std::time::Duration;
use std::sync::{Arc, Mutex};
// use std::cell::{RefCell, RefMut};

pub fn glicol_wrapper() -> Arc<Mutex<GlicolWrapper>> {
    Arc::new(Mutex::new(GlicolWrapper::new()))
}

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

pub enum Message {
    Stop,
    Start,
    Pause,
    Resume,
    Process(&'static str)
}

pub struct GlicolWrapper<'a> {
    engine: &'a mut glicol::Engine<32>,
    channel: &'a Channel,
    data_l: &'a VecDeque<f32>,
    data_r: &'a VecDeque<f32>
}

impl<'a> Clone for GlicolWrapper<'a> {
    fn clone(&self) -> Self {
        GlicolWrapper {
            engine: &self.engine.borrow_mut(),
            channel: &self.channel.clone(),
            data_l: &self.data_l.clone(),
            data_r: &self.data_r.clone()
        }
    }
}

impl GlicolWrapper {
    pub fn new() -> Self {
        GlicolWrapper {
            engine: Engine::<32>::new(),
            channel: Channel::L,
            data_l: VecDeque::new(),
            data_r: VecDeque::new()
        }
    }

    pub fn eval(&mut self, code: &str) {
        self.engine.update_with_code(code);
    }

    fn update(&mut self) -> bool {
        let (bufs, _mystery_data) = self.engine.next_block(vec![]);
        if bufs[0].is_empty() || bufs[1].is_empty() {
            false
        } else {
            self.data_l = VecDeque::from(bufs[0].to_vec());
            self.data_r = VecDeque::from(bufs[1].to_vec());
            true
        }
    }
}

impl Iterator for GlicolWrapper {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let result = {
            let result_ = match self.channel {
                Channel::L => self.data_l.pop_front(),
                Channel::R => self.data_r.pop_front()
            };
            if result_ == None {
                if self.update() {
                    match self.channel {
                        Channel::L => self.data_l.pop_front(),
                        Channel::R => self.data_r.pop_front()
                    }
                } else {
                    None
                }
            } else {
                result_
            }
        };
        self.channel = match self.channel {
            Channel::L => Channel::R,
            Channel::R => Channel::L
        };
        result
    }
}

impl Source for GlicolWrapper {
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
