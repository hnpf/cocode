use crate::{config, telemetry::Filter};
use std::{
    env,
    io::{self, BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    thread,
};

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

pub fn spawn(name: &str, extra: &[String]) -> io::Result<Child> {
    let cfg = config::load();
    let ac = cfg.agents.get(name);

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

    // inherit all streams so isatty() passes in the child process.
    // agents like codex and agy check for a real tty and refuse to run
    // or send garbage escape sequences if stdout/stderr are pipes.
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

    cmd.spawn()
}

pub fn run(mut child: Child) -> io::Result<i32> {
    // with inherited streams the child drives the terminal directly;
    // just wait for it to exit
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
//             if f1.check(&line).is_some() {
//                 let _ = writeln!(out, "{line}");
//             }
//         }
//     });
//     let f2 = Filter::new();
//     let t2 = thread::spawn(move || {
//         let reader = BufReader::new(stderr);
//         let mut err = io::stderr();
//         for line in reader.lines().flatten() {
//             if f2.check(&line).is_some() {
//                 let _ = writeln!(err, "{line}");
//             }
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
