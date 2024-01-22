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

#[derive(Debug, Clone, PartialEq, Serialize, Display, Deserialize)]
pub enum TypingMode {
    Navigation,
    Filter,
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
  UpdateStats(Vec<HaproxyStat>),
  MetricUpdate(HaproxyMetrics),
  SelectResource(ResourceType),
  TypingMode(TypingMode)
}
