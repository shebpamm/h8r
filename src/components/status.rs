use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Style},
  widgets::{Block, Borders, Paragraph},
};

use std::sync::Arc;
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
  metrics: Option<Arc<HaproxyMetrics>>,
  selected_backend: Option<String>,
  cached_metrics: Vec<Option<String>>,
}

impl Default for Status {
  fn default() -> Self {
    Self::new()
  }
}

impl Status {
  pub fn new() -> Self {
    log::debug!("Status::new: Creating new Status component");
    Self { 
      metrics: None, 
      selected_backend: None, 
      cached_metrics: vec![None; STATUS_METRICS.len()]
    }
  }

  fn update_cached_metrics(&mut self) {
    use std::time::Instant;
    let start = Instant::now();
    log::debug!("Status::update_cached_metrics: Refreshing cached metrics for backend: {:?}", self.selected_backend);
    
    self.cached_metrics = self.get_all_metrics();
    
    let elapsed = start.elapsed();
    log::debug!("Status::update_cached_metrics: Cache update took: {:?}", elapsed);
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
    log::trace!("Status::get_metric: Retrieving metric: {:?} for backend: {:?}", metric, self.selected_backend);
    
    if self.metrics.is_none() { 
      log::trace!("Status::get_metric: No metrics available");
      return None;
    }

    let metrics = self.metrics.clone()?;

    let instant = metrics.instant.clone();
    if instant.is_none() {
      log::warn!("Status::get_metric: No instant data available in metrics");
      return None;
    }
    let instant = instant?;

    // find the correct backend
    let backend = instant.data.backends.iter().find(|b| b.name == self.selected_backend);
    if backend.is_none() {
      log::trace!("Status::get_metric: Backend {:?} not found in metrics", self.selected_backend);
      return None;
    }
    let backend = backend?;

    let result = match metric {
      StatusMetrics::BackendName => backend.clone().name,
      StatusMetrics::BackendStatus => Some(backend.clone().status),
      StatusMetrics::ProxyMode => Some(backend.clone().proxy_mode),
      StatusMetrics::CurrentSessions => Some(backend.clone().sessions.to_string()),
      StatusMetrics::ActiveServers => Some(backend.clone().active_servers.to_string()),
      StatusMetrics::BackupServers => Some(backend.clone().backup_servers.to_string()),
    };

    log::trace!("Status::get_metric: Retrieved {:?} = {:?}", metric, result);
    result
  }

  fn get_all_metrics(&self) -> Vec<Option<String>> {
    log::trace!("Status::get_all_metrics: Retrieving all metrics for backend: {:?}", self.selected_backend);
    
    if self.metrics.is_none() { 
      log::trace!("Status::get_all_metrics: No metrics available");
      return vec![None; STATUS_METRICS.len()];
    }

    let metrics = match self.metrics.clone() {
      Some(m) => m,
      None => return vec![None; STATUS_METRICS.len()],
    };

    let instant = match metrics.instant.clone() {
      Some(i) => i,
      None => {
        log::warn!("Status::get_all_metrics: No instant data available in metrics");
        return vec![None; STATUS_METRICS.len()];
      }
    };

    // Find the backend ONCE instead of 6 times
    let backend = match instant.data.backends.iter().find(|b| b.name == self.selected_backend) {
      Some(b) => b,
      None => {
        log::trace!("Status::get_all_metrics: Backend {:?} not found in {} backends", 
                   self.selected_backend, instant.data.backends.len());
        return vec![None; STATUS_METRICS.len()];
      }
    };
    
    log::trace!("Status::get_all_metrics: Found backend, extracting all metrics");

    // Extract all metrics from the single backend instance
    vec![
      backend.clone().name,                                    // BackendName
      Some(backend.clone().status),                           // BackendStatus  
      Some(backend.clone().proxy_mode),                       // ProxyMode
      Some(backend.clone().sessions.to_string()),             // CurrentSessions
      Some(backend.clone().active_servers.to_string()),       // ActiveServers
      Some(backend.clone().backup_servers.to_string()),       // BackupServers
    ]
  }
}

impl Component for Status {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    log::trace!("Status::update: Received action: {:?}", action);
    
    match action {
      Action::MetricUpdate(metrics) => {
        log::debug!("Status::update: Updating metrics");
        self.metrics = Some(metrics.clone());
        self.update_cached_metrics();
        Ok(None)
      },
      Action::UseItem(backend_name) => {
        log::info!("Status::update: Switching to backend: {}", backend_name);
        self.selected_backend = Some(backend_name);
        self.update_cached_metrics();
        Ok(None)
      },
      _ => {
        log::trace!("Status::update: Ignoring action: {:?}", action);
        Ok(None)
      },
    }
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();
    log::trace!("Status::draw: Starting draw for backend: {:?}", self.selected_backend);

    let border = Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow));

    let sides = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(vec![Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
      .split(border.inner(rect));

    let lengths = vec![Constraint::Length(1); 3];

    let left = Layout::default().direction(Direction::Vertical).constraints(lengths.clone()).split(sides[0]);
    let right = Layout::default().direction(Direction::Vertical).constraints(lengths.clone()).split(sides[1]);

    f.render_widget(border.clone(), rect);
    
    // Use cached metrics directly - no backend iteration needed!
    log::trace!("Status::draw: Using cached metrics (no backend search needed)");
    
    let render_start = Instant::now();
    for i in 0..STATUS_METRICS.len() {
      let key = Span::styled(
        format!("{}: ", STATUS_METRICS[i]),
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
      );
      
      let value = Span::styled(
        self.cached_metrics[i].clone().unwrap_or_else(|| "N/A".to_string()),
        Style::default().fg(Color::White),
      );

      let text = Paragraph::new(Line::from(vec![key, value]));

      if i < 3 {
        f.render_widget(text, left[i]);
      } else {
        f.render_widget(text, right[i - 3]);
      }
    }
    
    let elapsed = start.elapsed();
    if elapsed.as_millis() > 1 {
      log::debug!("Status::draw: Draw took: {:?} (render: {:?}) - used cached metrics", 
                 elapsed, render_start.elapsed());
    } else {
      log::trace!("Status::draw: Draw took: {:?} (render: {:?}) - used cached metrics", 
                 elapsed, render_start.elapsed());
    }

    Ok(())
  }
}
