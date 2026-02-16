# claudeline

A high-performance, customizable status line for [Claude Code](https://claude.ai/code) CLI.

Built in Rust. Zero runtime dependencies. Sub-millisecond rendering.

```
[Opus] ▓▓▓▓░░░░░░ 42% | $0.08 | 5m 23s
📁 my-project | 🌿 main +3 ~2 | +156 -23 | v2.1.31
```

## Why claudeline?

| | claudeline | ccstatusline |
|---|---|---|
| **Language** | Rust (compiled binary) | TypeScript (bunx/npx) |
| **Render time** | <1ms | ~200ms (npx overhead) |
| **Binary size** | 1.0 MB | N/A (requires Node.js) |
| **Runtime deps** | None | Node.js or Bun |
| **Data source** | Native JSON API (stdin) | Transcript file parsing |
| **Accuracy** | Always correct (official API) | Breaks across models/versions |
| **Memory** | 1.2 MB | ~50 MB (Node.js runtime) |
| **Widgets** | 25 | ~15 |
| **Config format** | TOML (with comments) | JSON |

## Quick Start

### Install

```bash
# npm (downloads platform binary automatically)
npm install -g claudeline

# Or direct binary
curl -fsSL https://raw.githubusercontent.com/mstuart/claudeline/main/scripts/install.sh | sh

# Or build from source
cargo install --path .
```

### Configure Claude Code

Add to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "claudeline"
  }
}
```

Restart Claude Code. Done.

### Apply a preset

```bash
claudeline preset full       # Two-line layout with everything
claudeline preset minimal    # Just model + context %
claudeline preset powerline  # Full layout with powerline arrows
claudeline preset compact    # Single line, compact values
```

## Widgets

25 built-in widgets, all reading from Claude Code's native JSON API:

### Core Metrics
| Widget | Type | Description |
|--------|------|-------------|
| Model | `model` | Current model name (Opus, Sonnet, etc.) |
| Context % | `context-percentage` | Context window usage with optional progress bar |
| Context Length | `context-length` | Absolute token count (e.g., "42K") |
| Tokens In | `tokens-input` | Input tokens from current usage |
| Tokens Out | `tokens-output` | Output tokens |
| Tokens Cached | `tokens-cached` | Cache creation + read tokens |
| Tokens Total | `tokens-total` | All tokens combined |
| Session Cost | `session-cost` | Running cost in USD with optional burn rate |
| Session Duration | `session-duration` | Elapsed time with optional API ratio |
| Block Timer | `block-timer` | 5-hour usage block tracker with progress bar |

### Git Integration
| Widget | Type | Description |
|--------|------|-------------|
| Branch | `git-branch` | Current branch (with detached HEAD support) |
| Status | `git-status` | Staged/modified/untracked file counts |
| Worktree | `git-worktree` | Active worktree name (hidden when not in worktree) |

### Workspace
| Widget | Type | Description |
|--------|------|-------------|
| CWD | `cwd` | Current directory (basename, full, fish-style) |
| Lines Changed | `lines-changed` | Lines added/removed this session |
| Version | `version` | Claude Code version |
| Session ID | `session-id` | Truncated session identifier |

### Advanced
| Widget | Type | Description |
|--------|------|-------------|
| Vim Mode | `vim-mode` | NORMAL/INSERT (hidden when vim mode off) |
| Agent Name | `agent-name` | Active agent (hidden when not using --agent) |
| Output Style | `output-style` | Current output style (hidden when "default") |
| Exceeds 200K | `exceeds-tokens` | Warning when tokens exceed 200K threshold |
| API Duration | `api-duration` | Ratio of API wait time to total time |
| Custom Command | `custom-command` | Run any shell command, display output |
| Custom Text | `custom-text` | Static text with emoji support |
| Separator | `separator` | Visual divider between widgets |
| Terminal Width | `terminal-width` | Current terminal width in columns |

## Configuration

Config lives at `~/.config/claudeline/config.toml`. Generate a default:

```bash
claudeline init
```

### Example config

```toml
theme = "dracula"
default_separator = " | "
default_padding = " "
flex_mode = "full-minus-40"
compact_threshold = 60
global_bold = false
inherit_separator_colors = false

# First status line
[[lines]]
type = "model"
color = "cyan"
raw_value = true

[[lines]]
type = "context-percentage"
metadata = { bar = "true" }

[[lines]]
type = "session-cost"
color = "yellow"
raw_value = true

[[lines]]
type = "session-duration"
raw_value = true

# Second status line (start a new line group)
[[lines]]
type = "cwd"
metadata = { fish_style = "true" }

[[lines]]
type = "git-branch"
color = "magenta"

[[lines]]
type = "git-status"

[powerline]
enabled = false
separator = "\uE0B0"
auto_align = false
```

### Widget options

Every widget supports:

| Option | Type | Description |
|--------|------|-------------|
| `type` | string | Widget type (see table above) |
| `color` | string | Foreground color (named, hex, or 256-color index) |
| `background_color` | string | Background color |
| `bold` | bool | Bold text |
| `raw_value` | bool | Compact mode without labels |
| `padding` | string | Override default padding |
| `merge_next` | bool | Merge with next widget (no separator) |
| `metadata` | table | Widget-specific options |

### Widget-specific metadata

| Widget | Key | Values | Description |
|--------|-----|--------|-------------|
| `context-percentage` | `bar` | `"true"` | Show progress bar |
| `context-percentage` | `inverse` | `"true"` | Show remaining instead of used |
| `session-cost` | `burn_rate` | `"true"` | Show hourly burn rate |
| `session-duration` | `api_ratio` | `"true"` | Show API time percentage |
| `block-timer` | `bar` | `"true"` | Show progress bar |
| `block-timer` | `bar_width` | `"16"` | Progress bar width |
| `cwd` | `full` | `"true"` | Show full path |
| `cwd` | `fish_style` | `"true"` | Fish-style abbreviation |
| `cwd` | `segments` | `"3"` | Show last N segments |
| `custom-command` | `command` | shell cmd | Command to execute |
| `custom-text` | `text` | any string | Static text to display |
| `separator` | `char` | any char | Separator character |

## Themes

6 built-in themes optimized for popular terminal color schemes:

```bash
claudeline theme list    # List all themes
claudeline theme set nord  # Switch theme
```

Available: `default`, `solarized`, `nord`, `dracula`, `gruvbox`, `monokai`

## Color Support

claudeline auto-detects terminal color capabilities:

- **Truecolor** (24-bit): detected via `COLORTERM=truecolor`
- **256-color**: detected via `TERM=*256color*`
- **16-color**: fallback for basic terminals
- **No color**: respects `NO_COLOR` environment variable

Override with `--color-level`:

```bash
claudeline --color-level truecolor
claudeline --color-level none
```

## CLI Commands

```bash
claudeline              # Render status line (reads JSON from stdin)
claudeline init         # Generate default config file
claudeline doctor       # Check environment compatibility
claudeline theme list   # List available themes
claudeline theme set <name>  # Switch theme
claudeline preset <name>     # Apply a preset layout
claudeline dump-schema       # Print expected JSON input schema
claudeline --version         # Show version
```

## Performance

Benchmarked on Apple M1:

| Metric | Value |
|--------|-------|
| Render time | <1ms |
| Binary size | 1.0 MB |
| Memory usage | 1.2 MB |
| Startup time | <1ms |

Claude Code debounces status line updates at 300ms. claudeline completes in <1ms, ensuring the status line is always fresh and never causes UI lag.

## How It Works

Claude Code pipes JSON session data to your status line script via stdin. claudeline reads this JSON, applies your configuration, and prints formatted ANSI text to stdout. No transcript parsing, no file watching, no external dependencies.

```
Claude Code → JSON stdin → claudeline → ANSI stdout → Terminal
```

## Building from Source

```bash
git clone https://github.com/mstuart/claudeline
cd claudeline
cargo build --release
# Binary at ./target/release/claudeline
```

## License

MIT
