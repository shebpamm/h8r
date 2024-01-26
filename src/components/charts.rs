use std::collections::HashMap;

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

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum Metric {
  HTTPError,
  HTTPRequest,
  Sessions,
}

#[derive(Debug, Clone)]
pub struct HTTPErrorChart {
  datasets: HashMap<Metric, Vec<(f64, f64)>>,
  x_bounds: [f64; 2],
  y_bounds: [f64; 2],
  selected_backend: Option<String>,
}

impl Default for HTTPErrorChart {
  fn default() -> Self {
    Self::new()
  }
}

impl HTTPErrorChart {
  pub fn new() -> Self {
    let mut datasets = HashMap::new();
    datasets.insert(Metric::HTTPError, Vec::new());
    datasets.insert(Metric::HTTPRequest, Vec::new());
    datasets.insert(Metric::Sessions, Vec::new());
    Self { datasets, selected_backend: None, x_bounds: [0., 0.], y_bounds: [0., 0.] }
  }

  fn calculate_x_bounds(&self) -> Option<[f64; 2]> {
    // Find first and last data point in all of the datasets
    let first = self
      .datasets
      .values()
      .map(|dataset| dataset.first())
      .flatten()
      .map(|point| point.0)
      .min_by(|a, b| a.partial_cmp(b).unwrap());
    let last = self
      .datasets
      .values()
      .map(|dataset| dataset.last())
      .flatten()
      .map(|point| point.0)
      .max_by(|a, b| a.partial_cmp(b).unwrap());

    match (first, last) {
      (Some(first), Some(last)) => Some([first, last]),
      _ => None,
    }
  }

  fn calculate_y_bounds(&self) -> Option<[f64; 2]> {
    // Find highest value across all datasets
    let maximum = self
      .datasets
      .values()
      .map(|dataset| dataset.iter().map(|point| point.1).max_by(|a, b| a.partial_cmp(b).unwrap()))
      .flatten()
      .max_by(|a, b| a.partial_cmp(b).unwrap());

    // We should always have at  minimum a range of 0 to 10
    if let Some(maximum) = maximum {
      if maximum < 10.0 {
        return Some([0., 10.]);
      }
    }

    match maximum {
      Some(maximum) => Some([0., maximum]),
      _ => None,
    }
  }

  fn calculate_data(&self, metric: Metric, metrics: HaproxyMetrics) -> Vec<(f64, f64)> {
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
        let point = match metric {
          Metric::HTTPError => backend.http_500_req as f64,
          Metric::HTTPRequest => backend.requests as f64,
          Metric::Sessions => backend.sessions as f64,
        };
        data.push((time, point));
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
    // Calculate datasets
    let http_error_data = self.calculate_data(Metric::HTTPError, metrics.clone());
    let http_request_data = self.calculate_data(Metric::HTTPRequest, metrics.clone());
    let sessions_data = self.calculate_data(Metric::Sessions, metrics.clone());

    // Update datasets
    self.datasets.insert(Metric::HTTPError, http_error_data);
    self.datasets.insert(Metric::HTTPRequest, http_request_data);
    self.datasets.insert(Metric::Sessions, sessions_data);

    // Calculate bounds
    if let Some(bounds) = self.calculate_x_bounds() {
      self.x_bounds = bounds;
    } else {
      self.x_bounds = [0., 0.];
    }

    if let Some(bounds) = self.calculate_y_bounds() {
      self.y_bounds = bounds;
    } else {
      self.y_bounds = [0., 0.];
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
        // old:
    // let dataset =
    //   Dataset::default().name("HTTP Errors").data(&self.data).marker(Marker::Braille).graph_type(GraphType::Line).red();

    // Create the datasets
    let http_error_dataset = Dataset::default()
      .name("HTTP Errors")
      .marker(Marker::Braille)
      .graph_type(GraphType::Line)
      .style(Style::default().red())
      .data(&self.datasets.get(&Metric::HTTPError).unwrap());

    let http_request_dataset = Dataset::default()
        .name("HTTP Requests")
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().blue())
        .data(&self.datasets.get(&Metric::HTTPRequest).unwrap());

    let sessions_dataset = Dataset::default()
        .name("Sessions")
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().green())
        .data(&self.datasets.get(&Metric::Sessions).unwrap());

    let datasets = vec![http_error_dataset, http_request_dataset, sessions_dataset];


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
      .bounds(self.y_bounds)
      .labels(vec![self.y_bounds[0].floor().to_string().bold(), self.y_bounds[1].floor().to_string().bold()]);

    // Create the chart and link all the parts together
    let chart = Chart::new(datasets).block(Block::default().title("Chart")).x_axis(x_axis).y_axis(y_axis);

    f.render_widget(chart, rect);

    Ok(())
  }
}
