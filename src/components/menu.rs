use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Style},
  widgets::{Block, Borders, Paragraph},
};
use tui_textarea::TextArea;

use crate::{
  action::{Action, TypingMode},
  stats::data::{ResourceType, StatusType},
  tui::Frame,
};

use super::Component;

#[derive(Default, Debug, Clone)]
pub struct Menu<'a> {
  resource: ResourceType,
  status: StatusType,
  filter: TextArea<'a>,
  focused: bool,
}

impl Menu<'_> {
  pub fn new() -> Self {
    Self { resource: ResourceType::default(), 
           status: StatusType::default(),
           filter: TextArea::default(), 
           focused: false 
        }
  }
}

impl Component for Menu<'_> {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::TypingMode(typing_mode) => {
        self.focused = typing_mode == TypingMode::Filter;
        Ok(None)
      },
      _ => Ok(None),
    }
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let resources = vec![ResourceType::Combined, ResourceType::Backend, ResourceType::Frontend, ResourceType::Server];
    let status = vec![StatusType::All, StatusType::Failing, StatusType::Healthy];

    let sides = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(vec![Constraint::Length(15), Constraint::Length(15), Constraint::Min(0)])
      .split(rect);

    let resource_lengths = vec![Constraint::Length(1); resources.len() + 1];
    let status_lengths = vec![Constraint::Length(1); status.len() + 1];

    let left_resources = Layout::default().direction(Direction::Vertical).constraints(resource_lengths).split(sides[0]);
    let left_status = Layout::default().direction(Direction::Vertical).constraints(status_lengths).split(sides[1]);

    let right = Layout::default()
      .direction(Direction::Vertical)
      .constraints(vec![Constraint::Length(3), Constraint::Min(0)])
      .split(sides[2]);

    let resource_border =
      Block::new().title("Resource").borders(Borders::ALL).border_style(Style::default().fg(Color::White));
    f.render_widget(resource_border, sides[0]);

    let status_border = Block::new().title("Status").borders(Borders::ALL).border_style(Style::default().fg(Color::White));
    f.render_widget(status_border, sides[1]);

    for i in 0..resources.len() {
      let resource = resources[i];
      let resource_inactive_style = Style::default().fg(Color::White);
      let resource_active_style = Style::default().fg(Color::Yellow);
      let resource_widget = Paragraph::new(format!("{}: {}", i, resource))
        .style(if resource == self.resource { resource_active_style } else { resource_inactive_style })
        .block(Block::default().borders(Borders::NONE));

      f.render_widget(resource_widget, left_resources[i + 1]);
    }

    let status_keys = vec!['a', 'e', 'h'];
    for i in 0..status.len() {
      let status = status[i];
      let status_inactive_style = Style::default().fg(Color::White);
      let status_active_style = Style::default().fg(Color::Yellow);
      let status_widget = Paragraph::new(format!("{}: {}", status_keys[i], status))
        .style(if status == self.status { status_active_style } else { status_inactive_style })
        .block(Block::default().borders(Borders::NONE));

      f.render_widget(status_widget, left_status[i + 1]);
    }

    let filter_border = Block::new().title("Filter").borders(Borders::ALL);

    let filter_border = match self.focused {
      true => filter_border.border_style(Style::default().fg(Color::Yellow)),
      false => filter_border.border_style(Style::default().fg(Color::White)),
    };

    f.render_widget(self.filter.widget(), filter_border.inner(right[0]));

    f.render_widget(filter_border, right[0]);

    Ok(())
  }

  fn handle_key_events(&mut self, typing_mode: TypingMode, key: KeyEvent) -> Result<Option<Action>> {
    if typing_mode == TypingMode::Filter {
      match key {
        KeyEvent { code: KeyCode::Esc, .. } | KeyEvent { code: KeyCode::Enter, .. } => {
          self.focused = false;
          Ok(Some(Action::TypingMode(TypingMode::Navigation)))
        },
        input => {
          self.filter.input(input);
          Ok(Some(Action::Filter(self.filter.lines()[0].to_string())))
        },
      }
    } else {
      match key.code {
        KeyCode::Char('0') => {
          self.resource = ResourceType::Combined;
          Ok(Some(Action::SelectResource(self.resource)))
        },
        KeyCode::Char('1') => {
          self.resource = ResourceType::Backend;
          Ok(Some(Action::SelectResource(self.resource)))
        },
        KeyCode::Char('2') => {
          self.resource = ResourceType::Frontend;
          Ok(Some(Action::SelectResource(self.resource)))
        },
        KeyCode::Char('3') => {
          self.resource = ResourceType::Server;
          Ok(Some(Action::SelectResource(self.resource)))
        },
        KeyCode::Char('f') | KeyCode::Char('/') => {
          self.focused = true;
          Ok(Some(Action::TypingMode(TypingMode::Filter)))
        },
        KeyCode::Char('a') => {
          self.status = StatusType::All;
          Ok(Some(Action::SelectStatus(self.status)))
        },
        KeyCode::Char('e') => {
          self.status = StatusType::Failing;
          Ok(Some(Action::SelectStatus(self.status)))
        },
        KeyCode::Char('h') => {
          self.status = StatusType::Healthy;
          Ok(Some(Action::SelectStatus(self.status)))
        },
        _ => Ok(None),
      }
    }
  }
}
