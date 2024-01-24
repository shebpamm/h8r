use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Style},
  widgets::{Block, Borders, Paragraph},
};

use ratatui::{prelude::*, widgets::*};
use tui_textarea::TextArea;

use crate::{
  action::{Action, TypingMode},
  stats::data::ResourceType,
  tui::Frame,
};

use super::Component;

#[derive(Debug, Clone)]
pub struct HTTPErrorChart {
}

impl Default for HTTPErrorChart {
  fn default() -> Self {
    Self::new()
  }
}

impl HTTPErrorChart {
  pub fn new() -> Self {
    Self { }
  }
}

impl Component for HTTPErrorChart {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
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
    for i in 0..lengths.len() {
      let key = Span::styled(format!("{}: ", i), Style::default().fg(Color::Yellow));
      let value = Span::styled("test", Style::default().fg(Color::White));
      
      let text = Paragraph::new(Line::from(vec![key, value]));

      f.render_widget(text.clone(), left[i]);
      f.render_widget(text, right[i]);
    }

    Ok(())
  }
}
