use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AgentConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub extra_args: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub agents: HashMap<String, AgentConfig>,
}

fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("cocode").join("config.json")
}

pub fn load() -> Config {
    let path = config_path();
    if !path.exists() {
        return Config::default();
    }
    let raw = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save(cfg: &Config) -> std::io::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(cfg).unwrap();
    fs::write(path, json)
}

pub fn set_key(agent: &str, key: &str) {
    let mut cfg = load();
    let entry = cfg.agents.entry(agent.to_string()).or_default();
    entry.api_key = Some(key.to_string());
    save(&cfg).expect("failed to save config");
    println!("saved api key for {agent}");
}

pub fn set_model(agent: &str, model: &str) {
    let mut cfg = load();
    let entry = cfg.agents.entry(agent.to_string()).or_default();
    entry.model = Some(model.to_string());
    save(&cfg).expect("failed to save config");
    println!("saved model for {agent}: {model}");
}

pub fn show() {
    let cfg = load();
    if cfg.agents.is_empty() {
        println!("no config yet. run: cocode config set-key <agent> <key>");
        return;
    }
    for (name, ac) in &cfg.agents {
        println!("{}:", name);
        if let Some(k) = &ac.api_key {
            let masked = mask(k);
            println!("  api_key : {masked}");
        }
        if let Some(m) = &ac.model {
            println!("  model   : {m}");
        }
        if let Some(args) = &ac.extra_args {
            println!("  args    : {}", args.join(" "));
        }
    }
}

fn mask(s: &str) -> String {
    if s.len() <= 8 {
        return "***".to_string();
    }
    format!("{}...{}", &s[..4], &s[s.len() - 4..])
}
