# codewiki

[![crates.io](https://img.shields.io/crates/v/codewiki-cli.svg)](https://crates.io/crates/codewiki-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

Query GitHub repository wikis via [Google Code Wiki](https://codewiki.google/) — without opening a browser.

Built for LLM coding agents and humans. Outputs Markdown to stdout, with GitHub source references resolved to clickable URLs and architecture diagrams preserved as fenced ` ```dot ` blocks.

## Install

### Homebrew (macOS arm64)

```bash
brew install aeroxy/tap/codewiki-cli
```

### Cargo

```bash
cargo install codewiki-cli
```

The crate is `codewiki-cli`; the binary it installs is `codewiki`.

### Building with Docker

To cross-compile release binaries locally without setting up native target toolchains:

```bash
docker build --file .\Dockerfile.build --output "type=local,dest=target" .
```

This outputs built binaries (e.g. for Linux/Windows) to the local `./target` directory.

## Usage

```bash
codewiki structure facebook/react           # list section titles
codewiki read facebook/react                # full wiki as Markdown to stdout
codewiki ask facebook/react "How does useEffect work?"
```

Pipe the output into your agent of choice:

```bash
codewiki read ast-grep/ast-grep | claude -p "Summarise the rule engine"
```

### Splitting into multiple files

By default, `read` prints everything as a single combined document. To save the wiki locally as a structured hierarchy of files (perfect for Obsidian vaults or indexing in local RAG vector stores):

```bash
codewiki read facebook/react -o              # split into files inside "wiki/" (default)
codewiki read facebook/react -o docs -d 3    # split up to depth 3, write to "docs/"
```

* **`-o, --out-dir [DIR]`**: Enables writing to a directory instead of stdout. If no path is provided, it defaults to `wiki`.
* **`-d, --depth <DEPTH>`**: The heading level boundary to split on. Defaults to `2`.
  * `-d 1`: Combines everything into a single file named after the root header.
  * `-d 2` (default when `-o` is present): Splits at level-2 headings (`##`) into flat `.md` files. Subsections (`###`, etc.) are appended to their parent file.
  * `-d 3`: Splits at level-2 headings (`##`) into folders, and level-3 headings (`###`) into separate `.md` files within them.

## How it works

Code Wiki has no public API. `codewiki` speaks Google's `batchexecute` RPC the same way the web frontend does:

- `VSX6ub` returns the entire wiki for a repo as structured JSON (the page is server-rendered from this same call).
- `EgIxfe` answers a chat question with Gemini.

A 6-hour disk cache for the build label / session id (`~/Library/Caches/codewiki/bootstrap.json` on macOS, equivalent on Linux/Windows) means back-to-back invocations skip the bootstrap GET. Override the cache location with `$CODEWIKI_CACHE_DIR`.

No authentication required. Public GitHub repos only (Code Wiki itself doesn't yet support private repos).

### TLS

TLS certificate verification is **disabled by default**. `codewiki` is built to run inside monitored agent sandboxes whose TLS-intercepting proxies present certificates that don't chain to a trusted root — strict verification would otherwise make every request fail with an opaque error. Set `CODEWIKI_TLS_VERIFY=1` (or `true`/`yes`) to restore strict certificate checking.

## Output format

When printing to stdout, every command prints a header line followed by the result:

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
