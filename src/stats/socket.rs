use std::io::{Read, Write};
use tokio::{
  io::{self, Interest, AsyncReadExt},
  net::{UnixListener, UnixStream},
  sync::mpsc::UnboundedSender,
};

use color_eyre::eyre::Result;

use crate::action::Action;

use super::data::HaproxyStat;
use std::fs::File;
use chrono::Utc;

pub struct Socket {
  stream_path: String,
}

impl Socket {
  pub async fn new(stream_path: String) -> Result<Socket> {
    Ok(Socket { stream_path })
  }

  pub async fn refresh(&mut self, action_tx: UnboundedSender<Action>) -> Result<()> {
    let mut stream = UnixStream::connect(&self.stream_path).await?;

    loop {
      let ready = stream.ready(Interest::READABLE | Interest::WRITABLE).await?;

      if ready.is_writable() {
        match stream.try_write(b"show stat\n") {
          Ok(_) => {
            log::debug!("Querying stats");
            break
          },
          Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            continue;
          },
          Err(e) => {
            println!("Error: {}", e);
          },
        }
      }
    }
    loop {
      let ready = stream.ready(Interest::READABLE | Interest::WRITABLE).await?;

      if ready.is_readable() {
              let mut resp = String::new();
              stream.read_to_string(&mut resp).await?;

              // let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S");
              // let filename = format!("stats_{}.csv", timestamp);
              // let mut file = File::create(filename)?;
              // file.write_all(resp.as_bytes())?;

              let stats = HaproxyStat::parse_csv(&resp)?;
              action_tx.send(Action::UpdateStats(stats)).unwrap();

              break;
      }
    }
    Ok(())
  }
  pub async fn collect(&mut self, action_tx: UnboundedSender<Action>) -> Result<()> {
    // Initialize prompt mode, waiting until we can write
    loop {
        self.refresh(action_tx.clone()).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
}
}
