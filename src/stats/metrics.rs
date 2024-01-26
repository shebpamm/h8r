use chrono::{DateTime, Local};
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

use color_eyre::eyre::Result;
use strum::Display;
use thiserror::Error;

use super::data::{HaproxyStat, ResourceType};

#[derive(Error, Debug)]
pub enum MetricError {
  #[error("Invalid SVNameMeaning")]
  InvalidSVNameMeaning,
}

#[derive(Debug, Display, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HaproxyFrontendStatus {
  Open,
  Closed,
}

#[derive(Debug, PartialEq)]
pub enum SVNameMeaning {
  Frontend,
  Backend,
  Server,
}

impl FromStr for SVNameMeaning {
  type Err = MetricError;

  fn from_str(s: &str) -> Result<Self, MetricError> {
    match s {
      "FRONTEND" => Ok(SVNameMeaning::Frontend),
      "BACKEND" => Ok(SVNameMeaning::Backend),
      _ => Ok(SVNameMeaning::Server),
    }
  }
}

trait FromHaproxyStat {
  fn new(row: HaproxyStat) -> Result<Self>
  where
    Self: Sized + serde::de::DeserializeOwned,
  {
    let v = serde_json::to_value(row)?;
    let frontend: Self = serde_json::from_value(v)?;

    Ok(frontend)
  }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HaproxyFrontend {
  #[serde(rename = "pxname")]
  pub name: Option<String>,
  pub status: HaproxyFrontendStatus,
  #[serde(rename = "req_tot")]
  #[serde(deserialize_with = "deserialize_null_default")]
  pub requests: f64,
  #[serde(rename = "scur")]
  pub sessions: i64,
}
impl FromHaproxyStat for HaproxyFrontend {}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HaproxyBackend {
  #[serde(rename = "pxname")]
  pub name: Option<String>,
  pub status: String,
  #[serde(rename = "req_tot")]
  #[serde(deserialize_with = "deserialize_null_default")]
  pub requests: f64,
  #[serde(rename = "mode")]
  pub proxy_mode: String,
  #[serde(rename = "act")]
  pub active_servers: i64,
  #[serde(rename = "bck")]
  pub backup_servers: i64,
  #[serde(rename = "scur")]
  pub sessions: i64,
  #[serde(rename = "hrsp_5xx")]
  pub http_500_req: f64,
  #[serde(rename = "hrsp_4xx")]
  pub http_400_req: f64,

  #[serde(skip)]
  pub servers: Vec<HaproxyServer>,
}

impl FromHaproxyStat for HaproxyBackend {}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HaproxyServer {
  #[serde(rename = "svname")]
  pub name: Option<String>,
  #[serde(rename = "pxname")]
  pub backend_name: Option<String>,
  pub status: String,
  #[serde(rename = "check_status")]
  pub status_code: String,
  #[serde(rename = "req_tot")]
  #[serde(deserialize_with = "deserialize_null_default")]
  pub requests: f64,
}

impl FromHaproxyStat for HaproxyServer {}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InstantHaproxyMetricData {
  pub raw: Vec<HaproxyStat>,
  pub frontends: Vec<HaproxyFrontend>,
  pub backends: Vec<HaproxyBackend>,
  pub servers: Vec<HaproxyServer>,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InstantHaproxyMetrics {
  pub data: InstantHaproxyMetricData,
  pub time: DateTime<Local>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct HaproxyMetrics {
  pub instant: Option<InstantHaproxyMetrics>,
}

impl HaproxyMetrics {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn update(&mut self, data: Vec<HaproxyStat>) -> Result<()> {
    let mut frontends: Vec<HaproxyFrontend> = Vec::new();
    let mut backends: Vec<HaproxyBackend> = Vec::new();
    let mut servers: Vec<HaproxyServer> = Vec::new();

    for row in &data {
      match row.resource_type.unwrap_or(99) {
        0 => {
          frontends.push(HaproxyFrontend::new(row.to_owned())?);
        },
        1 => {
          backends.push(HaproxyBackend::new(row.to_owned())?);
        },
        2 => {
          servers.push(HaproxyServer::new(row.to_owned())?);
        },
        _ => {
          log::warn!("Unknown resource type: {:?}", row);
        },
      }
    }

    for backend in &mut backends {
      backend.servers =
        servers.iter().filter(|server| server.backend_name == backend.name).cloned().collect::<Vec<HaproxyServer>>();
    }

    self.instant = Some(InstantHaproxyMetrics {
      data: InstantHaproxyMetricData { raw: data, frontends, backends, servers },
      time: Local::now(),
    });

    Ok(())
  }
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
  T: Default + Deserialize<'de>,
  D: Deserializer<'de>,
{
  let opt = Option::deserialize(deserializer)?;
  Ok(opt.unwrap_or_default())
}
