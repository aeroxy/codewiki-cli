# codewiki

[![crates.io](https://img.shields.io/crates/v/codewiki-cli.svg)](https://crates.io/crates/codewiki-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

Query GitHub repository wikis via [Google Code Wiki](https://codewiki.google/) — without opening a browser.

Built for LLM coding agents and humans. Outputs Markdown to stdout, with GitHub source references resolved to clickable URLs and architecture diagrams preserved as fenced ` ```dot ` blocks.

## Install

```bash
cargo install codewiki-cli
```

The crate is `codewiki-cli`; the binary it installs is `codewiki`.

## Usage

```bash
codewiki structure facebook/react           # list section titles
codewiki read facebook/react                # full wiki as Markdown
codewiki ask facebook/react "How does useEffect work?"
```

Pipe the output into your agent of choice:

```bash
codewiki read ast-grep/ast-grep | claude -p "Summarise the rule engine"
```

## How it works

Code Wiki has no public API. `codewiki` speaks Google's `batchexecute` RPC the same way the web frontend does:

- `VSX6ub` returns the entire wiki for a repo as structured JSON (the page is server-rendered from this same call).
- `EgIxfe` answers a chat question with Gemini.

A 6-hour disk cache for the build label / session id (`~/Library/Caches/codewiki/bootstrap.json` on macOS, equivalent on Linux/Windows) means back-to-back invocations skip the bootstrap GET. Override the cache location with `$CODEWIKI_CACHE_DIR`.

No authentication required. Public GitHub repos only (Code Wiki itself doesn't yet support private repos).

## Output format

Every command prints a header line followed by the result:

```
## CodeWiki: <owner>/<repo> (<command>)

<content>
```

`read` rewrites `[`text`](%2Fowner%2Frepo%2Fpath)` references to absolute `https://github.com/owner/repo/path` URLs and emits any embedded Graphviz diagrams after their section as fenced `dot` code blocks.

## Claude Code skill

A ready-to-install Claude Code skill ships in [`skill/codewiki/`](./skill/codewiki/SKILL.md) so Claude knows when to reach for `codewiki` automatically. Install:

```bash
cp -r skill/codewiki ~/.claude/skills/
```

(or symlink `skill/codewiki` to `~/.claude/skills/codewiki` if you want to track upstream changes.)

## Testing

```bash
cargo test
```

Integration tests are gated behind `CODEWIKI_MOCK_TEXT`, so the suite runs offline.

## License

MIT
