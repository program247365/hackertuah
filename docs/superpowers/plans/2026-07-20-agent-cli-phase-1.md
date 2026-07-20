# Agent-Friendly CLI (Phase 1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `stories` subcommand with `--json` output, stable exit codes, and stderr discipline, while `hackertuah` with no args keeps launching the TUI unchanged.

**Architecture:** A new `src/cli.rs` holds clap arg definitions and the CLI execution path. `main()` parses args before any terminal setup: subcommand present → run CLI and `std::process::exit`; no subcommand → TTY guard, then the existing TUI. `hn_api::fetch_stories` gains a `limit` parameter (it currently fetches 100 items serially; the TUI keeps passing 100).

**Tech Stack:** Rust 2021, clap 4 (derive), serde/serde_json, tokio, existing reqwest-based `hn_api`.

## Global Constraints

- Contract: stdout carries data only; progress/errors go to stderr. Exit 0 = success, 1 = runtime/network failure, 2 = usage error (clap's default) or TTY guard refusal.
- Run `make verify` (fmt + clippy -D warnings + build + test) before every commit.
- Conventional commits (cocogitto is in use).
- Do not change TUI behavior: bare `hackertuah` in a terminal must behave exactly as before.
- Don't refactor code outside the listed files; keep edits small and focused.

---

### Task 1: clap dependency + `src/cli.rs` arg definitions

**Files:**
- Modify: `Cargo.toml` (add clap)
- Create: `src/cli.rs` (Cli/Commands/CliSection + tests)
- Modify: `src/main.rs:9-12` (declare `mod cli;`)

**Interfaces:**
- Produces: `cli::Cli` (clap Parser with `pub command: Option<Commands>`), `cli::Commands::Stories { section: CliSection, limit: usize, json: bool }`, `cli::CliSection` (Top|Ask|Show|Jobs) with `From<CliSection> for types::Section`.

- [ ] **Step 1: Add clap to Cargo.toml**

In `[dependencies]`:

```toml
clap = { version = "4.5", features = ["derive"] }
```

- [ ] **Step 2: Write failing tests in a new `src/cli.rs`**

Create `src/cli.rs` with ONLY the test module first (so the test run fails to compile — that's the red step in Rust):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn bare_invocation_has_no_subcommand() {
        let cli = Cli::try_parse_from(["hackertuah"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn stories_defaults_to_top_limit_30_human_output() {
        let cli = Cli::try_parse_from(["hackertuah", "stories"]).unwrap();
        match cli.command {
            Some(Commands::Stories { section, limit, json }) => {
                assert_eq!(section, CliSection::Top);
                assert_eq!(limit, 30);
                assert!(!json);
            }
            _ => panic!("expected stories subcommand"),
        }
    }

    #[test]
    fn stories_accepts_section_limit_and_json() {
        let cli =
            Cli::try_parse_from(["hackertuah", "stories", "ask", "--limit", "5", "--json"])
                .unwrap();
        match cli.command {
            Some(Commands::Stories { section, limit, json }) => {
                assert_eq!(section, CliSection::Ask);
                assert_eq!(limit, 5);
                assert!(json);
            }
            _ => panic!("expected stories subcommand"),
        }
    }

    #[test]
    fn unknown_section_is_a_usage_error() {
        assert!(Cli::try_parse_from(["hackertuah", "stories", "bestest"]).is_err());
    }

    #[test]
    fn cli_section_maps_to_types_section() {
        assert_eq!(Section::from(CliSection::Jobs).as_str(), "Jobs");
    }
}
```

Also add `mod cli;` to `src/main.rs` next to the existing `mod hn_api;` block (line 9).

- [ ] **Step 3: Run tests to verify failure**

Run: `cargo test`
Expected: compile error — `Cli`, `Commands`, `CliSection` not found.

- [ ] **Step 4: Implement the definitions above the test module in `src/cli.rs`**

```rust
use clap::{Parser, Subcommand, ValueEnum};

use crate::types::Section;

#[derive(Parser)]
#[command(name = "hackertuah", version, about = "Hacker News in your terminal — TUI by default, agent-friendly subcommands for scripts")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List stories from a Hacker News section
    Stories {
        /// Section to fetch
        #[arg(value_enum, default_value_t = CliSection::Top)]
        section: CliSection,
        /// Maximum number of stories to fetch
        #[arg(long, default_value_t = 30)]
        limit: usize,
        /// Emit JSON on stdout instead of human-readable text
        #[arg(long)]
        json: bool,
    },
}

#[derive(ValueEnum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CliSection {
    Top,
    Ask,
    Show,
    Jobs,
}

impl From<CliSection> for Section {
    fn from(s: CliSection) -> Section {
        match s {
            CliSection::Top => Section::Top,
            CliSection::Ask => Section::Ask,
            CliSection::Show => Section::Show,
            CliSection::Jobs => Section::Jobs,
        }
    }
}
```

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test`
Expected: all 5 tests PASS. (`cargo build` will warn about unused `cli` items — acceptable until Task 4 wires them up; if `clippy -D warnings` fails on dead_code, add `#[allow(dead_code)]` on `Cli` temporarily and remove it in Task 4.)

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/cli.rs src/main.rs
git commit -m "feat(cli): add clap arg definitions for stories subcommand"
```

---

### Task 2: `Story` serialization + serde_json

**Files:**
- Modify: `Cargo.toml` (add serde_json)
- Modify: `src/types.rs:3` (derive Serialize on Story)

**Interfaces:**
- Produces: `types::Story: serde::Serialize` — Task 4 calls `serde_json::to_string_pretty(&Vec<Story>)`.

- [ ] **Step 1: Add serde_json to Cargo.toml**

In `[dependencies]`:

```toml
serde_json = "1.0"
```

- [ ] **Step 2: Write failing test at the bottom of `src/types.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story_serializes_with_stable_field_names() {
        let story = Story {
            id: 42,
            title: "Test".to_string(),
            url: Some("https://example.com".to_string()),
            text: None,
            by: "pg".to_string(),
            score: 100,
            descendants: 12,
            kids: vec![1, 2],
        };
        let json = serde_json::to_value(&story).unwrap();
        assert_eq!(json["id"], 42);
        assert_eq!(json["title"], "Test");
        assert_eq!(json["url"], "https://example.com");
        assert_eq!(json["by"], "pg");
        assert_eq!(json["score"], 100);
        assert_eq!(json["descendants"], 12);
    }
}
```

- [ ] **Step 3: Run test to verify failure**

Run: `cargo test story_serializes`
Expected: compile error — `Story` does not implement `Serialize`.

- [ ] **Step 4: Add the derive**

Change `src/types.rs:3` from:

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct Story {
```

to:

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Story {
```

(`use serde::{Deserialize, Serialize};` is already imported at `src/types.rs:1`.)

- [ ] **Step 5: Run test to verify pass**

Run: `cargo test story_serializes`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/types.rs
git commit -m "feat(types): derive Serialize on Story for JSON output"
```

---

### Task 3: `limit` parameter on `fetch_stories`

**Files:**
- Modify: `src/hn_api.rs:5-16`
- Modify: `src/main.rs:317` (TUI call site passes 100)

**Interfaces:**
- Produces: `pub async fn fetch_stories(section: Section, limit: usize) -> Result<Vec<Story>, Box<dyn Error + Send + Sync>>` — Task 4 calls it with the CLI `--limit` value.

No unit test: the function is a thin network wrapper (mocking reqwest here is not worth the harness). Verification is compile + existing behavior via `make verify`.

- [ ] **Step 1: Change the signature and take(limit)**

In `src/hn_api.rs:5`:

```rust
pub async fn fetch_stories(
    section: Section,
    limit: usize,
) -> Result<Vec<Story>, Box<dyn Error + Send + Sync>> {
```

and at `src/hn_api.rs:16` change `for id in ids.iter().take(100) {` to:

```rust
    for id in ids.iter().take(limit) {
```

- [ ] **Step 2: Update the TUI call site**

In `src/main.rs:317`, change:

```rust
.map(|section| tokio::spawn(async move { (section, fetch_stories(section).await) }))
```

to:

```rust
.map(|section| tokio::spawn(async move { (section, fetch_stories(section, 100).await) }))
```

- [ ] **Step 3: Verify**

Run: `make verify`
Expected: fmt clean, clippy clean, build OK, all tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/hn_api.rs src/main.rs
git commit -m "refactor(api): add limit parameter to fetch_stories"
```

---

### Task 4: CLI execution path, human/JSON output, main() branch, TTY guard

**Files:**
- Modify: `src/cli.rs` (add `run`, `run_stories`, `format_story` + tests)
- Modify: `src/main.rs:569-580` (branch before terminal setup)

**Interfaces:**
- Consumes: `Cli`/`Commands`/`CliSection` (Task 1), `Story: Serialize` (Task 2), `fetch_stories(section, limit)` (Task 3).
- Produces: `pub async fn cli::run(cmd: Commands) -> i32` (exit code), `pub fn cli::format_story(story: &Story, rank: usize) -> String`.

- [ ] **Step 1: Write failing tests for `format_story` (append inside the existing tests module in `src/cli.rs`)**

```rust
    use crate::types::Story;

    fn sample_story() -> Story {
        Story {
            id: 42,
            title: "Test story".to_string(),
            url: Some("https://example.com".to_string()),
            text: None,
            by: "pg".to_string(),
            score: 100,
            descendants: 12,
            kids: vec![],
        }
    }

    #[test]
    fn format_story_shows_rank_title_meta_url_and_id() {
        let out = format_story(&sample_story(), 1);
        assert!(out.contains("  1. Test story"));
        assert!(out.contains("100 points by pg | 12 comments"));
        assert!(out.contains("https://example.com"));
        assert!(out.contains("id:42"));
    }

    #[test]
    fn format_story_omits_url_line_when_absent() {
        let mut story = sample_story();
        story.url = None;
        let out = format_story(&story, 3);
        assert!(out.contains("  3. Test story"));
        assert!(!out.contains("https://"));
    }
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test format_story`
Expected: compile error — `format_story` not found.

- [ ] **Step 3: Implement `format_story`, `run_stories`, `run` in `src/cli.rs`**

```rust
use crate::hn_api;
use crate::types::Story;

pub async fn run(cmd: Commands) -> i32 {
    match cmd {
        Commands::Stories { section, limit, json } => {
            run_stories(section.into(), limit, json).await
        }
    }
}

async fn run_stories(section: Section, limit: usize, json: bool) -> i32 {
    eprintln!("fetching up to {} {} stories...", limit, section.as_str());
    let stories = match hn_api::fetch_stories(section, limit).await {
        Ok(stories) => stories,
        Err(e) => {
            eprintln!("error: {}", e);
            return 1;
        }
    };
    if json {
        match serde_json::to_string_pretty(&stories) {
            Ok(out) => println!("{}", out),
            Err(e) => {
                eprintln!("error: {}", e);
                return 1;
            }
        }
    } else if stories.is_empty() {
        println!("(no stories)");
    } else {
        for (i, story) in stories.iter().enumerate() {
            print!("{}", format_story(story, i + 1));
        }
    }
    0
}

pub fn format_story(story: &Story, rank: usize) -> String {
    let mut out = format!("{:>3}. {}\n", rank, story.title);
    out.push_str(&format!(
        "     {} points by {} | {} comments\n",
        story.score, story.by, story.descendants
    ));
    if let Some(url) = &story.url {
        out.push_str(&format!("     {}\n", url));
    }
    out.push_str(&format!("     id:{}\n", story.id));
    out
}
```

(Adjust the top-of-file imports to: `use crate::hn_api;` `use crate::types::{Section, Story};` — merging with the existing `use crate::types::Section;`. Remove any temporary `#[allow(dead_code)]` from Task 1.)

- [ ] **Step 4: Branch in `main()` before terminal setup**

In `src/main.rs:569`, at the top of `main()` before `enable_raw_mode()?;`:

```rust
    let parsed = <cli::Cli as clap::Parser>::parse();
    if let Some(cmd) = parsed.command {
        std::process::exit(cli::run(cmd).await);
    }

    {
        use std::io::IsTerminal;
        if !io::stdout().is_terminal() {
            eprintln!("stdout is not a terminal; the TUI needs one.");
            eprintln!("hint: did you mean `hackertuah stories --json`?");
            std::process::exit(2);
        }
    }
```

- [ ] **Step 5: Run tests, then verify end-to-end behavior**

Run: `cargo test`
Expected: all tests PASS.

Run: `cargo run --quiet -- stories --limit 3 --json 2>/dev/null | python3 -m json.tool > /dev/null && echo JSON-OK`
Expected: `JSON-OK` (stdout is pure parseable JSON; progress went to stderr).

Run: `cargo run --quiet -- stories ask --limit 3`
Expected: 3 human-readable entries, each with an `id:` line.

Run: `cargo run --quiet 2>&1 | head -2; echo "exit: ${pipestatus[1]}"`
Expected: the TTY-guard hint and `exit: 2` (stdout piped, no subcommand → guard refuses to launch TUI).

Run: `cargo run --quiet -- stories bogus; echo "exit: $?"`
Expected: clap usage error on stderr, `exit: 2`.

Manually run `cargo run` in the terminal: TUI launches as before.

- [ ] **Step 6: Run full verification and commit**

Run: `make verify`
Expected: clean.

```bash
git add src/cli.rs src/main.rs
git commit -m "feat(cli): stories subcommand with --json, exit codes, and TTY guard"
```

---

### Task 5: Document the contract in README + CLAUDE.md

**Files:**
- Modify: `README.md` (add "Agent / scripting usage" section after the usage/keybindings docs)
- Modify: `CLAUDE.md` (Architecture + Build sections mention `cli.rs` and the subcommand)

**Interfaces:**
- Consumes: the CLI surface from Task 4 — document exactly what shipped, nothing aspirational.

- [ ] **Step 1: Add README section**

```markdown
## Agent / scripting usage

Running `hackertuah` with no arguments launches the TUI. Subcommands never
touch the terminal — data goes to stdout, progress and errors to stderr:

```bash
hackertuah stories [top|ask|show|jobs] [--limit N] [--json]
```

- `--json` emits raw HN story objects (`id`, `title`, `url`, `text`, `by`,
  `score`, `descendants`, `kids`), pretty-printed.
- Exit codes: 0 success, 1 network/runtime failure, 2 usage error (also
  returned when the bare TUI is invoked with stdout piped).

```bash
hackertuah stories --limit 10 --json | jq -r '.[].title'
```
```

- [ ] **Step 2: Update CLAUDE.md**

In the Architecture list, after the `ui.rs` bullet:

```markdown
- **`cli.rs`** — Agent-friendly CLI: clap definitions (`Cli`, `Commands`, `CliSection`) and the non-TUI execution path (`run`, `format_story`). Data on stdout, progress on stderr; exit 0/1/2. Depends on `types` and `hn_api`.
```

And in Key Details:

```markdown
- CLI mode: `hackertuah stories [section] [--limit N] [--json]` bypasses the TUI entirely; bare `hackertuah` with piped stdout exits 2 with a hint.
```

- [ ] **Step 3: Commit**

```bash
git add README.md CLAUDE.md
git commit -m "docs: document agent-friendly CLI contract"
```

---

## Out of scope (later phases)

- Phase 2: `comments <id>`, `story <id>`, `--plain` TSV mode, `doctor`, `new` section (needs a `Section::New` variant threaded through the TUI menu).
- Phase 3: `summarize <id>` (and replacing the deprecated hardcoded `claude-3-opus-20240229` model), AGENTS.md.
- Concurrent story hydration in `fetch_stories` (currently serial; `--limit` keeps CLI latency acceptable).
