# cocode

a universal multiplexer, manager, and wrapper for terminal-based coding agents; `claude`, `agy`, `codex`, `kimi`.

```
cocode                           # sexy TUI picker
cocode claude                    # launch claude directly (and other agents)
cocode agy --model gemini-pro    # launch with extra args
```

---

## install

### one-liner installation (via curl)

> requires `git` and a Rust toolchain ([rustup.rs](https://rustup.rs))

```sh
curl -fsSL https://raw.githubusercontent.com/hnpf/cocode/main/install.sh | bash
```

this clones the repo to a temp directory, builds a release binary, and installs it to `~/.local/bin`.

If `~/.local/bin` isn't in your `PATH`, add this to your `~/.bashrc` or `~/.zshrc`:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

then reload your shell:

```sh
source ~/.bashrc   # or ~/.zshrc
```

### from source (local)

```sh
git clone https://github.com/hnpf/cocode
cd cocode
bash install.sh
```

### shortcuts

want the wait even shorter? add `coco` symlink during installation! this is how to do so:

```sh
bash install.sh --shortcuts
```

for the curl install, pass the option through to Bash:

```sh
curl -fsSL https://raw.githubusercontent.com/hnpf/cocode/main/install.sh | bash -s -- --shortcuts
```

the shortcut behaves exactly like `cocode` (for example, `coco claude`)!

### via cargo

```sh
cargo install --path .
```

---

## agents

| name   | binary | provider          |
|--------|--------|-------------------|
| claude | claude | Anthropic         |
| agy    | agy    | Google Antigravity|
| codex  | codex  | OpenAI            |
| kimi   | kimi   | Moonshot AI       |

the agent binaries must already be installed and on your `$PATH`. cocode just manages and wraps them.

---

## config

APIkeys and model prefs are stored in `~/.config/cocode/config.json`.

```sh
cocode config set-key   claude  sk-ant-...
cocode config set-key   agy     AIza...
cocode config set-key   codex   sk-...
cocode config set-key   kimi    sk-...

cocode config set-model claude  claude-opus-4-5
cocode config set-model agy     gemini-2.5-pro

cocode config show
```

APIkeys are injected as env vars (`ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`, etc.) only when not already set, your shell env wins.

---

## context migration

when an agent hits a rate limit or fails mid-session:

```sh
# 1. paste your context (ctrl+d to finish)
cocode ctx capture claude-1

# 2. migrate it to another agent
cocode ctx migrate claude-1 agy-1

# 3. resume in the new agent
cocode agy

# other ctx commands
cocode ctx dump claude-1    # print a saved session
cocode ctx list             # list all sessions
```

sessions are plain text files under `~/.local/share/cocode/`.

---

## telemetry filtering

stdout and stderr from child agents are piped through a filter that silently drops lines matching known telemetry/analytics patterns (sentry, posthog, amplitude, mixpanel, datadog, etc.) before they reach your terminal.

---

## project codebase structure (really simple to navigate!)

```
src/
├── main.rs       # cli dispatch
├── config.rs     # config read/write (~/.config/cocode/config.json)
├── agent.rs      # spawn + pipe + telemetry filter
├── telemetry.rs  # regex patterns for known analytics
├── migrate.rs    # context save / load / migrate
└── tui.rs        # arrow-key agent picker
install.sh        # curl-friendly installer
```
