use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
  action::Action,
  config::{Config, KeyBindings},
  stats::data::{HaproxyStat, ResourceType},
};

pub struct Items<'a> {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  state: TableState,
  data: Vec<HaproxyStat>,
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
      data: Vec::default(),
      rows: Vec::default(),
      resource: ResourceType::Backend,
    }
  }

  fn update_rows(&mut self, data: Vec<HaproxyStat>) {
    let mut rows = Vec::new();

    // Rows
    let data: Vec<HaproxyStat> = data
      .into_iter()
      .filter(match self.resource {
        ResourceType::Backend => |row: &HaproxyStat| row.svname == Some("BACKEND".to_string()),
        ResourceType::Frontend => |row: &HaproxyStat| row.svname == Some("FRONTEND".to_string()),
        ResourceType::Server => {
          |row: &HaproxyStat| row.svname != Some("BACKEND".to_string()) && row.svname != Some("FRONTEND".to_string())
        },
      })
      .collect();

    // Headers
    self.headers = match self.resource {
      ResourceType::Backend => vec!["".to_string(), "Name".to_string(), "Status".to_string()],
      ResourceType::Frontend => vec!["".to_string(), "Name".to_string(), "Status".to_string()],
      ResourceType::Server => vec!["".to_string(), "Status".to_string(), "".to_string()],
    };


    // Columns
    for row in data {
      match self.resource {
        ResourceType::Backend => rows.push(Row::new(vec![
          row.pxname.unwrap_or("".to_string()),
          row.svname.unwrap_or("".to_string()),
          row.status.unwrap_or("".to_string()),
        ])),
        ResourceType::Frontend => rows.push(Row::new(vec![
          row.pxname.unwrap_or("".to_string()),
          row.svname.unwrap_or("".to_string()),
          row.status.unwrap_or("".to_string()),
        ])),
        ResourceType::Server => rows.push(Row::new(vec![
          row.svname.unwrap_or("".to_string()),
          row.status.unwrap_or("".to_string()),
          "".to_string(),
        ])),
      }
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
    self.state.select(Some(self.state.selected().unwrap_or(1) + 1));
    log::info!("Selected: {}", self.state.selected().unwrap_or(1));
    Ok(None)
  }

  fn move_up(&mut self) -> Result<Option<Action>> {
    self.state.select(Some(self.state.selected().unwrap_or(1) - 1));
    log::info!("Selected: {}", self.state.selected().unwrap_or(1));
    Ok(None)
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::UpdateStats(stats) => {
        self.data = stats.clone();
        self.update_rows(stats);
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
