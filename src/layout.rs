use color_eyre::eyre::Result;
use ratatui::layout::{Layout, Direction, Constraint, Rect, Size};

use crate::{components::{Component,items::Items, fps::FpsCounter}, tui::Frame};

pub struct HomeLayout {
    pub components: Vec<Box<dyn Component>>,
    pub layout: Layout,
}

impl HomeLayout {
    pub fn new() -> Self {
        let mut components: Vec<Box<dyn Component>> = Vec::new();
        components.push(Box::new(FpsCounter::new()));
        components.push(Box::new(Items::new()));
        Self {
            components,
            layout: Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    vec![
                        Constraint::Length(5),
                        Constraint::Min(0),
                    ])
        }
    }
}

impl Component for HomeLayout {
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

    fn register_action_handler(&mut self, tx: tokio::sync::mpsc::UnboundedSender<crate::action::Action>) -> Result<()> {
        for component in self.components.iter_mut() {
            component.register_action_handler(tx.clone())?;
        }
        Ok(())
    }

    fn register_config_handler(&mut self, config: crate::config::Config) -> Result<()> {
        for component in self.components.iter_mut() {
            component.register_config_handler(config.clone())?;
        }
        Ok(())
    }

    fn move_down(&mut self) -> Result<Option<crate::action::Action>> {
        for component in self.components.iter_mut() {
            component.move_down()?;
        }
        Ok(None)
    }

    fn move_up(&mut self) -> Result<Option<crate::action::Action>> {
        for component in self.components.iter_mut() {
            component.move_up()?;
        }
        Ok(None)
    }
}
