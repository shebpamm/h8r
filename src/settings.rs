use serde::Deserialize;
use crate::stats::data::ResourceType;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Settings {
  resource: ResourceType,
  filter: Option<String>,
}
