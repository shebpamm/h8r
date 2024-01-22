use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;
use color_eyre::eyre::Result;

use crate::action::Action;
use super::data::HaproxyStat;

pub struct Socket {
    stream_path: String,
}

impl Socket {
    pub fn new(stream_path: String) -> Result<Socket> {
        Ok(Socket { stream_path })
    }

    pub fn refresh(&mut self, action_tx: UnboundedSender<Action>) -> Result<()> {
        let mut stream = UnixStream::connect(&self.stream_path)?;

        loop {
            match stream.write(b"show stat\n") {
                Ok(_) => {
                    log::debug!("Querying stats");
                    break;
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }

        loop {
            let mut resp = String::new();
            stream.read_to_string(&mut resp)?;

            // Uncomment the following code if you need to write to a file synchronously
            // let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S");
            // let filename = format!("stats_{}.csv", timestamp);
            // let mut file = File::create(filename)?;
            // file.write_all(resp.as_bytes())?;

            let stats = HaproxyStat::parse_csv(&resp)?;
            action_tx.send(Action::UpdateStats(stats)).unwrap();

            break;
        }

        Ok(())
    }

    pub fn collect(&mut self, action_tx: UnboundedSender<Action>) -> Result<()> {
        // Initialize prompt mode, waiting until we can write
        loop {
            self.refresh(action_tx.clone())?;
            for _ in 0..10 {
                // check if channel is closed
                if action_tx.is_closed() {
                    return Ok(());
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}
