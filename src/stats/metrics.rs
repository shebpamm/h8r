use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
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
  pub name: String,
  pub status: HaproxyFrontendStatus,
  #[serde(rename = "req_tot")]
  pub requests: f64,
  #[serde(rename = "scur")]
  pub sessions: i64,
}
impl FromHaproxyStat for HaproxyFrontend {}

#[derive(Debug, Display, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HaproxyBackendStatus {
  Up,
  Down,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HaproxyBackend {
  #[serde(rename = "pxname")]
  pub name: String,
  pub status: HaproxyBackendStatus,
  #[serde(rename = "req_tot")]
  pub requests: f64,

  #[serde(skip)]
  pub servers: Vec<HaproxyServer>,
}

impl FromHaproxyStat for HaproxyBackend {}

#[derive(Debug, Display, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HaproxyServerStatus {
  Up,
  Down,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HaproxyServer {
  #[serde(rename = "svname")]
  pub name: String,
  #[serde(rename = "pxname")]
  pub backend_name: String,
  pub status: HaproxyServerStatus,
  #[serde(rename = "req_tot")]
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
  pub history: Vec<InstantHaproxyMetrics>,
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
      let svname = row.svname.clone().unwrap_or("".to_string()).parse::<SVNameMeaning>()?;

      match svname {
        SVNameMeaning::Frontend => {
          frontends.push(HaproxyFrontend::new(row.to_owned())?);
        },
        SVNameMeaning::Backend => {
          backends.push(HaproxyBackend::new(row.to_owned())?);
        },
        SVNameMeaning::Server => {
          servers.push(HaproxyServer::new(row.to_owned())?);
        },
      }
    }

    for backend in &mut backends {
      backend.servers = servers
        .iter()
        .filter(|server| server.backend_name == backend.name)
        .cloned()
        .collect::<Vec<HaproxyServer>>();
    }

    self.instant = Some(InstantHaproxyMetrics {
      data: InstantHaproxyMetricData { raw: data, frontends, backends, servers },
      time: Local::now(),
    });

    Ok(())
  }
}
