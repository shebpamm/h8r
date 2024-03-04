use std::collections::HashSet;
use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
  action::Action,
  config::{Config, KeyBindings},
  stats::{
    data::{HaproxyStat, ResourceType},
    metrics::{HaproxyBackend, HaproxyMetrics},
  }, mode::Mode,
};

pub struct Items<'a> {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  state: TableState,
  metrics: HaproxyMetrics,
  headers: Vec<String>,
  rows: Vec<Row<'a>>,
  row_lookup: HashMap<Row<'a>, HaproxyBackend>,
  resource: ResourceType,
  filter: Option<String>,
  sticky_backends: HashSet<String>,
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
      row_lookup: HashMap::new(),
      resource: ResourceType::default(),
      filter: None,
      sticky_backends: HashSet::new(),
    }
  }

  fn update_rows(&mut self, data: HaproxyMetrics) {
    use std::time::Instant;
    let now = Instant::now();

    let mut rows = Vec::new();
    let mut row_lookup: HashMap<Row, HaproxyBackend> = HashMap::new();

    fn format_backend_name(name: String, is_stickied: bool) -> Span<'static> {
      if is_stickied {
        format!("{} (sticky)", name).cyan().bold()
      } else {
        name.bold()
      }
    }

    match (self.resource, data.instant) {
      (ResourceType::Frontend, Some(instant)) => {
        self.headers = vec!["".to_string(), "State".to_string(), "Requests".to_string()];
        for frontend in instant.data.frontends {
          let name = frontend.name.unwrap_or("".to_string());

          if let Some(filter) = &self.filter {
            if !name.contains(filter) {
              continue;
            }
          }

          let row = Row::new(vec![name, frontend.status.to_string(), frontend.requests.to_string()]);
          rows.push(row);
        }
      },
      (ResourceType::Backend, Some(instant)) => {
        self.headers = vec!["".to_string(), "State".to_string(), "Requests".to_string()];
        for backend in instant.data.backends {
          let name = backend.clone().name.unwrap_or("".to_string());

          let is_stickied = self.sticky_backends.contains(&name);

          if let Some(filter) = &self.filter {
            if !name.contains(filter) && !is_stickied {
              continue;
            }
          }

          let row = Row::new(vec![format_backend_name(name, is_stickied), backend.status.to_string().white(), backend.requests.to_string().white()]);
          row_lookup.insert(row.clone(), backend);
          rows.push(row);
        }
      },
      (ResourceType::Server, Some(instant)) => {
        self.headers = vec!["".to_string(), "Backend".to_string(), "State".to_string(), "Requests".to_string()];
        for server in instant.data.servers {
          let name = server.name.unwrap_or("".to_string());

          if let Some(filter) = &self.filter {
            if !name.contains(filter) {
              continue;
            }
          }

          let row = Row::new(vec![name, server.status.to_string(), server.requests.to_string()]);
          rows.push(row);
        }
      },
      (ResourceType::Combined, Some(instant)) => {
        self.headers = vec!["".to_string(), "Type".to_string(), "State".to_string(), "Code".to_string(), "Requests".to_string()];
        for backend in instant.data.backends {
          let backend_name = backend.clone().name.unwrap_or("".to_string());

          let is_stickied = self.sticky_backends.contains(&backend_name);

          if let Some(filter) = &self.filter {
            if !backend_name.contains(filter) && !is_stickied {
              continue;
            }
          }

          let backend_row = Row::new(vec![
            format_backend_name(backend_name, is_stickied),
            "Backend".to_string().bold(),
            backend.status.to_string().bold(),
            "".to_string().bold(),
            backend.requests.to_string().bold(),
          ]);
          row_lookup.insert(backend_row.clone(), backend.clone());
          rows.push(backend_row);
          for server in backend.servers {
            rows.push(Row::new(vec![
              format!("└ {}", server.name.unwrap_or("".to_string())),
              "Server".to_string(),
              server.status.to_string(),
              server.status_code.to_string(),
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

    self.row_lookup = row_lookup;
    self.rows = rows;

    let elapsed = now.elapsed();
    log::debug!("Update rows took: {:?}", elapsed);
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
      Action::Filter(filter_string) => {
        self.filter = Some(filter_string);
        self.state.select(Some(0));
        self.update_rows(self.metrics.clone());
        Ok(None)
      },
      Action::Sticky => {
        match self.state.selected() {
            Some(selection) => {
                if let Some(row) = self.rows.get(selection) {
                    if let Some(data) = &self.row_lookup.get(row) {
                        if self.sticky_backends.contains(&data.name.clone().unwrap_or("".to_string())) {
                            self.sticky_backends.remove(&data.name.clone().unwrap_or("".to_string()));
                        } else {
                            self.sticky_backends.insert(data.name.clone().unwrap_or("".to_string()));
                        }
                    }
                }
            },
            None => {
                log::info!("No selection");
            },
        }
         
        Ok(None)
      },
      Action::SelectItem => {
        log::info!("Selected: {:?}", self.state.selected());
        if let Some(selection) = self.state.selected() {
          if let Some(row) = self.rows.get(selection) {
            if let Some(data) = &self.row_lookup.get(row) {
              if let Some(tx) = &self.command_tx {
                let name = data.name.clone().unwrap_or("".to_string());

                tx.send(Action::SwitchMode(Mode::Graph))?;
                tx.send(Action::UseItem(name))?;
              }
            }
          }
        }
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

    let table = Table::new(self.rows.clone(), lengths)
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
