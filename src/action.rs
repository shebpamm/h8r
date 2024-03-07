use crate::mode::Mode;
use crate::stats::metrics::HaproxyMetrics;
use std::sync::Arc;
use std::{fmt, string::ToString};

use serde::{
  de::{self, Deserializer, Visitor},
  Deserialize, Serialize,
};
use strum::Display;
use tokio::sync::RwLock;

use crate::stats::data::{HaproxyStat, ResourceType};

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Display, Deserialize)]
pub enum TypingMode {
  #[default]
  Navigation,
  Filter,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Display, Deserialize)]
pub enum MovementMode {
    #[default]
    Single,
    Section,
}

#[derive(Debug, Clone, PartialEq, Serialize, Display, Deserialize)]
pub enum Action {
  Tick,
  Render,
  Resize(u16, u16),
  Suspend,
  Resume,
  Quit,
  Refresh,
  Error(String),
  Help,
  MoveUp,
  MoveDown,
  MoveSectionUp,
  MoveSectionDown,
  Sticky,
  UpdateStats(Vec<HaproxyStat>),
  MetricUpdate(HaproxyMetrics),
  SelectResource(ResourceType),
  TypingMode(TypingMode),
  Filter(String),
  SwitchMode(Mode),
  UseItem(String),
  SelectItem,
}
