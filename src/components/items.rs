use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

pub struct Items {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  state: TableState,
}

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};

impl Items {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            state: TableState::default(),
        }
    }

    fn table(&mut self) -> Table<'static> {
            Table::new(vec![
                Row::new(vec!["Item 1", "Item 2", "Item 3"]),
                Row::new(vec!["Item 4", "Item 5", "Item 6"]),
                Row::new(vec!["Item 7", "Item 8", "Item 9"]),
            ], [
                Constraint::Length(15),
                Constraint::Length(15),
                Constraint::Length(15)])
            .header(Row::new(vec!["Header 1", "Header 2", "Header 3"]))
    }
}

impl Component for Items {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        f.render_stateful_widget(self.table(), area, &mut self.state);
        Ok(())
    }
}
