use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Instant;

use color_eyre::eyre::Result;
use thiserror::Error;

use super::data::{HaproxyStat, ResourceType};

#[derive(Error, Debug)]
pub enum MetricError {
  #[error("Invalid SVNameMeaning")]
  InvalidSVNameMeaning,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct HaproxyFrontend {
  #[serde(rename = "pxname")]
  pub name: String,
  pub status: HaproxyFrontendStatus,
  #[serde(rename = "req_tot")]
  pub requests: f64,
  #[serde(rename = "scur")]
  pub sessions: i64,

  #[serde(skip)]
  pub backends: Vec<HaproxyBackend>,
}
impl HaproxyFrontend {
    fn new(row: HaproxyStat) -> Result<Self> {
        let v = serde_json::to_value(row)?;
        let frontend: HaproxyFrontend = serde_json::from_value(v)?;

        Ok(frontend)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum HaproxyBackendStatus {
  Up,
  Down,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct HaproxyBackend {
  pub name: String,
  pub status: HaproxyBackendStatus,
  pub requests: i64,
  pub connections: i64,

  pub servers: Vec<HaproxyServer>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum HaproxyServerStatus {
  Up,
  Down,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct HaproxyServer {
  pub name: String,
  pub status: HaproxyServerStatus,
  pub requests: i64,
  pub connections: i64,
}

#[derive(Debug, PartialEq)]
pub struct InstantHaproxyMetricData {
  pub raw: Vec<HaproxyStat>,
  pub frontends: Vec<HaproxyFrontend>,
  pub backends: Vec<HaproxyBackend>,
  pub servers: Vec<HaproxyServer>,
}
#[derive(Debug, PartialEq)]
pub struct InstantHaproxyMetrics {
  pub data: InstantHaproxyMetricData,
  pub time: Instant,
}

#[derive(Debug, PartialEq, Default)]
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

    for row in data {
      let svname = row.svname.clone().unwrap_or("".to_string()).parse::<SVNameMeaning>()?;

      match svname {
        SVNameMeaning::Frontend => {
          frontends.push(HaproxyFrontend::new(row)?);
        },
        SVNameMeaning::Backend => {},
        SVNameMeaning::Server => {},
      }
    }

    Ok(())
  }
}
