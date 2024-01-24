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
  stats::{data::ResourceType, metrics::HaproxyMetrics},
  tui::Frame,
};

use super::Component;

#[derive(Debug, Clone)]
pub struct HTTPErrorChart {
  data: Vec<(f64, f64)>,
  x_bounds: [f64; 2],
  selected_backend: Option<String>,
}

impl Default for HTTPErrorChart {
  fn default() -> Self {
    Self::new()
  }
}

impl HTTPErrorChart {
  pub fn new() -> Self {
    Self { data: Vec::new(), selected_backend: None, x_bounds: [0., 0.] }
  }

  fn calculate_bounds(&self) -> Option<[f64; 2]> {
    let first = self.data.first().map(|point| point.0);
    let last = self.data.last().map(|point| point.0);

    match (first, last) {
      (Some(first), Some(last)) => Some([first, last]),
      _ => None,
    }
  }

  fn calculate_data(&self, metrics: HaproxyMetrics) -> Vec<(f64, f64)> {
    // Data is an array of f64 tuples ((f64, f64)), the first element being X and the second Y.
    // It’s also worth noting that, unlike the Rect, here the Y axis is bottom to top, as in
    // math.
    let mut data: Vec<(f64, f64)> = Vec::new();
    for instant in metrics.history {
        let mut time = instant.time.timestamp_millis() as f64;
        // relative to current timestamp
        time = time - metrics.instant.clone().unwrap().time.timestamp_millis() as f64;

        // in seconds
        time = time / 1000.0;

        // find the correct backend
        let backend = instant.data.backends.iter().find(|b| b.name == self.selected_backend);
        if let Some(backend) = backend {
            let errors = backend.http_400_req as f64;
            data.push((time, errors));
        }
    }
    // Calculate rate of change
    let mut rate_of_change_data: Vec<(f64, f64)> = Vec::new();
    for i in 0..data.len() - 1 {
        let (x1, y1) = data[i];
        let (x2, y2) = data[i + 1];

        // Calculate rate of change (derivative)
        let delta_x = x2 - x1;
        let delta_y = y2 - y1;
        let rate_of_change = delta_y / delta_x;

        rate_of_change_data.push((x1, rate_of_change));
    }
    rate_of_change_data
  }

  fn update_dataset(&mut self, metrics: HaproxyMetrics) -> Result<()> {
    // Calculate data
    let data = self.calculate_data(metrics);
    self.data = data;

    // Calculate bounds
    if let Some(bounds) = self.calculate_bounds() {
      self.x_bounds = bounds;
    } else {
      self.x_bounds = [0., 0.];
    }

    Ok(())
  }
}

impl Component for HTTPErrorChart {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::MetricUpdate(metrics) => {
        self.update_dataset(metrics)?;
        Ok(None)
      },
      Action::UseItem(backend_name) => {
        self.selected_backend = Some(backend_name);
        Ok(None)
      },
      _ => Ok(None),
    }
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let dataset =
      Dataset::default().name("HTTP Errors").data(&self.data).marker(Marker::Braille).graph_type(GraphType::Line).red();

    // Create the X axis and define its properties
    let x_axis = Axis::default()
      .title("X Axis".red())
      .style(Style::default().white())
      .bounds(self.x_bounds)
      .labels(vec![self.x_bounds[0].floor().to_string().bold(), self.x_bounds[1].floor().to_string().bold()]);

    // Create the Y axis and define its properties
    let y_axis = Axis::default()
      .title("Y Axis".green())
      .style(Style::default().white())
      .bounds([0.0, 10.0])
      .labels(vec!["0.0".into(), "5.0".into(), "10.0".into()]);

    // Create the chart and link all the parts together
    let chart = Chart::new(vec![dataset]).block(Block::default().title("Chart")).x_axis(x_axis).y_axis(y_axis);

    f.render_widget(chart, rect);

    Ok(())
  }
}
