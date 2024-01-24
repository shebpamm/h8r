use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Style},
  widgets::{Block, Borders, Paragraph},
};

use std::string::ToString;
use strum::Display;

use ratatui::{prelude::*, widgets::*};
use tui_textarea::TextArea;

use crate::{
  action::{Action, TypingMode},
  stats::{data::ResourceType, metrics::HaproxyMetrics},
  tui::Frame,
};

use super::Component;

#[derive(Debug, Clone)]
pub struct Status {
  metrics: HaproxyMetrics,
  selected_backend: Option<String>,
}

impl Default for Status {
  fn default() -> Self {
    Self::new()
  }
}

impl Status {
  pub fn new() -> Self {
    Self { metrics: HaproxyMetrics::default(), selected_backend: None }
  }
}

#[derive(Display, Debug, Clone)]
enum StatusMetrics {
  #[strum(serialize = "Backend")]
  BackendName,
  #[strum(serialize = "Status")]
  BackendStatus,
  #[strum(serialize = "Proxy Mode")]
  ProxyMode,
  #[strum(serialize = "Current Sessions")]
  CurrentSessions,
  #[strum(serialize = "Active Servers")]
  ActiveServers,
  #[strum(serialize = "Backup Servers")]
  BackupServers,
}

const STATUS_METRICS: [StatusMetrics; 6] = [
  StatusMetrics::BackendName,
  StatusMetrics::BackendStatus,
  StatusMetrics::ProxyMode,
  StatusMetrics::CurrentSessions,
  StatusMetrics::ActiveServers,
  StatusMetrics::BackupServers,
];

impl Status {
  fn get_metric(&self, metric: StatusMetrics) -> Option<String> {
    let instant = self.metrics.instant.clone()?;

    // find the correct backend
    let backend = instant.data.backends.iter().find(|b| b.name == self.selected_backend)?;

    match metric {
      StatusMetrics::BackendName => backend.clone().name,
      StatusMetrics::BackendStatus => Some(backend.clone().status),
      StatusMetrics::ProxyMode => Some(backend.clone().proxy_mode),
      StatusMetrics::CurrentSessions => Some(backend.clone().sessions.to_string()),
      StatusMetrics::ActiveServers => Some(backend.clone().active_servers.to_string()),
      StatusMetrics::BackupServers => Some(backend.clone().backup_servers.to_string()),
    }
  }
}

impl Component for Status {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::MetricUpdate(metrics) => {
        self.metrics = metrics.clone();
        Ok(None)
      },
      Action::UseItem(backend_name) => {
        self.selected_backend = Some(backend_name);
        Ok(None)
      },
      _ => Ok(None),
    }
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let border = Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow));

    let sides = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(vec![Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
      .split(border.inner(rect));

    let lengths = vec![Constraint::Length(1); 3];

    let left = Layout::default().direction(Direction::Vertical).constraints(lengths.clone()).split(sides[0]);
    let right = Layout::default().direction(Direction::Vertical).constraints(lengths.clone()).split(sides[1]);

    f.render_widget(border.clone(), rect);
    for i in 0..STATUS_METRICS.len() {
      let key = Span::styled(
        format!("{}: ", STATUS_METRICS[i]),
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
      );
      let value = Span::styled(
        self.get_metric(STATUS_METRICS[i].clone()).unwrap_or_else(|| "N/A".to_string()),
        Style::default().fg(Color::White),
      );

      let text = Paragraph::new(Line::from(vec![key, value]));

      if i < 3 {
        f.render_widget(text, left[i]);
      } else {
        f.render_widget(text, right[i - 3]);
      }
    }

    Ok(())
  }
}
