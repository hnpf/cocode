mod agent;
mod config;
mod migrate;
mod telemetry;
mod tui;

use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    match refs.as_slice() {
        // bare invocation — show TUI picker
        [] => match tui::pick_agent() {
            Ok(Some(name)) => launch(&name, &[]),
            Ok(None) => {}
            Err(e) => die(&format!("tui error: {e}")),
        },

        // known agent
        [name, rest @ ..] if agent::known_agents().iter().any(|(n, _)| n == name) => {
            let extra: Vec<String> = rest.iter().map(|s| s.to_string()).collect();
            launch(name, &extra);
        }

        [cmd, rest @ ..] if *cmd == "config" => {
            let rest: Vec<String> = rest.iter().map(|s| s.to_string()).collect();
            handle_config(&rest);
        }

        [cmd, rest @ ..] if *cmd == "ctx" => {
            let rest: Vec<String> = rest.iter().map(|s| s.to_string()).collect();
            handle_ctx(&rest);
        }

        ["help" | "--help" | "-h", ..] => print_help(),

        // passthrough unknown binary
        [name, rest @ ..] => {
            let extra: Vec<String> = rest.iter().map(|s| s.to_string()).collect();
            launch(name, &extra);
        }
    }
}

fn launch(name: &str, extra: &[String]) {
    match agent::spawn(name, extra) {
        Ok(child) => {
            let code = agent::run(child).unwrap_or(1);
            process::exit(code);
        }
        Err(e) => die(&format!("could not launch '{name}': {e}")),
    }
}

fn handle_config(args: &[String]) {
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    match refs.as_slice() {
        ["set-key", agent, key] => config::set_key(agent, key),
        ["set-model", agent, model] => config::set_model(agent, model),
        ["show"] => config::show(),
        _ => {
            eprintln!("usage:");
            eprintln!("  cocode config set-key <agent> <key>");
            eprintln!("  cocode config set-model <agent> <model>");
            eprintln!("  cocode config show");
        }
    }
}

fn handle_ctx(args: &[String]) {
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    match refs.as_slice() {
        ["migrate", src, dst] => {
            migrate::migrate(src, dst).unwrap_or_else(|e| eprintln!("migrate error: {e}"));
        }
        ["capture", session] => migrate::capture(session),
        ["dump", session] => migrate::dump(session),
        ["list"] => migrate::list_sessions(),
        _ => {
            eprintln!("usage:");
            eprintln!("  cocode ctx migrate <src> <dst>");
            eprintln!("  cocode ctx capture <session>");
            eprintln!("  cocode ctx dump    <session>");
            eprintln!("  cocode ctx list");
        }
    }
}

fn print_help() {
    println!("cocode — multiplexer for terminal coding agents\n");
    println!("usage:");
    println!("  cocode                           pick agent interactively");
    println!("  cocode <agent> [args...]         launch agent directly");
    println!("  cocode config set-key  <agent> <key>");
    println!("  cocode config set-model <agent> <model>");
    println!("  cocode config show");
    println!("  cocode ctx capture <session>     save context from stdin");
    println!("  cocode ctx dump    <session>     print saved context");
    println!("  cocode ctx migrate <src> <dst>   copy context between agents");
    println!("  cocode ctx list                  list saved sessions");
    println!("\nagents: claude  agy  codex  kimi");
}

fn die(msg: &str) -> ! {
    eprintln!("cocode: {msg}");
    process::exit(1);
}
