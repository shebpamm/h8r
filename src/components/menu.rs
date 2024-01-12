use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Style},
  widgets::{Block, Borders, Paragraph},
};

use crate::{action::Action, stats::data::ResourceType, tui::Frame};

use super::Component;

#[derive(Debug, Clone, PartialEq)]
pub struct Menu {
  resource: ResourceType,
}

impl Default for Menu {
  fn default() -> Self {
    Self::new()
  }
}

impl Menu {
  pub fn new() -> Self {
    Self { resource: ResourceType::Backend }
  }
}

impl Component for Menu {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let sides = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(vec![Constraint::Length(15), Constraint::Min(0)])
      .split(rect);

    let left = Layout::default()
      .direction(Direction::Vertical)
      .constraints(vec![Constraint::Length(1), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
      .split(sides[0]);

    let resources = vec![ResourceType::Backend, ResourceType::Frontend, ResourceType::Server];

    let resource_border =
      Block::new().title("Resource").borders(Borders::ALL).border_style(Style::default().fg(Color::White));

    f.render_widget(resource_border, sides[0]);
    for i in 0..resources.len() {
      let resource = resources[i];
      let resource_inactive_style = Style::default().fg(Color::White);
      let resource_active_style = Style::default().fg(Color::Yellow);
      let resource_widget = Paragraph::new(format!("{}: {}", i, resource))
        .style(if resource == self.resource { resource_active_style } else { resource_inactive_style })
        .block(Block::default().borders(Borders::NONE));

      f.render_widget(resource_widget, left[i + 1]);
    }

    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    match key.code {
      KeyCode::Char('0') => {
        self.resource = ResourceType::Backend;
        Ok(Some(Action::SelectResource(self.resource)))
      },
      KeyCode::Char('1') => {
        self.resource = ResourceType::Frontend;
        Ok(Some(Action::SelectResource(self.resource)))
      },
      KeyCode::Char('2') => {
        self.resource = ResourceType::Server;
        Ok(Some(Action::SelectResource(self.resource)))
      },
      _ => Ok(None),
    }
  }
}
