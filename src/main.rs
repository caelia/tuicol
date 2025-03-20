#![allow(unused_imports)]
#![allow(dead_code)]

mod common;
mod glikol;

use glikol::{GlicolWrapper, GlicolAudioSource};
use common::CtrlReq;
use cpal::{HostId, host_from_id};
use cpal::traits::HostTrait;
use rodio::OutputStream;
use std::time::Duration;
use std::thread;
use std::sync::mpsc::{channel, sync_channel};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use std::io;
use tui_textarea::{Input, Key, TextArea};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("GLICOL")
    );
    
    let (data_req_tx, data_req_rx) = sync_channel(1);
    let (data_rsp_tx, data_rsp_rx) = sync_channel(1);
    let (ctrl_req_tx, ctrl_req_rx) = channel();
    let (ctrl_rsp_tx, ctrl_rsp_rx) = channel();
        
    let (tx, _rx) = (ctrl_req_tx, ctrl_rsp_rx);
    let mut wrapper = GlicolWrapper::new(data_rsp_tx, data_req_rx, ctrl_rsp_tx, ctrl_req_rx);
    let src = GlicolAudioSource::new(data_req_tx, data_rsp_rx);

    let host = host_from_id(HostId::Jack).unwrap();
    let device = host.default_output_device().unwrap();
    let (_stream, handle) = OutputStream::try_from_device(&device).unwrap();

    let _ = tx.send(CtrlReq::Pause);
    thread::spawn(move || {
        wrapper.run();
    });

    thread::spawn(move || {
        let _ = handle.play_raw(src);
    });

    textarea.insert_str(r#"
        // Welcome to TUICol, the TUI implementation of the GLICOL music programming environment!
        // This is just a simple demo: you can enter GLICOL code in this window and hear the
        // results. You can use the following keystroke commands:
        // * Ctrl+Enter : process the contents of the window
        // * Ctrl+. : stop playing sound
        // * Ctrl+Q : quit
    "#);

    loop {
        term.draw(|f| {
            f.render_widget(&textarea, f.area());
        })?;
        match crossterm::event::read()?.into() {
            Input {
                key: Key::Enter,
                ctrl: true,
                .. 
            } => {
                let code = textarea.lines().join("\n");
                let _ = tx.send(CtrlReq::Process(code));
            },
            Input {
                key: Key::Char('q' | 'Q'),
                ctrl: true,
                ..
            } => {
                let _ = tx.send(CtrlReq::Stop);
                break;
            },
            Input {
                key: Key::Char('.'),
                ctrl: true,
                ..
            } => {
                let _ = tx.send(CtrlReq::Pause);
            },
            input => {
                let _ = textarea.input(input);
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    Ok(())
}
