use crossterm::event::{self, Event};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct EventHandler {
    rx: mpsc::Receiver<Option<Event>>,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let tick_duration = Duration::from_millis(tick_rate_ms);

            loop {
                if event::poll(tick_duration).unwrap_or(false) {
                    if let Ok(event) = event::read() {
                        if tx.send(Some(event)).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Self { rx }
    }

    pub fn next(&self) -> io::Result<Option<Event>> {
        Ok(self.rx.try_recv().ok().flatten())
    }
}
