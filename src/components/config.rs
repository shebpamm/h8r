use crate::action::Action;
use crate::components::Component;
use crate::components::Rect;
use crate::config::Config;
use crate::tui::Frame;
use ansi_to_tui::IntoText;
use color_eyre::eyre::{Error, Result};
use ratatui::layout::Direction;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxDefinition, SyntaxSet, SyntaxSetBuilder};
use syntect::util::as_24_bit_terminal_escaped;
use std::time::Instant;

const HAPROXY_SYNTAX: &str = include_str!("../../.config/haproxy.sublime-syntax");

#[derive(Debug)]
struct HaproxyConfigSnippets {
    frontend: Vec<String>,
    backend: Vec<String>,
    acl: Vec<String>,
}

enum HaproxyDisplay {
    Lines(HaproxyConfigSnippets),
    Error(String),
}

#[derive(Debug, Default)]
pub struct ConfigView {
  config: Config,
  pid: Option<u32>,
  config_path: Option<String>,
  haproxy_config: Option<Vec<String>>,
  highlighted_config: Option<Vec<String>>,
  haproxy_parse_error: Option<Box<Error>>,
  selected_backend: Option<String>,
  parsed_snippets: Option<HaproxyConfigSnippets>,
}

impl ConfigView {
  fn find_config(&mut self) -> Result<()> {
    log::debug!("ConfigView::find_config: Starting config search for PID: {:?}", self.pid);
    let process_path = format!("/proc/{}/cmdline", self.pid.unwrap());
    let process_cwd = format!("/proc/{}/cwd", self.pid.unwrap());
    let process_cmdline = std::fs::read_to_string(process_path)?;
    let process_cmdline = process_cmdline.split("\0").collect::<Vec<&str>>();
    log::debug!("ConfigView::find_config: Process cmdline: {:?}", process_cmdline);
    
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
      log::debug!("ConfigView::find_config: Found config path flag: {}", config_path);
      // Resolve the path based on cwd
      let cwd = std::fs::read_link(process_cwd)?;
      let cwd = cwd.to_str().unwrap();
      let config_path = std::path::Path::new(cwd).join(config_path);
      let config_path = config_path.to_str().unwrap().to_string();
      self.config_path = Some(config_path.clone());
      log::debug!("ConfigView::find_config: Resolved config path: {}", config_path);

      // read the config
      self.read_config()?;
    } else {
      log::warn!("ConfigView::find_config: No -f flag found in cmdline");
    };

    log::debug!("ConfigView::find_config: Completed");
    Ok(())
  }

  fn read_config(&mut self) -> Result<()> {
    let start = Instant::now();
    log::debug!("ConfigView::read_config: Starting config read from: {:?}", self.config_path);
    
    // If the config is a directory, concatenate all files in alphabetic order, otherwise if
    // it's a file, read it directly
    let content = match std::fs::read_dir(&self.config_path.as_ref().unwrap()) {
      Ok(entries) => {
        log::debug!("ConfigView::read_config: Reading directory");
        let mut content = String::new();
        for entry in entries {
          let entry = entry?;
          let path = entry.path();
          let path = path.to_str().unwrap();
          log::debug!("ConfigView::read_config: Reading file: {}", path);
          let file_content = std::fs::read_to_string(path)?;
          content.push_str(&file_content);
        }
        content
      },
      Err(_) => {
        log::debug!("ConfigView::read_config: Reading single file");
        std::fs::read_to_string(&self.config_path.as_ref().unwrap())?
      },
    };

    log::debug!("ConfigView::read_config: Config content size: {} bytes", content.len());

    let syntax_start = Instant::now();
    let sd = SyntaxDefinition::load_from_str(HAPROXY_SYNTAX, false, None).unwrap();
    let mut ps_builder = SyntaxSetBuilder::new();
    ps_builder.add(sd);
    let ps = ps_builder.build();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_name("Haproxy").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    log::debug!("ConfigView::read_config: Syntax highlighting setup took: {:?}", syntax_start.elapsed());

    self.haproxy_config = Some(content.lines().map(|line| line.to_string()).collect());

    let is_inside_screen = std::env::var("TERM").unwrap_or_default().contains("screen");
    log::debug!("ConfigView::read_config: Terminal type (is_inside_screen): {}", is_inside_screen);

    let highlight_start = Instant::now();
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

    log::debug!("ConfigView::read_config: Syntax highlighting took: {:?}", highlight_start.elapsed());
    log::debug!("ConfigView::read_config: Total read_config took: {:?}", start.elapsed());

    Ok(())
  }

  fn parse_config_snippets(&mut self) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();
    log::debug!("ConfigView::parse_config_snippets: Starting config parsing for backend: {:?}", self.selected_backend);
    
    if let Some(ref config) = self.haproxy_config {
      log::debug!("ConfigView::parse_config_snippets: Config has {} lines", config.len());
      
      if let Some(ref selected_backend) = self.selected_backend {
        log::debug!("ConfigView::parse_config_snippets: Parsing for backend: {}", selected_backend);
        
        // find the backend section for current backend
        let mut backend = None;

        // Iterate through the lines, and when we find the line containing "backend
        // <name>", we start collecting lines until we hit the next backend or the end of
        // the file. The lines added come from self.highlighted_config but comparisons happen in
        // self.haproxy_config
        let mut backend_lines = vec![];

        let backend_search_start = Instant::now();
        for i in 0..config.len() {
          let line = &config[i];
          if line.starts_with("backend") && line.contains(selected_backend) {
            backend = Some(selected_backend);
            backend_lines.push(self.highlighted_config.as_ref().unwrap()[i].clone());
            log::debug!("ConfigView::parse_config_snippets: Found backend section at line {}", i);
            continue;
          }

          if let Some(ref backend) = backend {
            if line.starts_with("backend") {
              log::debug!("ConfigView::parse_config_snippets: End of backend section at line {}", i);
              break;
            }
            backend_lines.push(self.highlighted_config.as_ref().unwrap()[i].clone());
          }
        }
        log::debug!("ConfigView::parse_config_snippets: Backend search took: {:?}, found {} lines", 
                   backend_search_start.elapsed(), backend_lines.len());

        // find use_backend matching our backend
        let use_backend_search_start = Instant::now();
        let mut use_backend = None;
        let mut use_backend_highlighted = None;
        for i in 0..config.len() {
          let line = &config[i];
          if line.contains("use_backend") {
            if line.contains(selected_backend) {
              use_backend = Some(line.to_string());
              use_backend_highlighted = Some(self.highlighted_config.as_ref().unwrap()[i].clone());
              log::debug!("ConfigView::parse_config_snippets: Found use_backend at line {}: {}", i, line.trim());
              break;
            }
          }
        }
        log::debug!("ConfigView::parse_config_snippets: use_backend search took: {:?}", use_backend_search_start.elapsed());

        // parse acls used in use_backend line
        // example use_backend:
        // use_backend backend if acl1 acl2
        let acl_parse_start = Instant::now();
        let mut acls = vec![];
        if let Some(ref use_backend) = use_backend {
          let parts: Vec<&str> = use_backend.split_whitespace().collect();
          for i in 0..parts.len() {
            if parts[i] == "if" {
              for j in i+1..parts.len() {
                acls.push(parts[j].to_string());
              }
              break;
            }
          }
        }
        log::debug!("ConfigView::parse_config_snippets: Found ACLs: {:?}", acls);

        // find acl lines matching our acls
        let mut acl_lines = vec![];
        for i in 0..config.len() {
          let line = &config[i];
          for acl in &acls {
            if line.trim().starts_with(format!("acl {}", acl).as_str()) {
              acl_lines.push(self.highlighted_config.as_ref().unwrap()[i].clone());
              log::debug!("ConfigView::parse_config_snippets: Found ACL definition at line {}: {}", i, line.trim());
            }
          }
        }
        log::debug!("ConfigView::parse_config_snippets: ACL parsing took: {:?}, found {} ACL lines", 
                   acl_parse_start.elapsed(), acl_lines.len());

        self.parsed_snippets = Some(HaproxyConfigSnippets {
          frontend: vec!(use_backend_highlighted.unwrap_or("".to_string())),
          backend: backend_lines,
          acl: acl_lines,
        });
      } else {
        log::debug!("ConfigView::parse_config_snippets: No backend selected, creating empty snippets");
        self.parsed_snippets = Some(HaproxyConfigSnippets {
          frontend: vec![],
          backend: vec![],
          acl: vec![],
        });
      }
    } else {
      log::warn!("ConfigView::parse_config_snippets: No haproxy_config available");
    }
    
    log::debug!("ConfigView::parse_config_snippets: Total parsing took: {:?}", start.elapsed());
    Ok(())
  }

  fn ensure_config_loaded(&mut self) -> Result<()> {
    // If we already have parsed snippets or an error, we're done
    if self.parsed_snippets.is_some() || self.haproxy_parse_error.is_some() {
      return Ok(());
    }

    // If we don't have a PID yet, we can't load config
    if self.pid.is_none() {
      return Ok(());
    }

    log::debug!("ConfigView::ensure_config_loaded: Starting deferred config loading");
    
    match self.find_config() {
      Ok(_) => {
        self.parse_config_snippets()?;
        log::debug!("ConfigView::ensure_config_loaded: Config loaded and parsed successfully");
      },
      Err(e) => {
        log::error!("ConfigView::ensure_config_loaded: Config loading error: {}", e);
        self.haproxy_parse_error = Some(Box::new(e));
      },
    }
    
    Ok(())
  }
}

impl Component for ConfigView {
  fn init(&mut self, _rect: Rect) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();
    log::debug!("ConfigView::init: Starting lightweight initialization");
    
    let socket_path = &self.config.paths.socket;
    log::debug!("ConfigView::init: Connecting to socket: {}", socket_path);

    let mut stream = UnixStream::connect(socket_path)?;

    loop {
      match stream.write(b"show info\n") {
        Ok(_) => {
          log::debug!("ConfigView::init: Querying info");
        },
        Err(e) => {
          log::error!("ConfigView::init: Socket write error: {}", e);
          println!("Error: {}", e);
        },
      }

      let mut resp = String::new();
      let read_start = Instant::now();
      stream.read_to_string(&mut resp)?;
      log::debug!("ConfigView::init: Socket read took: {:?}, response size: {} bytes", 
                 read_start.elapsed(), resp.len());

      // find line that begins with Pid:
      let pid_line = resp.lines().find(|line| line.starts_with("Pid:"));

      if let Some(pid_line) = pid_line {
        let pid = pid_line.split(":").last().unwrap().trim();
        self.pid = Some(pid.parse().unwrap());
        log::debug!("ConfigView::init: Found HAProxy PID: {}", pid);
        
        // Don't do expensive config parsing here - let it happen later
        log::debug!("ConfigView::init: Deferring config parsing to allow UI to render");
        break;
      } else {
        log::warn!("ConfigView::init: No Pid line found in HAProxy info response");
      }

      break;
    }

    log::debug!("ConfigView::init: Lightweight initialization took: {:?}", start.elapsed());
    Ok(())
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();
    log::trace!("ConfigView::draw: Starting draw");
    
    // Ensure config is loaded (happens on first draw, allows Loading... to show initially)
    self.ensure_config_loaded()?;
    
    let content = match self.parsed_snippets {
      Some(ref snippets) => {
        log::trace!("ConfigView::draw: Using cached snippets - frontend: {}, backend: {}, acl: {}", 
                   snippets.frontend.len(), snippets.backend.len(), snippets.acl.len());
        HaproxyDisplay::Lines(HaproxyConfigSnippets {
          frontend: snippets.frontend.clone(),
          backend: snippets.backend.clone(),
          acl: snippets.acl.clone(),
        })
      },
      None => match self.haproxy_parse_error {
        Some(ref e) => {
          log::trace!("ConfigView::draw: Displaying error: {}", e);
          HaproxyDisplay::Error(format!("Error: {}", e))
        },
        None => {
          log::trace!("ConfigView::draw: Displaying loading message");
          HaproxyDisplay::Error("Loading...".to_string())
        },
      },
    };

    match content {
      HaproxyDisplay::Lines(snippets) => {
        let frontend_size = snippets.frontend.len();
        let acl_size = snippets.acl.len();

        let code_layout = Layout::default()
          .direction(Direction::Vertical)
          .constraints(vec![Constraint::Length(frontend_size as u16 + 2), Constraint::Length(acl_size as u16 + 2), Constraint::Min(0)])
          .split(rect);

        let frontend_frame = code_layout[0];
        let acl_frame = code_layout[1];
        let backend_frame = code_layout[2];

        let render_start = Instant::now();
        let frontend = Paragraph::new(snippets.frontend.join("\n").into_text()?)
          .block(Block::default().borders(Borders::ALL).title("Frontend"));
        f.render_widget(frontend, frontend_frame);

        let acl = Paragraph::new(snippets.acl.join("\n").into_text()?)
          .block(Block::default().borders(Borders::ALL).title("Acls"));
        f.render_widget(acl, acl_frame);

        let backend = Paragraph::new(snippets.backend.join("\n").into_text()?)
          .block(Block::default().borders(Borders::ALL).title("Backend"));
        f.render_widget(backend, backend_frame);
        
        log::trace!("ConfigView::draw: Widget rendering took: {:?}", render_start.elapsed());
      },
      HaproxyDisplay::Error(ref e) => {
        let error = Paragraph::new(e.clone())
          .block(Block::default().borders(Borders::ALL).title("Error"));
        f.render_widget(error, rect);
      },
    }

    let elapsed = start.elapsed();
    if elapsed.as_millis() > 1 {
      log::debug!("ConfigView::draw: Total draw took: {:?}", elapsed);
    } else {
      log::trace!("ConfigView::draw: Total draw took: {:?}", elapsed);
    }
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    log::debug!("ConfigView::register_config_handler: Registering config");
    self.config = config;
    Ok(())
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    log::trace!("ConfigView::update: Received action: {:?}", action);
    match action {
      Action::UseItem(backend_name) => {
        log::info!("ConfigView::update: Switching to backend: {}", backend_name);
        self.selected_backend = Some(backend_name);
        self.parse_config_snippets()?;
        Ok(None)
      },
      _ => {
        log::trace!("ConfigView::update: Ignoring action: {:?}", action);
        Ok(None)
      },
    }
  }
}
