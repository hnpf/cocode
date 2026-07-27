use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

// context is stored as plain text under ~/.local/share/cocode/<session>.txt
fn ctx_path(name: &str) -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("cocode").join(format!("{name}.txt"))
}

pub fn save_context(session: &str, text: &str) -> io::Result<()> {
    let path = ctx_path(session);
    if let Some(p) = path.parent() {
        fs::create_dir_all(p)?;
    }
    fs::write(path, text)
}

pub fn load_context(session: &str) -> io::Result<String> {
    fs::read_to_string(ctx_path(session))
}

// migrate: read context from src agent session, write it into a new file for dst
pub fn migrate(src: &str, dst: &str) -> io::Result<()> {
    let ctx = load_context(src)?;
    save_context(dst, &ctx)?;
    println!("migrated context from '{src}' to '{dst}'");
    println!("context saved at: {}", ctx_path(dst).display());
    Ok(())
}

// print a summary of stored sessions
pub fn list_sessions() {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cocode");

    let Ok(entries) = fs::read_dir(&base) else {
        println!("no saved sessions found");
        return;
    };

    let mut found = false;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "txt").unwrap_or(false) {
            let name = path.file_stem().unwrap().to_string_lossy();
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            println!("  {name}  ({size} bytes)");
            found = true;
        }
    }
    if !found {
        println!("no saved sessions found");
    }
}

// interactive: paste context from stdin into a named session
pub fn capture(session: &str) {
    println!("paste context for session '{session}', then press ctrl+d:");
    let mut buf = String::new();
    let stdin = io::stdin();
    loop {
        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => buf.push_str(&line),
            Err(_) => break,
        }
    }
    save_context(session, &buf).expect("failed to save context");
    println!("saved {} bytes to session '{session}'", buf.len());
}

pub fn dump(session: &str) {
    match load_context(session) {
        Ok(ctx) => {
            let mut out = io::stdout();
            let _ = out.write_all(ctx.as_bytes());
        }
        Err(_) => eprintln!("session '{session}' not found"),
    }
}
