use std::collections::VecDeque;

pub enum Channel {
    L,
    R
}

#[derive(Debug)]
pub enum State {
    Stopped,
    Paused,
    Running
}

#[derive(Debug)]
pub enum DataReq {
    NextBlock,
}

#[derive(Debug)]
pub enum CtrlReq {
    Process(String),
    Stop,
    Pause,
    Resume
}

#[derive(Debug)]
pub enum DataRsp {
    Data((VecDeque<f32>, VecDeque<f32>)),
    NoData,
    Ok
}

#[derive(Debug)]
pub enum CtrlRsp {
    Ok,
    Error    
}

