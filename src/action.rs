use std::{fmt, string::ToString};

use serde::{
  de::{self, Deserializer, Visitor},
  Deserialize, Serialize,
};
use strum::Display;

use crate::stats::data::HaproxyStat;

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
}
