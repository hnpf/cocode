use crate::config;
use std::{
    env,
    io,
    process::{Child, Command, Stdio},
};

// imports used by the commented-out telemetry-filtered path:
// use crate::telemetry::Filter;
// use std::io::{BufRead, BufReader, Write};
// use std::thread;

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

// how to install each agent if the binary isn't on PATH
fn install_hint(name: &str) -> Option<&'static str> {
    match name {
        "claude" => Some("npm install -g @anthropic-ai/claude-code"),
        "agy" => Some("npm install -g @google/agy  OR  pip install antigravity-cli"),
        "codex" => Some("npm install -g @openai/codex"),
        "kimi" => Some("pip install kimi-cli"),
        _ => None,
    }
}

pub fn spawn(name: &str, extra: &[String]) -> io::Result<Child> {
    let cfg = config::load();
    let ac = cfg.agents.get(name);

    // warn if no api key is configured and the env var isn't set either
    let key_var = match name {
        "claude" => Some("ANTHROPIC_API_KEY"),
        "agy" | "antigravity" => Some("GOOGLE_API_KEY"),
        "codex" => Some("OPENAI_API_KEY"),
        "kimi" => Some("MOONSHOT_API_KEY"),
        _ => None,
    };
    if let Some(var) = key_var {
        let has_env = env::var(var).is_ok();
        let has_cfg = ac.as_ref().and_then(|a| a.api_key.as_ref()).is_some();
        if !has_env && !has_cfg {
            eprintln!("cocode: warning: no api key for '{name}' (set with: cocode config set-key {name} <key>)");
        }
    }

    let bin = binary(name);
    let mut cmd = Command::new(bin);
    cmd.args(extra);

    if let Some(ac) = ac {
        if let Some(ref args) = ac.extra_args {
            cmd.args(args);
        }
        if let Some(ref key) = ac.api_key {
            set_key_env(&mut cmd, name, key);
        }
        if let Some(ref model) = ac.model {
            set_model_env(&mut cmd, name, model);
        }
    }

    // inherit all streams so isatty() passes in the child process
    //
    // telemetry filtering via piped streams is left below for future use
    // with a pty (e.g. portable-pty crate), which would satisfy isatty()
    // while still allowing interception:
    //
    // cmd.stdin(Stdio::inherit())
    //    .stdout(Stdio::piped())
    //    .stderr(Stdio::piped());
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    cmd.spawn().map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            if let Some(hint) = install_hint(name) {
                eprintln!("cocode: '{bin}' not found. install it with:");
                eprintln!("  {hint}");
            }
        }
        e
    })
}

pub fn run(mut child: Child) -> io::Result<i32> {
    let status = child.wait()?;
    Ok(status.code().unwrap_or(1))
}

// --- telemetry-filtered run (needs pty to satisfy isatty) ---
//
// pub fn run_filtered(mut child: Child) -> io::Result<i32> {
//     let stdout = child.stdout.take().unwrap();
//     let stderr = child.stderr.take().unwrap();
//     let f1 = Filter::new();
//     let t1 = thread::spawn(move || {
//         let reader = BufReader::new(stdout);
//         let mut out = io::stdout();
//         for line in reader.lines().flatten() {
//             if f1.check(&line).is_some() { let _ = writeln!(out, "{line}"); }
//         }
//     });
//     let f2 = Filter::new();
//     let t2 = thread::spawn(move || {
//         let reader = BufReader::new(stderr);
//         let mut err = io::stderr();
//         for line in reader.lines().flatten() {
//             if f2.check(&line).is_some() { let _ = writeln!(err, "{line}"); }
//         }
//     });
//     let status = child.wait()?;
//     let _ = t1.join();
//     let _ = t2.join();
//     Ok(status.code().unwrap_or(1))
// }

fn set_key_env(cmd: &mut Command, agent: &str, key: &str) {
    let var = match agent {
        "claude" => "ANTHROPIC_API_KEY",
        "agy" | "antigravity" => "GOOGLE_API_KEY",
        "codex" => "OPENAI_API_KEY",
        "kimi" => "MOONSHOT_API_KEY",
        _ => return,
    };
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
