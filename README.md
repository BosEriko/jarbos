# jarbos

Desktop companion that shows small animated avatars for AI coding agents (Claude Code, Codex CLI) along the bottom of the screen. Built with Tauri (Rust backend) + vanilla TypeScript frontend.

## How it works

- Rust polls running system processes every second (via `sysinfo`) and matches them against configured agent process names (`claude`, `codex`).
- Process start/stop transitions drive an agent state machine (`Active` → `Sleeping` → `Hidden`) in Rust.
- State changes are pushed to the frontend via Tauri events (plus a query command to hydrate current state on startup); the frontend renders/animates placeholder avatars in a transparent, click-through, always-on-top overlay window at the bottom of the screen.

## Prerequisites

- Node.js (LTS) and pnpm (`corepack enable`, or `npm i -g pnpm`)
- Rust toolchain via [rustup](https://rustup.rs)
- Platform build tools:
  - macOS: Xcode Command Line Tools (`xcode-select --install`)
  - Windows: Microsoft Visual Studio C++ Build Tools + WebView2 (see [Tauri prerequisites](https://tauri.app/start/prerequisites/))

## Development

```bash
pnpm install
pnpm tauri dev
```

This starts Vite (frontend, hot-reload) and builds/runs the Rust backend, opening the overlay window. Since `claude`/`codex` processes are detected system-wide (not tied to a specific terminal), you can test by opening a separate terminal and running `claude` or `codex` normally, then watching the overlay.

### Checks

Run these before committing changes:

```bash
# Rust
cd src-tauri
cargo build
cargo test
cargo clippy --all-targets
cargo fmt --check      # `cargo fmt` to fix formatting

# Frontend
cd ..
pnpm exec tsc --noEmit
```

### Project layout

- `src-tauri/src/agent.rs` — agent config + `AgentState` enum (add new agents here)
- `src-tauri/src/process_monitor.rs` — polls OS processes, emits start/stop events
- `src-tauri/src/state_manager.rs` — turns process events into agent state transitions (sleep/hide timing, restart-cancels-hide)
- `src-tauri/src/window.rs` — overlay window sizing/positioning/click-through setup
- `src-tauri/src/lib.rs` — wires everything together: window setup, tray icon, Tauri commands/events
- `src/config.ts` — frontend display config per agent (name, glyph, accent color) — **keep in sync with `agent.rs`'s `default_agents()`**
- `src/movement.ts` / `src/avatar.ts` / `src/avatarManager.ts` — walking/pausing behavior and DOM/CSS rendering
- `src/agentStore.ts` — Tauri event listener + initial-state fetch

### Adding a new agent

1. Add an entry to `default_agents()` in `src-tauri/src/agent.rs` (id, name, process name(s)).
2. Add a matching entry to `AGENT_CONFIGS` in `src/config.ts` (id, name, glyph, accent color).

No other code changes are needed — process detection, state management, and rendering are all driven by these two config lists.

## Building a release binary

```bash
pnpm tauri build
```

Output (paths may vary slightly by Tauri version):

- macOS: `src-tauri/target/release/bundle/macos/jarbos.app` (and a `.dmg`)
- Windows: `src-tauri/target/release/bundle/msi/*.msi` and/or `src-tauri/target/release/bundle/nsis/*.exe`

Use the **built app**, not `pnpm tauri dev`, for anything below.

## Running it at login (autostart) on a new machine

The app doesn't manage its own autostart yet (see note at the end), so it's configured through the OS.

### macOS

**Option A — System Settings (simplest):**

1. Build the app and drag `jarbos.app` into `/Applications`.
2. System Settings → General → Login Items → click **+** → select `jarbos.app`.

**Option B — LaunchAgent (scriptable, no GUI needed):**

Create `~/Library/LaunchAgents/ph.eriko.jarbos.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>ph.eriko.jarbos</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Applications/jarbos.app/Contents/MacOS/jarbos</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>
```

Then load it:

```bash
launchctl load ~/Library/LaunchAgents/ph.eriko.jarbos.plist
```

To stop autostarting, run `launchctl unload ~/Library/LaunchAgents/ph.eriko.jarbos.plist` and delete the file (or just remove it from Login Items if you used Option A).

### Windows

**Option A — Startup folder (simplest):**

1. Build the app (`pnpm tauri build`) and install it, or copy the built `.exe` somewhere permanent.
2. Press `Win + R`, type `shell:startup`, press Enter.
3. Create a shortcut to `jarbos.exe` inside that folder.

**Option B — Task Scheduler (more control, e.g. delay after login):**

```powershell
schtasks /create /tn "jarbos" /tr "C:\Path\To\jarbos.exe" /sc onlogon /rl limited
```

To remove: `schtasks /delete /tn "jarbos"` (Option B) or delete the shortcut from the Startup folder (Option A).

### Future improvement

For a more polished, cross-platform autostart (a toggle inside the app itself, no manual OS steps), consider adding [`tauri-plugin-autostart`](https://v2.tauri.app/plugin/autostart/) — it wraps the mechanisms above behind one Rust/JS API. Not added yet since it wasn't part of the original v1 scope.

## Known limitations (v1)

- Placeholder emoji avatars, not real sprite sheets yet (see `assets/agents/<id>/*.png` convention mentioned in the original spec — not wired up).
- Verified on macOS only; Windows code paths are written to be cross-platform but untested there.
- No drag-and-drop interaction with avatars.
- Only `active`/`sleeping`/`hidden` states are driven (from process existence); `waiting`/`success`/`error` exist in the `AgentState` type but nothing produces them yet.
- Single-monitor only; multi-monitor support is not implemented.
