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
  pub stream: UnixStream,
}

impl Socket {
  pub async fn new(stream_path: String) -> Result<Socket> {
    let stream = UnixStream::connect(&stream_path).await?;
    Ok(Socket { stream })
  }

  pub async fn collect(&mut self, action_tx: UnboundedSender<Action>) -> Result<()> {
    // Initialize prompt mode, waiting until we can write
    loop {
      let ready = self.stream.ready(Interest::READABLE | Interest::WRITABLE).await?;

      if ready.is_writable() {
        match self.stream.try_write(b"prompt\n") {
          Ok(_) => {
            log::debug!("Prompt mode enabled");
            loop {
              let mut buf = [0; 1024];
              match self.stream.try_read(&mut buf) {
                Ok(_) => {
                  log::debug!("Prompt mode enabled");
                  log::info!("trash data: {}", String::from_utf8_lossy(&buf));
                  break;
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                  continue;
                },
                Err(e) => {
                  println!("Error: {}", e);
                },
              }
            }
            break;
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

    let mut should_query = true;
    let mut resp = String::new();
    loop {
      let ready = self.stream.ready(Interest::READABLE | Interest::WRITABLE).await?;

      if ready.is_writable() && should_query {
        match self.stream.try_write(b"show stat\n") {
          Ok(_) => {
            log::debug!("Querying stats");
            should_query = false;
          },
          Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            continue;
          },
          Err(e) => {
            println!("Error: {}", e);
          },
        }
      }

      if ready.is_readable() {
        let mut buf = [0; 1024];
        match self.stream.try_read(&mut buf) {
          Ok(buf_size) => {
            resp.push_str(&String::from_utf8_lossy(&buf));
            log::info!("buf contents: {}", &String::from_utf8_lossy(&buf));

            if buf_size != 1024 {
              should_query = true;
              let res = resp
                .clone()
                .split("\n")
                .filter(|line| !line.starts_with(">"))
                .collect::<Vec<&str>>()
                .join("\n");


              let stats = HaproxyStat::parse_csv(&res)?;
              action_tx.send(Action::UpdateStats(stats)).unwrap();
              resp.clear();

              tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
              continue;
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
  }
}
