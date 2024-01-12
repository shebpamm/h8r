use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{prelude::Rect, layout::{Layout, Direction, Constraint}};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
  action::Action,
  components::{items::Items, fps::FpsCounter, Component},
  config::Config,
  mode::Mode,
  tui,
  layout::HomeLayout,
  stats::{socket::Socket,data::HaproxyStat},
};

pub struct App {
  pub config: Config,
  pub tick_rate: f64,
  pub frame_rate: f64,
  pub layout: Box<dyn Component>,
  pub should_quit: bool,
  pub should_suspend: bool,
  pub mode: Mode,
  pub last_tick_key_events: Vec<KeyEvent>,
  pub haproxy_stats: Vec<HaproxyStat>,
}

impl App {
  pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
    let config = Config::new()?;
    let mode = Mode::Home;
    let layout = match mode {
        Mode::Home => Box::new(HomeLayout::new()),
    };

    Ok(Self {
      tick_rate,
      frame_rate,
      layout,
      should_quit: false,
      should_suspend: false,
      config,
      mode,
      last_tick_key_events: Vec::new(),
      haproxy_stats: Vec::new(),
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    let mut tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
    // tui.mouse(true);
    tui.enter()?;

      self.layout.register_action_handler(action_tx.clone())?;
      self.layout.register_config_handler(self.config.clone())?;
      self.layout.init(tui.size()?)?;

    let mut socket = Socket::new(self.config.config._socket_path.clone()).await?;
    let socket_tx = action_tx.clone();

    tokio::spawn(async move {
        socket.collect(socket_tx).await.unwrap();
    });

    loop {
      if let Some(e) = tui.next().await {
        match e {
          tui::Event::Quit => action_tx.send(Action::Quit)?,
          tui::Event::Tick => action_tx.send(Action::Tick)?,
          tui::Event::Render => action_tx.send(Action::Render)?,
          tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
          tui::Event::Key(key) => {
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
          },
          _ => {},
        }
          if let Some(action) = self.layout.handle_events(Some(e.clone()))? {
            action_tx.send(action)?;
        }
      }

      while let Ok(action) = action_rx.try_recv() {
        if action != Action::Tick && action != Action::Render {
          log::debug!("{action:?}");
        }
        match action.clone() {
          Action::MoveUp => {
            self.layout.move_up()?;
          },
          Action::MoveDown => {
            self.layout.move_down()?;
          },
          Action::UpdateStats(stats) => {
            log::info!("Updating stats: {stats:?}");
            self.haproxy_stats = stats;
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
                let r = self.layout.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
            })?;
          },
          Action::Render => {
            tui.draw(|f| {
                let r = self.layout.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
            })?;
          },
          _ => {},
        }
          if let Some(action) = self.layout.update(action.clone())? {
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
    }
    tui.exit()?;
    Ok(())
  }
}
