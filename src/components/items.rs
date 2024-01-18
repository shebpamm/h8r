use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
  action::Action,
  config::{Config, KeyBindings},
  stats::{
    data::{HaproxyStat, ResourceType},
    metrics::HaproxyMetrics,
  },
};

pub struct Items<'a> {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  state: TableState,
  metrics: HaproxyMetrics,
  headers: Vec<String>,
  rows: Vec<Row<'a>>,
  resource: ResourceType,
}

impl Items<'_> {
  pub fn new() -> Self {
    Self {
      command_tx: None,
      config: Config::default(),
      state: TableState::default(),
      headers: Vec::default(),
      metrics: HaproxyMetrics::default(),
      rows: Vec::default(),
      resource: ResourceType::Combined,
    }
  }

  fn update_rows(&mut self, data: HaproxyMetrics) {
    let mut rows = Vec::new();

    match (self.resource, data.instant) {
      (ResourceType::Frontend, Some(instant)) => {
        self.headers = vec!["".to_string(), "State".to_string(), "Requests".to_string()];
        for frontend in instant.data.frontends {
          rows.push(Row::new(vec![
            frontend.name.unwrap_or("".to_string()),
            frontend.status.to_string(),
            frontend.requests.to_string(),
          ]));
        }
      },
      (ResourceType::Backend, Some(instant)) => {
        self.headers = vec!["".to_string(), "State".to_string(), "Requests".to_string()];
        for backend in instant.data.backends {
          rows.push(Row::new(vec![backend.name.unwrap_or("".to_string()), backend.status.to_string(), backend.requests.to_string()]));
        }
      },
      (ResourceType::Server, Some(instant)) => {
        self.headers = vec!["".to_string(), "Backend".to_string(), "State".to_string(), "Requests".to_string()];
        for server in instant.data.servers {
          rows.push(Row::new(vec![server.name.unwrap_or("".to_string()), server.backend_name.unwrap_or("".to_string()), server.status.to_string(), server.requests.to_string()]));
        }
      },
      (ResourceType::Combined, Some(instant)) => {
        self.headers = vec!["".to_string(), "Type".to_string(), "State".to_string(), "Requests".to_string()];
        for backend in instant.data.backends {
          rows.push(Row::new(vec![
            format!("{}", backend.name.unwrap_or("".to_string())).bold(),
            "Backend".to_string().bold(),
            backend.status.to_string().bold(),
            backend.requests.to_string().bold(),
          ]));
          for server in backend.servers {
            rows.push(Row::new(vec![
              format!("└ {}", server.name.unwrap_or("".to_string())),
              "Server".to_string(),
              server.status.to_string(),
              server.requests.to_string(),
            ]));
          }
        }
      },
      (_, None) => {
        self.headers = vec!["".to_string()];
        rows.push(Row::new(vec!["No data available".to_string().red()]));
      },
    }

    self.rows = rows;
  }
}

impl Component for Items<'_> {
  fn init(&mut self, size: Rect) -> Result<()> {
    self.rows = Vec::new();
    self.state.select(Some(0));
    Ok(())
  }

  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn move_down(&mut self) -> Result<Option<Action>> {
    if self.rows.is_empty() {
      return Ok(None);
    }

    let selection = (self.state.selected().unwrap_or(1) + 1) % self.rows.len();

    self.state.select(Some(selection));

    log::info!("Selected: {}", selection);

    Ok(None)
  }

  fn move_up(&mut self) -> Result<Option<Action>> {
    if self.rows.is_empty() {
      return Ok(None);
    }

    let selection = if self.state.selected().unwrap_or(1) == 0 {
      self.rows.len() - 1
    } else {
      self.state.selected().unwrap_or(1) - 1
    };

    self.state.select(Some(selection));

    log::info!("Selected: {}", selection);

    Ok(None)
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::MetricUpdate(metrics) => {
        self.metrics = metrics.clone();
        self.update_rows(metrics);
        Ok(None)
      },
      Action::SelectResource(resource) => {
        self.resource = resource;
        self.state.select(Some(0));
        self.update_rows(self.metrics.clone());
        Ok(None)
      },
      _ => Ok(None),
    }
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    let mut lengths = Vec::new();
    for header in &self.headers {
      lengths.push(Constraint::Length(15));
    }
    
    if lengths.is_empty() {
      lengths.push(Constraint::Length(15));
    }

    lengths[0] = Constraint::Length(area.width - (lengths.len() as u16 - 1) * 15);

    let table = Table::new(
      self.rows.clone(),
      lengths
    )
    .header(Row::new(self.headers.clone()).bold())
    .highlight_style(Style::new().light_yellow());

    let border = Block::new()
      .title(self.resource.to_string())
      .borders(Borders::ALL)
      .border_style(Style::default().fg(Color::White));

    f.render_widget(border, area);
    f.render_stateful_widget(table, area, &mut self.state);
    Ok(())
  }
}
