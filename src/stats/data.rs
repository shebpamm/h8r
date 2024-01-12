use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, PartialOrd, Clone, Default)]
pub struct HaproxyStat {
    pub pxname: String,
    pub svname: String,
    pub qcur: i64,
    pub qmax: i64,
    pub scur: i64,
    pub smax: i64,
    pub slim: i64,
    pub stot: i64,
    pub bin: i64,
    pub bout: i64,
    pub dreq: i64,
    pub dresp: i64,
    pub ereq: i64,
    pub econ: i64,
    pub eresp: i64,
    pub wretr: i64,
    pub wredis: i64,
    pub status: String,
    pub weight: i64,
    pub act: i64,
    pub bck: i64,
    pub chkfail: i64,
    pub chkdown: i64,
    pub lastchg: i64,
    pub downtime: i64,
    pub qlimit: i64,
    pub pid: i64,
    pub iid: i64,
    pub sid: i64,
    pub throttle: i64,
    pub lbtot: i64,
    pub tracked: i64,
    pub r#type: String,
    pub rate: f64,
    pub rate_lim: f64,
    pub rate_max: f64,
    pub check_status: String,
    pub check_code: String,
    pub check_duration: i64,
    pub hrsp_1xx: f64,
    pub hrsp_2xx: f64,
    pub hrsp_3xx: f64,
    pub hrsp_4xx: f64,
    pub hrsp_5xx: f64,
    pub hrsp_other: f64,
    pub hanafail: i64,
    pub req_rate: f64,
    pub req_rate_max: f64,
    pub req_tot: f64,
    pub cli_abrt: i64,
    pub srv_abrt: i64,
    pub comp_in: f64,
    pub comp_out: f64,
    pub comp_byp: f64,
    pub comp_rsp: f64,
    pub lastsess: i64,
    pub last_chk: String,
    pub last_agt: String,
    pub qtime: i64,
    pub ctime: i64,
    pub rtime: i64,
    pub ttime: i64,
    pub agent_status: String,
    pub agent_code: String,
    pub agent_duration: i64,
    pub check_desc: String,
    pub agent_desc: String,
    pub check_rise: i64,
    pub check_fall: i64,
    pub check_health: i64,
    pub agent_rise: i64,
    pub agent_fall: i64,
    pub agent_health: i64,
    pub addr: String,
    pub cookie: String,
    pub mode: String,
    pub algo: String,
    pub conn_rate: f64,
    pub conn_rate_max: f64,
    pub conn_tot: f64,
    pub intercepted: f64,
    pub dcon: i64,
    pub dses: i64,
    pub wrew: i64,
    pub connect: i64,
    pub reuse: i64,
    pub cache_lookups: f64,
    pub cache_hits: f64,
    pub srv_icur: i64,
    pub src_ilim: i64,
    pub qtime_max: i64,
    pub ctime_max: i64,
    pub rtime_max: i64,
    pub ttime_max: i64,
    pub eint: i64,
    pub idle_conn_cur: i64,
    pub safe_conn_cur: i64,
    pub used_conn_cur: i64,
    pub need_conn_est: i64,
    pub uweight: i64,
}
