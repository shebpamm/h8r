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

use crate::stats::data::{ResourceType, StatusType};

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
  #[serde(with = "serde_arc_metrics")]
  MetricUpdate(Arc<HaproxyMetrics>),
  SelectResource(ResourceType),
  SelectStatus(StatusType),
  TypingMode(TypingMode),
  Filter(String),
  SwitchMode(Mode),
  UseItem(String),
  SelectItem,
}


mod serde_arc_metrics {
    use super::*;
    use serde::{Serializer, Deserializer};
    use serde::ser::Serialize;
    use serde::de::Deserialize;

    pub fn serialize<S>(value: &Arc<HaproxyMetrics>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.as_ref().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<HaproxyMetrics>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = HaproxyMetrics::deserialize(deserializer)?;
        Ok(Arc::new(data))
    }
}
