use crate::action::Action;
use crate::components::Component;
use crate::components::Rect;
use crate::config::Config;
use crate::tui::Frame;
use color_eyre::eyre::{Error, Result};
use ratatui::widgets::Paragraph;
use std::io::{Read, Write};
use ansi_to_tui::IntoText;

#[derive(Debug, Default)]
pub struct LogView {
  config: Config,
  log_content: Option<Vec<String>>,
  log_parse_error: Option<Box<Error>>,
  selected_backend: Option<String>,
}

impl LogView {
  fn read_config(&mut self) -> Result<()> {
    let path = &self.config.config._log_path;
    let content = std::fs::read_to_string(path)?;

    self.log_content = Some(content.lines().map(|line| line.to_string()).collect());

    Ok(())
  }
}

impl Component for LogView {
  fn init(&mut self, _rect: Rect) -> Result<()> {
    match self.read_config() {
      Ok(_) => (),
      Err(e) => self.log_parse_error = Some(Box::new(e)),
    }

    Ok(())
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let content = match self.log_content {
      Some(ref logs) => {
        // find the backend section for current backend
        let mut backend = None;

        let mut lines = vec![];

        for i in 0..logs.len() {
          let line = &logs[i];
          if let Some(ref selected_backend) = self.selected_backend {
            if line.contains(selected_backend) {
              backend = Some(selected_backend);
              lines.push(line.clone());
              continue;
            }
          }

          if let Some(ref backend) = backend {
            if line.starts_with("backend") {
              break;
            }
          }
        }

        lines.join("\n").into_text()?
      },
      None => match self.log_parse_error {
        Some(ref e) => format!("Error: {}", e).into_text()?,
        None => "Loading...".into_text()?,
      },
    };
    let text = Paragraph::new(content);
    f.render_widget(text, rect);

    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::UseItem(backend_name) => {
        self.selected_backend = Some(backend_name);
        Ok(None)
      },
      _ => Ok(None),
    }
  }
}
