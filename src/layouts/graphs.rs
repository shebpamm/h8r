use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout, Rect, Size};

use crate::{
  action::{Action, TypingMode},
  components::{fps::FpsCounter, items::Items, menu::Menu, status::Status, charts, Component},
  config::Config,
  tui::{Event, Frame}, mode::Mode,
};

pub struct GraphLayout {
  pub components: Vec<Box<dyn Component>>,
  pub layout: Layout,
  action_handler: Option<tokio::sync::mpsc::UnboundedSender<Action>>,
}

impl GraphLayout {
  pub fn new() -> Self {
    let mut components: Vec<Box<dyn Component>> = Vec::new();
    components.push(Box::new(Status::new()));
    components.push(Box::new(charts::HTTPErrorChart::new()));
    Self {
      components,
      action_handler: None,
      layout: Layout::default().direction(Direction::Vertical).constraints(vec![
        Constraint::Length(5),
        Constraint::Min(0),
      ]),
    }
  }
}

impl Component for GraphLayout {
  fn init(&mut self, area: Rect) -> Result<()> {
    let layout = self.layout.split(area);
    // Give each element a slice of the screen
    for (i, component) in self.components.iter_mut().enumerate() {
      component.init(layout[i])?;
    }
    Ok(())
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    let layout = self.layout.split(area);
    // Give each element a slice of the screen
    for (i, component) in self.components.iter_mut().enumerate() {
      component.draw(f, layout[i])?;
    }
    Ok(())
  }

  fn register_action_handler(&mut self, tx: tokio::sync::mpsc::UnboundedSender<Action>) -> Result<()> {
    self.action_handler = Some(tx.clone());

    for component in self.components.iter_mut() {
      component.register_action_handler(tx.clone())?;
    }
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    for component in self.components.iter_mut() {
      component.register_config_handler(config.clone())?;
    }
    Ok(())
  }

  fn move_down(&mut self) -> Result<Option<Action>> {
    for component in self.components.iter_mut() {
      component.move_down()?;
    }
    Ok(None)
  }

  fn move_up(&mut self) -> Result<Option<Action>> {
    for component in self.components.iter_mut() {
      component.move_up()?;
    }
    Ok(None)
  }

  fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
    let r = match event {
      Some(crate::tui::Event::ModeKey(typing_mode, key_event)) => self.handle_key_events(typing_mode, key_event)?,
      Some(crate::tui::Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
      _ => None,
    };
    Ok(r)
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    let mut actions: Vec<Action> = Vec::new();

    for component in self.components.iter_mut() {
      if let Some(action) = component.update(action.clone())? {
        actions.push(action);
      }
    }

    if let Some(action_handler) = &self.action_handler {
      for action in actions {
        action_handler.send(action)?;
      }
    }

    Ok(None)
  }

  fn handle_key_events(&mut self, typing_mode: TypingMode, key: KeyEvent) -> Result<Option<Action>> {
    // hack because i don't bother with fixing this in app.rs
    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
      return Ok(Some(Action::SwitchMode(Mode::Home)));
    }

    let mut actions: Vec<Action> = Vec::new();
    for component in self.components.iter_mut() {
      if let Some(action) = component.handle_key_events(typing_mode.clone(), key.clone())? {
        actions.push(action);
      }
    }

    if let Some(action_handler) = &self.action_handler {
      for action in actions {
        action_handler.send(action)?;
      }
    }

    Ok(None)
  }
}
