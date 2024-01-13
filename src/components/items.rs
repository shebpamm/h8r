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

    match self.resource {
      ResourceType::Frontend => {
        self.headers = vec!["".to_string(), "State".to_string(), "Requests".to_string()];
        if let Some(instant) = data.instant {
          for frontend in instant.data.frontends {
            rows.push(Row::new(vec![
              frontend.name.to_string(),
              frontend.status.to_string(),
              frontend.requests.to_string()
            ]));
          }
        }
      },
      ResourceType::Backend => {},
      ResourceType::Server => {},
      ResourceType::Combined => {},
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
    let selection = (self.state.selected().unwrap_or(1) + 1) % self.rows.len();

    self.state.select(Some(selection));

    log::info!("Selected: {}", selection);

    Ok(None)
  }

  fn move_up(&mut self) -> Result<Option<Action>> {
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
    let table = Table::new(
      self.rows.clone(),
      [Constraint::Length(area.width - 30), Constraint::Length(15), Constraint::Length(15)],
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
