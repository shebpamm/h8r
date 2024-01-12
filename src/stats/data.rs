use csv::StringRecord;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Deserialize, Serialize, PartialEq, PartialOrd, Clone, Default)]
pub struct HaproxyStat {
  pub pxname: Option<String>,
  pub svname: Option<String>,
  pub qcur: Option<i64>,
  pub qmax: Option<i64>,
  pub scur: Option<i64>,
  pub smax: Option<i64>,
  pub slim: Option<i64>,
  pub stot: Option<i64>,
  pub bin: Option<i64>,
  pub bout: Option<i64>,
  pub dreq: Option<i64>,
  pub dresp: Option<i64>,
  pub ereq: Option<i64>,
  pub econ: Option<i64>,
  pub eresp: Option<i64>,
  pub wretr: Option<i64>,
  pub wredis: Option<i64>,
  pub status: Option<String>,
  pub weight: Option<i64>,
  pub act: Option<i64>,
  pub bck: Option<i64>,
  pub chkfail: Option<i64>,
  pub chkdown: Option<i64>,
  pub lastchg: Option<i64>,
  pub downtime: Option<i64>,
  pub qlimit: Option<i64>,
  pub pid: Option<i64>,
  pub iid: Option<i64>,
  pub sid: Option<i64>,
  pub throttle: Option<i64>,
  pub lbtot: Option<i64>,
  pub tracked: Option<i64>,
  pub r#type: Option<String>,
  pub rate: Option<f64>,
  pub rate_lim: Option<f64>,
  pub rate_max: Option<f64>,
  pub check_status: Option<String>,
  pub check_code: Option<String>,
  pub check_duration: Option<i64>,
  pub hrsp_1xx: Option<f64>,
  pub hrsp_2xx: Option<f64>,
  pub hrsp_3xx: Option<f64>,
  pub hrsp_4xx: Option<f64>,
  pub hrsp_5xx: Option<f64>,
  pub hrsp_other: Option<f64>,
  pub hanafail: Option<i64>,
  pub req_rate: Option<f64>,
  pub req_rate_max: Option<f64>,
  pub req_tot: Option<f64>,
  pub cli_abrt: Option<i64>,
  pub srv_abrt: Option<i64>,
  pub comp_in: Option<f64>,
  pub comp_out: Option<f64>,
  pub comp_byp: Option<f64>,
  pub comp_rsp: Option<f64>,
  pub lastsess: Option<i64>,
  pub last_chk: Option<String>,
  pub last_agt: Option<String>,
  pub qtime: Option<i64>,
  pub ctime: Option<i64>,
  pub rtime: Option<i64>,
  pub ttime: Option<i64>,
  pub agent_status: Option<String>,
  pub agent_code: Option<String>,
  pub agent_duration: Option<i64>,
  pub check_desc: Option<String>,
  pub agent_desc: Option<String>,
  pub check_rise: Option<i64>,
  pub check_fall: Option<i64>,
  pub check_health: Option<i64>,
  pub agent_rise: Option<i64>,
  pub agent_fall: Option<i64>,
  pub agent_health: Option<i64>,
  pub addr: Option<String>,
  pub cookie: Option<String>,
  pub mode: Option<String>,
  pub algo: Option<String>,
  pub conn_rate: Option<f64>,
  pub conn_rate_max: Option<f64>,
  pub conn_tot: Option<f64>,
  pub intercepted: Option<f64>,
  pub dcon: Option<i64>,
  pub dses: Option<i64>,
  pub wrew: Option<i64>,
  pub connect: Option<i64>,
  pub reuse: Option<i64>,
  pub cache_lookups: Option<f64>,
  pub cache_hits: Option<f64>,
  pub srv_icur: Option<i64>,
  pub src_ilim: Option<i64>,
  pub qtime_max: Option<i64>,
  pub ctime_max: Option<i64>,
  pub rtime_max: Option<i64>,
  pub ttime_max: Option<i64>,
  pub eint: Option<i64>,
  pub idle_conn_cur: Option<i64>,
  pub safe_conn_cur: Option<i64>,
  pub used_conn_cur: Option<i64>,
  pub need_conn_est: Option<i64>,
  pub uweight: Option<i64>,
}

impl HaproxyStat {
  pub fn parse_csv(csv: &str) -> Result<Vec<HaproxyStat>, csv::Error> {
    let mut records = Vec::new();
    log::debug!("csv: {}", csv);
    let mut rdr =
      csv::ReaderBuilder::new().has_headers(false).flexible(true).comment(Some(b'#')).from_reader(csv.as_bytes());
    for result in rdr.records() {
      let mut fields = result?;
      for _ in fields.len()..100 {
        fields.push_field("");
      }

      let stat: HaproxyStat = fields.deserialize(None)?;

      records.push(stat);
    }

    Ok(records)
  }
}

#[derive(Debug, PartialEq, Clone, Display)]
pub enum ResourceType {
  Frontend,
  Backend,
  Server,
}
