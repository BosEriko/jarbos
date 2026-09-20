#!/bin/sh
set -e

CLAUDE_SETTINGS="$HOME/.claude/settings.json"
CODEX_HOOKS="$HOME/.codex/hooks.json"
COMMAND="mkdir -p \"$HOME/Library/Application Support/ph.eriko.jarbos/hook-events\" && cat > \"$HOME/Library/Application Support/ph.eriko.jarbos/hook-events/\$(date +%s%N)-\$\$.json\""

node - "$CLAUDE_SETTINGS" "$COMMAND" <<'NODEEOF'
const fs = require("fs");
const path = require("path");

const [, , settingsPath, command] = process.argv;
const hookEntry = { type: "command", command };

let data = {};
if (fs.existsSync(settingsPath)) {
  const content = fs.readFileSync(settingsPath, "utf8").trim();
  if (content) {
    data = JSON.parse(content);
  }
}

data.hooks = data.hooks || {};

function ensure(event, withMatcher) {
  const entries = data.hooks[event] || (data.hooks[event] = []);
  for (const entry of entries) {
    for (const h of entry.hooks || []) {
      if (h.command === command) {
        return;
      }
    }
  }
  const entry = { hooks: [hookEntry] };
  if (withMatcher) {
    entry.matcher = "*";
  }
  entries.push(entry);
}

ensure("Notification", false);
ensure("PreToolUse", true);
ensure("UserPromptSubmit", false);
ensure("Stop", false);

fs.mkdirSync(path.dirname(settingsPath), { recursive: true });
fs.writeFileSync(settingsPath, JSON.stringify(data, null, 2) + "\n");
NODEEOF

node - "$CODEX_HOOKS" "$COMMAND" <<'NODEEOF'
const fs = require("fs");
const path = require("path");

const [, , hooksPath, command] = process.argv;
const hookEntry = { type: "command", command };

let data = {};
if (fs.existsSync(hooksPath)) {
  const content = fs.readFileSync(hooksPath, "utf8").trim();
  if (content) {
    data = JSON.parse(content);
  }
}

function ensure(event, withMatcher) {
  const entries = data[event] || (data[event] = []);
  for (const entry of entries) {
    for (const h of entry.hooks || []) {
      if (h.command === command) {
        return;
      }
    }
  }
  const entry = { hooks: [hookEntry] };
  if (withMatcher) {
    entry.matcher = "*";
  }
  entries.push(entry);
}

ensure("PermissionRequest", false);
ensure("PreToolUse", true);
ensure("UserPromptSubmit", false);
ensure("Stop", false);

fs.mkdirSync(path.dirname(hooksPath), { recursive: true });
fs.writeFileSync(hooksPath, JSON.stringify(data, null, 2) + "\n");
NODEEOF
