use std::io::{Read, Write};
use tokio::{
  io::{self, Interest},
  net::{UnixListener, UnixStream},
  sync::mpsc::UnboundedSender,
};

use color_eyre::eyre::Result;

use crate::action::Action;

use super::data::HaproxyStat;

pub struct Socket {
    stream_path: String,
}

impl Socket {
  pub async fn new(stream_path: String) -> Result<Socket> {
    Ok(Socket { stream_path })
  }

  pub async fn refresh(&mut self, action_tx: UnboundedSender<Action>) -> Result<()> {
    let stream = UnixStream::connect(&self.stream_path).await?;

    let mut resp = String::new();
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
        let mut buf = [0; 1024];
        match stream.try_read(&mut buf) {
          Ok(buf_size) => {
            resp.push_str(&String::from_utf8_lossy(&buf));

            if buf_size != 1024 {
              let res = resp
                .clone()
                .split("\n")
                .filter(|line| !line.starts_with(">"))
                .collect::<Vec<&str>>()
                .join("\n");


              let stats = HaproxyStat::parse_csv(&res)?;
              action_tx.send(Action::UpdateStats(stats)).unwrap();
              break;
            }
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
