use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::stats::data::StatusType;
use crate::stats::metrics::HaproxyFrontendStatus;
use crate::{
  action::{Action, MovementMode},
  config::{Config, KeyBindings},
  mode::Mode,
  stats::{
    data::{HaproxyStat, ResourceType},
    metrics::{HaproxyBackend, HaproxyMetrics},
  },
};

enum LookupType {
  Backend(HaproxyBackend),
  Server(HaproxyBackend),
}

pub struct Items<'a> {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  state: TableState,
  metrics: Option<Arc<HaproxyMetrics>>,
  headers: Vec<String>,
  rows: Vec<Row<'a>>,
  row_lookup: HashMap<Row<'a>, LookupType>,
  resource: ResourceType,
  status_filter: StatusType,
  filter: Option<String>,
  sticky_backends: HashSet<String>,
  table: Table<'a>,
  area: Option<Rect>,
}

impl Items<'_> {
  pub fn new() -> Self {
    Self {
      command_tx: None,
      config: Config::default(),
      state: TableState::default(),
      headers: Vec::default(),
      metrics: None,
      rows: Vec::default(),
      row_lookup: HashMap::new(),
      resource: ResourceType::default(),
      status_filter: StatusType::default(),
      filter: None,
      table: Table::default(),
      sticky_backends: HashSet::new(),
      area: None,
    }
  }

  fn update_rows(&mut self) {
    let data = self.metrics.clone();

    if data.is_none() {
      return;
    }

    let data = data.unwrap();

    use std::time::Instant;
    let now = Instant::now();

    let mut rows = Vec::new();
    let mut row_lookup: HashMap<Row, LookupType> = HashMap::new();

    fn format_backend_name(name: String, is_stickied: bool) -> Span<'static> {
      if is_stickied {
        format!("{} (sticky)", name).cyan().bold()
      } else {
        name.bold()
      }
    }

    fn parse_backend_status(status: &str) -> StatusType {
      match status {
        "UP" => StatusType::Healthy,
        "DOWN" => StatusType::Failing,
        _ => StatusType::Failing,
      }
    }

    fn parse_frontend_status(status: &HaproxyFrontendStatus) -> StatusType {
      match status {
        HaproxyFrontendStatus::Open => StatusType::Healthy,
        HaproxyFrontendStatus::Closed => StatusType::Failing,
      }
    }

    match (self.resource, &data.instant) {
      (ResourceType::Frontend, Some(instant)) => {
        self.headers = vec!["".to_string(), "State".to_string(), "Requests".to_string()];
        for frontend in &instant.data.frontends {
          let name = &frontend.name.clone().unwrap_or("".to_string());

          if let Some(filter) = &self.filter {
            if !name.contains(filter) {
              continue;
            }
          }

          if parse_frontend_status(&frontend.status) != self.status_filter {
            continue;
          }

          let row = Row::new(vec![name.clone(), frontend.status.to_string(), frontend.requests.to_string()]);
          rows.push(row);
        }
      },
      (ResourceType::Backend, Some(instant)) => {
        self.headers = vec!["".to_string(), "State".to_string(), "Requests".to_string()];
        for backend in instant.data.backends.clone() {
          let name = backend.clone().name.unwrap_or("".to_string());

          let is_stickied = self.sticky_backends.contains(&name);

          if let Some(filter) = &self.filter {
            if !name.contains(filter) && !is_stickied {
              continue;
            }
          }

          if self.status_filter != StatusType::All && parse_backend_status(&backend.status) != self.status_filter {
            continue;
          }

          let row = Row::new(vec![
            format_backend_name(name, is_stickied),
            backend.status.to_string().white(),
            backend.requests.to_string().white(),
          ]);
          row_lookup.insert(row.clone(), LookupType::Backend(backend));
          rows.push(row);
        }
      },
      (ResourceType::Server, Some(instant)) => {
        self.headers = vec!["".to_string(), "Backend".to_string(), "State".to_string(), "Requests".to_string()];
        for server in instant.data.servers.clone() {
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
        self.headers =
          vec!["".to_string(), "Type".to_string(), "State".to_string(), "Code".to_string(), "Requests".to_string()];
        for backend in instant.data.backends.clone() {
          let backend_name = backend.clone().name.unwrap_or("".to_string());

          let is_stickied = self.sticky_backends.contains(&backend_name);

          if let Some(filter) = &self.filter {
            if !backend_name.contains(filter) && !is_stickied {
              continue;
            }
          }

          if self.status_filter != StatusType::All && parse_backend_status(&backend.status) != self.status_filter {
            continue;
          }

          let backend_row = Row::new(vec![
            format_backend_name(backend_name, is_stickied),
            "Backend".to_string().bold(),
            backend.status.to_string().bold(),
            "".to_string().bold(),
            backend.requests.to_string().bold(),
          ]);
          row_lookup.insert(backend_row.clone(), LookupType::Backend(backend.clone()));
          rows.push(backend_row);
          for server in &backend.servers {
            let server_row = Row::new(vec![
              format!("└ {}", server.name.clone().unwrap_or("".to_string())),
              "Server".to_string(),
              server.status.to_string(),
              server.status_code.to_string(),
              server.requests.to_string(),
            ]);
            row_lookup.insert(server_row.clone(), LookupType::Server(backend.clone()));
            rows.push(server_row);
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

    self.create_table();

    let elapsed = now.elapsed();
    log::trace!("Update rows took: {:?}", elapsed);
  }

  fn find_next_row(&self) -> usize {
    (self.state.selected().unwrap_or(1) + 1) % self.rows.len()
  }

  fn find_next_section(&mut self) -> usize {
    let mut next = self.find_next_row();
    while next < self.rows.len() {
      if let Some(row) = self.rows.get(next) {
        if let Some(data) = &self.row_lookup.get(row) {
          if let LookupType::Backend(_) = data {
            break;
          }
        }
      }
      self.state.select(Some(next));
      next = self.find_next_row();
    }
    next
  }

  fn find_prev_row(&self) -> usize {
    let selection = if self.state.selected().unwrap_or(1) == 0 {
      self.rows.len() - 1
    } else {
      self.state.selected().unwrap_or(1) - 1
    };

    selection
  }

  fn find_prev_section(&mut self) -> usize {
    let mut prev = self.find_prev_row();
    while prev > 0 {
      if let Some(row) = self.rows.get(prev) {
        if let Some(data) = &self.row_lookup.get(row) {
          if let LookupType::Backend(_) = data {
            break;
          }
        }
      }
      self.state.select(Some(prev));
      prev = self.find_prev_row();
    }
    prev
  }

  fn create_table(&mut self) -> () {
    let mut lengths = Vec::new();
    for header in &self.headers {
      lengths.push(Constraint::Length(15));
    }

    if lengths.is_empty() {
      lengths.push(Constraint::Length(15));
    }

    if let Some(area) = self.area {
      lengths[0] = Constraint::Length(area.width - (lengths.len() as u16 - 1) * 15);
    }
    // lengths[0] = Constraint::Length(area.width - (lengths.len() as u16 - 1) * 15);

    let table = Table::new(self.rows.iter().map(|row| row.clone()), lengths)
      .header(Row::new(self.headers.clone()).bold())
      .highlight_style(Style::new().light_yellow());

    self.table = table;
  }

}

impl Component for Items<'_> {
  fn init(&mut self, size: Rect) -> Result<()> {
    self.rows = Vec::new();
    if self.state.selected().is_none() {
      self.state.select(Some(0));
    }
    if let Some(metrics) = &self.metrics {
      self.update_rows();
    }

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

  fn move_down(&mut self, mode: MovementMode) -> Result<Option<Action>> {
    if self.rows.is_empty() {
      return Ok(None);
    }

    let selection = match mode {
      MovementMode::Single => self.find_next_row(),
      MovementMode::Section => match self.resource {
        ResourceType::Combined => self.find_next_section(),
        _ => self.find_next_row(),
      },
    };

    self.state.select(Some(selection));

    log::info!("Selected: {}", selection);

    Ok(None)
  }

  fn move_up(&mut self, mode: MovementMode) -> Result<Option<Action>> {
    if self.rows.is_empty() {
      return Ok(None);
    }

    let selection = match mode {
      MovementMode::Single => self.find_prev_row(),
      MovementMode::Section => match self.resource {
        ResourceType::Combined => self.find_prev_section(),
        _ => self.find_prev_row(),
      },
    };

    self.state.select(Some(selection));

    log::info!("Selected: {}", selection);

    Ok(None)
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::MetricUpdate(metrics) => {
        self.metrics = Some(metrics.clone());
        self.update_rows();
        Ok(None)
      },
      Action::SelectResource(resource) => {
        self.resource = resource;
        self.state.select(Some(0));
        self.update_rows();
        Ok(None)
      },
      Action::SelectStatus(resource) => {
        self.status_filter = resource;
        self.state.select(Some(0));
        self.update_rows();
        Ok(None)
      },
      Action::Filter(filter_string) => {
        self.filter = Some(filter_string);
        self.state.select(Some(0));
        self.update_rows();
        Ok(None)
      },
      Action::Sticky => {
        match self.state.selected() {
          Some(selection) => {
            if let Some(row) = self.rows.get(selection) {
              if let Some(data) = &self.row_lookup.get(row) {
                let data = match data {
                  LookupType::Backend(backend) => backend,
                  LookupType::Server(server) => server,
                };

                if self.sticky_backends.contains(&data.name.clone().unwrap_or("".to_string())) {
                  self.sticky_backends.remove(&data.name.clone().unwrap_or("".to_string()));
                  self.update_rows();
                } else {
                  self.sticky_backends.insert(data.name.clone().unwrap_or("".to_string()));
                  self.update_rows();
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
                let data = match data {
                  LookupType::Backend(backend) => backend,
                  LookupType::Server(server) => server,
                };

                let name = data.name.clone().unwrap_or("".to_string());

                tx.send(Action::SwitchMode(Mode::Info))?;
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
    self.area = Some(area);

    let border = Block::new()
      .title(self.resource.to_string())
      .borders(Borders::ALL)
      .border_style(Style::default().fg(Color::White));

    f.render_widget(border, area);
    f.render_stateful_widget(&self.table, area, &mut self.state);
    Ok(())
  }
}
