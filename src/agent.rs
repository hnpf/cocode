use crate::{config, telemetry::Filter};
use std::{
    env,
    io::{self, BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    thread,
};

// maps agent name to its binary
pub fn binary(name: &str) -> &str {
    match name {
        "claude" => "claude",
        "agy" | "antigravity" => "agy",
        "codex" => "codex",
        "kimi" => "kimi",
        _ => name,
    }
}

pub fn known_agents() -> Vec<(&'static str, &'static str)> {
    vec![
        ("claude", "Anthropic Claude Code — agentic coding in the terminal"),
        ("agy", "Google Antigravity (agy) — Gemini-powered coding agent"),
        ("codex", "OpenAI Codex CLI — lightweight coding agent"),
        ("kimi", "Moonshot Kimi — multilingual coding agent"),
    ]
}

// inject api key and model as env vars the agent expects, then exec
pub fn spawn(name: &str, extra: &[String]) -> io::Result<Child> {
    let cfg = config::load();
    let ac = cfg.agents.get(name);

    let bin = binary(name);
    let mut cmd = Command::new(bin);

    // pass through the user's own args after the agent name
    cmd.args(extra);

    // append stored extra_args from config
    if let Some(ac) = ac {
        if let Some(ref args) = ac.extra_args {
            cmd.args(args);
        }
        // each agent has its own env var convention; cover common ones
        if let Some(ref key) = ac.api_key {
            set_key_env(&mut cmd, name, key);
        }
        if let Some(ref model) = ac.model {
            set_model_env(&mut cmd, name, model);
        }
    }

    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    cmd.spawn()
}

// run the child, pipe stdout/stderr through the telemetry filter, wait for exit
pub fn run(mut child: Child) -> io::Result<i32> {
    let filter = Filter::new();

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // stdout thread
    let f1 = Filter::new();
    let t1 = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        let mut out = io::stdout();
        for line in reader.lines().flatten() {
            if f1.check(&line).is_some() {
                let _ = writeln!(out, "{line}");
            }
        }
    });

    // stderr thread
    let f2 = filter;
    let t2 = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        let mut err = io::stderr();
        for line in reader.lines().flatten() {
            if f2.check(&line).is_some() {
                let _ = writeln!(err, "{line}");
            }
        }
    });

    let status = child.wait()?;
    let _ = t1.join();
    let _ = t2.join();

    Ok(status.code().unwrap_or(1))
}

fn set_key_env(cmd: &mut Command, agent: &str, key: &str) {
    let var = match agent {
        "claude" => "ANTHROPIC_API_KEY",
        "agy" | "antigravity" => "GOOGLE_API_KEY",
        "codex" => "OPENAI_API_KEY",
        "kimi" => "MOONSHOT_API_KEY",
        _ => return,
    };
    // don't overwrite if the user already has it set
    if env::var(var).is_err() {
        cmd.env(var, key);
    }
}

fn set_model_env(cmd: &mut Command, agent: &str, model: &str) {
    let var = match agent {
        "claude" => "CLAUDE_MODEL",
        "agy" | "antigravity" => "GOOGLE_MODEL",
        "codex" => "OPENAI_MODEL",
        "kimi" => "KIMI_MODEL",
        _ => return,
    };
    if env::var(var).is_err() {
        cmd.env(var, model);
    }
}
