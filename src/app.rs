use color_eyre::eyre::{eyre, Result};
use std::sync::Arc;
use crossterm::event::KeyEvent;
use ratatui::{
  layout::{Constraint, Direction, Layout},
  prelude::Rect,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
  action::{Action, TypingMode, MovementMode},
  components::{fps::FpsCounter, items::Items, Component},
  config::Config,
  layouts::info::InfoLayout,
  layouts::home::HomeLayout,
  mode::Mode,
  stats::{data::HaproxyStat, metrics::HaproxyMetrics, socket::Socket},
  tui,
};

use tokio::task;

pub struct App {
  pub config: Config,
  pub tick_rate: f64,
  pub frame_rate: f64,
  pub home: HomeLayout,
  pub graph: InfoLayout,
  pub should_quit: bool,
  pub should_suspend: bool,
  pub mode: Mode,
  pub last_tick_key_events: Vec<KeyEvent>,
  pub haproxy_metrics: Arc<HaproxyMetrics>,
  pub typing_mode: TypingMode,
}

impl App {
  pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
    let config = Config::new()?;
    let mode = Mode::Home;
    let home = HomeLayout::new();
    let graph = InfoLayout::new();

    Ok(Self {
      tick_rate,
      frame_rate,
      home,
      graph,
      should_quit: false,
      should_suspend: false,
      config,
      mode,
      last_tick_key_events: Vec::new(),
      haproxy_metrics: Arc::new(HaproxyMetrics::new()),
      typing_mode: TypingMode::Navigation,
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    let mut tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
    // tui.mouse(true);
    tui.enter()?;

    let config = self.config.clone();

    let layout = self.get_layout();

    layout.register_action_handler(action_tx.clone())?;
    layout.register_config_handler(config)?;
    layout.init(Rect::new(0, 0, tui.size()?.width, tui.size()?.height))?;

    let mut socket = Socket::new(self.config.paths.socket.to_string())?;
    let socket_tx = action_tx.clone();

    task::spawn_blocking(move || -> Result<()> {
      socket.collect(socket_tx)?;
      Ok(())
    });

    'main: loop {
      use std::time::Instant;
      let now = Instant::now();
      if let Some(mut e) = tui.next().await {

        match e {
          tui::Event::Quit => action_tx.send(Action::Quit)?,
          tui::Event::Tick => action_tx.send(Action::Tick)?,
          tui::Event::Render => action_tx.send(Action::Render)?,
          tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
          tui::Event::Key(key) => {
            if self.typing_mode == TypingMode::Navigation {
              if let Some(keymap) = self.config.keybindings.get(&self.mode) {
                if let Some(action) = keymap.get(&vec![key]) {
                  log::info!("Got action: {action:?}");
                  action_tx.send(action.clone())?;
                } else {
                  // If the key was not handled as a single key action,
                  // then consider it for multi-key combinations.
                  self.last_tick_key_events.push(key);

                  // Check for multi-key combinations
                  if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    log::info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                  }
                }
              };
            }
            e = tui::Event::ModeKey(self.typing_mode.clone(), key);
          },
          _ => {},
        }
        if let Some(action) = self.get_layout().handle_events(Some(e.clone()))? {
          action_tx.send(action)?;
        }
      }

      while let Ok(action) = action_rx.try_recv() {
        if action != Action::Tick && action != Action::Render {
           let name = action.to_string();
          log::trace!("{name}");
        }

        match action.clone() {
          Action::MetricUpdate(metrics) => {
            self.haproxy_metrics = metrics;
          },
          Action::MoveUp => {
            match self.typing_mode {
              TypingMode::Navigation => self.get_layout().move_up(MovementMode::Single)?,
              _ => None,
            };
          },
          Action::MoveDown => {
            match self.typing_mode {
              TypingMode::Navigation => self.get_layout().move_down(MovementMode::Single)?,
              _ => None,
            };
          },
          Action::MoveSectionUp => {
            match self.typing_mode {
              TypingMode::Navigation => self.get_layout().move_up(MovementMode::Section)?,
              _ => None,
            };
          },
          Action::MoveSectionDown => {
            match self.typing_mode {
              TypingMode::Navigation => self.get_layout().move_down(MovementMode::Section)?,
              _ => None,
            };
          },
          
          Action::Tick => {
            self.last_tick_key_events.drain(..);
          },
          Action::Quit => self.should_quit = true,
          Action::Suspend => self.should_suspend = true,
          Action::Resume => self.should_suspend = false,
          Action::Resize(w, h) => {
            tui.resize(Rect::new(0, 0, w, h))?;
            tui.draw(|f| {
              let r = self.get_layout().draw(f, f.area());
              if let Err(e) = r {
                action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
              }
            })?;
          },
          Action::Render => {
            tui.draw(|f| {
              let r = self.get_layout().draw(f, f.area());
              if let Err(e) = r {
                action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
              }
            })?;
          },
          Action::TypingMode(typing_mode) => {
            self.typing_mode = typing_mode;
          },
          Action::SwitchMode(mode) => {
            self.mode = mode;

            let config = self.config.clone();

            self.get_layout().register_action_handler(action_tx.clone())?;
            self.get_layout().register_config_handler(config)?;
            self.get_layout().init(Rect::new(0, 0, tui.size()?.width, tui.size()?.height))?;
          },
          _ => {},
        }
        if let Some(action) = self.get_layout().update(action.clone())? {
          action_tx.send(action)?
        };
      }
      if self.should_suspend {
        tui.suspend()?;
        action_tx.send(Action::Resume)?;
        tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
        // tui.mouse(true);
        tui.enter()?;
      } else if self.should_quit {
        tui.stop()?;
        break;
      }

      let elapsed = now.elapsed();
      log::debug!("Main loop matching took: {:?}", elapsed);
    }
    tui.exit()?;
    Ok(())
  }

  fn get_layout(&mut self) -> &mut dyn Component {
    match self.mode {
      Mode::Home => &mut self.home,
      Mode::Info => &mut self.graph
    }
  }
}
