use crate::action::Action;
use crate::components::Component;
use crate::components::Rect;
use crate::config::Config;
use crate::tui::Frame;
use ansi_to_tui::IntoText;
use color_eyre::eyre::{Error, Result};
use ratatui::widgets::Paragraph;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxDefinition, SyntaxSet, SyntaxSetBuilder};
use syntect::util::as_24_bit_terminal_escaped;

const HAPROXY_SYNTAX: &str = include_str!("../../.config/haproxy.sublime-syntax");

#[derive(Debug, Default)]
pub struct ConfigView {
  config: Config,
  pid: Option<u32>,
  config_path: Option<String>,
  haproxy_config: Option<Vec<String>>,
  highlighted_config: Option<Vec<String>>,
  haproxy_parse_error: Option<Box<Error>>,
  selected_backend: Option<String>,
}

impl ConfigView {
  fn find_config(&mut self) -> Result<()> {
    let process_path = format!("/proc/{}/cmdline", self.pid.unwrap());
    let process_cwd = format!("/proc/{}/cwd", self.pid.unwrap());
    let process_cmdline = std::fs::read_to_string(process_path)?;
    let process_cmdline = process_cmdline.split("\0").collect::<Vec<&str>>();
    // find -f flag
    let mut config_path = None;
    for (i, arg) in process_cmdline.iter().enumerate() {
      if *arg == "-f" {
        config_path = Some(process_cmdline[i + 1]);
        break;
      }
    }

    if let Some(config_path) = config_path {
      let raw_path = Some(config_path.to_string());
      // Resolve the path based on cwd
      let cwd = std::fs::read_link(process_cwd)?;
      let cwd = cwd.to_str().unwrap();
      let config_path = std::path::Path::new(cwd).join(config_path);
      let config_path = config_path.to_str().unwrap().to_string();
      self.config_path = Some(config_path);

      // read the config
      self.read_config()?;
    };

    Ok(())
  }

  fn read_config(&mut self) -> Result<()> {
    // If the config is a directory, concatenate all files in alphabetic order, otherwise if
    // it's a file, read it directly
    let content = match std::fs::read_dir(&self.config_path.as_ref().unwrap()) {
      Ok(entries) => {
        let mut content = String::new();
        for entry in entries {
          let entry = entry?;
          let path = entry.path();
          let path = path.to_str().unwrap();
          let file_content = std::fs::read_to_string(path)?;
          content.push_str(&file_content);
        }
        content
      },
      Err(_) => std::fs::read_to_string(&self.config_path.as_ref().unwrap())?,
    };

    let sd = SyntaxDefinition::load_from_str(HAPROXY_SYNTAX, false, None).unwrap();
    let mut ps_builder = SyntaxSetBuilder::new();
    ps_builder.add(sd);
    let ps = ps_builder.build();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_name("Haproxy").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    self.haproxy_config = Some(content.lines().map(|line| line.to_string()).collect());

    let is_inside_screen = std::env::var("TERM").unwrap_or_default().contains("screen");

    match is_inside_screen {
      true => {
        self.highlighted_config = self.haproxy_config.clone();
      },
      false => {
        let lines = content
          .split("\n")
          .map(|line| {
            let ranges = h.highlight_line(line, &ps).unwrap();
            let escaped = as_24_bit_terminal_escaped(&ranges[..], true);

            escaped
          })
          .collect();

        self.highlighted_config = Some(lines);
      },
    }

    Ok(())
  }
}

impl Component for ConfigView {
  fn init(&mut self, _rect: Rect) -> Result<()> {
    let socket_path = &self.config.config._socket_path;

    let mut stream = UnixStream::connect(socket_path)?;

    loop {
      match stream.write(b"show info\n") {
        Ok(_) => {
          log::debug!("Querying info");
        },
        Err(e) => {
          println!("Error: {}", e);
        },
      }

      let mut resp = String::new();
      stream.read_to_string(&mut resp)?;

      // find line that begins with Pid:
      let pid_line = resp.lines().find(|line| line.starts_with("Pid:"));

      if let Some(pid_line) = pid_line {
        let pid = pid_line.split(":").last().unwrap().trim();
        self.pid = Some(pid.parse().unwrap());
        match self.find_config() {
          Ok(_) => {
            break;
          },
          Err(e) => {
            self.haproxy_parse_error = Some(Box::new(e));
            break;
          },
        }
      }

      break;
    }

    Ok(())
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let content = match self.haproxy_config {
      Some(ref config) => {
        // find the backend section for current backend
        let mut backend = None;

        // Iterate through the lines, and when we find the line containing "backend
        // <name>", we start collecting lines until we hit the next backend or the end of
        // the file. The lines added come from self.highlighted_config but comparisons happen in
        // self.haproxy_config
        let mut lines = vec![];

        for i in 0..config.len() {
          let line = &config[i];
          if let Some(ref selected_backend) = self.selected_backend {
            if line.starts_with("backend") && line.contains(selected_backend) {
              backend = Some(selected_backend);
              lines.push(self.highlighted_config.as_ref().unwrap()[i].clone());
              continue;
            }
          }

          if let Some(ref backend) = backend {
            if line.starts_with("backend") {
              break;
            }
            lines.push(self.highlighted_config.as_ref().unwrap()[i].clone());
          }
        }

        lines.join("\n").into_text()?
      },
      None => match self.haproxy_parse_error {
        Some(ref e) => format!("Error: {}", e).into_text()?,
        None => "Loading...".into_text()?,
      },
    };
    let text = Paragraph::new(content);
    f.render_widget(text, rect);

    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::UseItem(backend_name) => {
        self.selected_backend = Some(backend_name);
        Ok(None)
      },
      _ => Ok(None),
    }
  }
}
