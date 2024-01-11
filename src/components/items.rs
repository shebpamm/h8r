use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};

pub struct Items<'a> {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  state: TableState,
  rows: Vec<Row<'a>>,
}

impl Items<'_> {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            state: TableState::default(),
            rows: Vec::default()
        }
    }

}

impl Component for Items<'_> {
    fn init(&mut self, size: Rect) -> Result<()> {
        let mut rows = Vec::new();
        for i in 0..100 {
            rows.push(Row::new(vec![
                format!("Item {}", i),
                format!("Item {}", i),
                format!("Item {}", i),
            ]));
        }
        self.rows = rows;
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

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let table = Table::new(self.rows.clone(), [
                Constraint::Length(area.width - 30),
                Constraint::Length(15),
                Constraint::Length(15)])
            .header(Row::new(vec!["Header 1", "Header 2", "Header 3"]).bold())
            .highlight_style(Style::new().light_yellow());

        f.render_stateful_widget(table, area, &mut self.state);
        Ok(())
    }
}
