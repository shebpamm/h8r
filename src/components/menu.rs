use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Style},
  widgets::{Block, Borders, Paragraph},
};
use tui_textarea::TextArea;

use crate::{action::{Action, TypingMode}, stats::data::ResourceType, tui::Frame};

use super::Component;

#[derive(Debug, Clone)]
pub struct Menu<'a> {
  resource: ResourceType,
  filter: TextArea<'a>,
  typing_mode: TypingMode,
}

impl Default for Menu<'_> {
  fn default() -> Self {
    Self::new()
  }
}

impl Menu<'_> {
  pub fn new() -> Self {
    Self { 
            resource: ResourceType::default(),
            filter: TextArea::default(),
            typing_mode: TypingMode::default(),
        }
  }
}

impl Component for Menu<'_> {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
        Action::TypingMode(typing_mode) => {
            self.typing_mode = typing_mode;
            Ok(None)
        },
        _ => Ok(None),
    }
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let resources = vec![ResourceType::Combined, ResourceType::Backend, ResourceType::Frontend, ResourceType::Server];

    let sides = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(vec![Constraint::Length(15), Constraint::Min(0)])
      .split(rect);

    let lengths = vec![Constraint::Length(1); resources.len()+1];

    let left = Layout::default()
      .direction(Direction::Vertical)
      .constraints(lengths)
      .split(sides[0]);

    let right = Layout::default()
      .direction(Direction::Vertical)
      .constraints(vec![Constraint::Length(3), Constraint::Min(0)])
      .split(sides[1]);


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

    let filter_border =
      Block::new().title("Filter").borders(Borders::ALL);

    let filter_border = match self.typing_mode {
        TypingMode::Filter => 
            filter_border.border_style(Style::default().fg(Color::Yellow)),
        _ => filter_border.border_style(Style::default().fg(Color::White)),
    };


    f.render_widget(self.filter.widget(), filter_border.inner(right[0]));

    f.render_widget(filter_border, right[0]);

    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    if self.typing_mode == TypingMode::Filter {
        match key {
            KeyEvent { code: KeyCode::Esc, .. } |
            KeyEvent { code: KeyCode::Enter, .. } => {
                self.typing_mode = TypingMode::Navigation;
                Ok(Some(Action::TypingMode(TypingMode::Navigation)))
            },
            input => {
                self.filter.input(input);
                Ok(Some(Action::Filter(self.filter.lines()[0].to_string())))
            }
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
            self.typing_mode = TypingMode::Filter;
            Ok(Some(Action::TypingMode(TypingMode::Filter)))
          },
          _ => Ok(None),
        }
    }
  }
}
