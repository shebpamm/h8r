use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;
use std::sync::Arc;

use color_eyre::eyre::Result;
use tokio::sync::mpsc::UnboundedSender;

use super::{data::HaproxyStat, metrics::HaproxyMetrics};
use crate::action::Action;

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
        },
        Err(e) => {
          println!("Error: {}", e);
        },
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
      
      // Process metrics in background thread to avoid blocking UI
      let mut metrics = HaproxyMetrics::new();
      metrics.update(stats)?;
      
      if action_tx.is_closed() {
        return Ok(());
      }
      
      // Send processed metrics instead of raw stats
      action_tx.send(Action::MetricUpdate(Arc::new(metrics)))?;

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
        std::thread::sleep(Duration::from_millis(1000));
      }
    }
  }
}
