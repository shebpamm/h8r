#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod action;
pub mod app;
pub mod cli;
pub mod components;
pub mod config;
pub mod mode;
pub mod tui;
pub mod utils;
pub mod layouts;
pub mod stats;

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::Result;

use crate::{
  app::App,
  utils::{initialize_logging, initialize_panic_handler, version},
};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

async fn tokio_main() -> Result<()> {
  initialize_logging()?;

  initialize_panic_handler()?;

  let args = Cli::parse();
  let mut app = App::new(args.tick_rate, args.frame_rate)?;
  app.run().await?;

  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
#[cfg(feature = "dhat-heap")]
let _profiler = dhat::Profiler::new_heap();

  match tokio_main().await {
    Ok(_) => Ok(()),
    Err(e) => {
      match  e.downcast_ref::<std::io::Error>() {
        Some(os_err) if os_err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("{} error:", env!("CARGO_PKG_NAME"));
            eprintln!("Try running with sudo or as the root user.");
            Err(e)
        }
        _ => {
          eprintln!("{} error:", env!("CARGO_PKG_NAME"));
          Err(e)
        }
      }
    }
  }
}
