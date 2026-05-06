use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::fmt::Write;

const GITHUB_PREFIX: &str = "https://github.com";

pub struct Wiki {
    pub repo_slug: String,
    pub commit_sha: String,
    pub sections: Vec<Section>,
}

pub struct Section {
    pub title: String,
    pub level: u8,
    pub markdown: String,
    pub diagrams: Vec<String>,
}

pub fn parse(payload: &Value) -> Result<Wiki> {
    // Layout (from a real VSX6ub response):
    //   payload = [ wiki, [null, githubUrl], true, 3 ]
    //   wiki    = [ [repoSlug, commitSha], [sections...], null, null, [...] ]
    let wiki = payload
        .get(0)
        .ok_or_else(|| anyhow!("missing wiki container"))?;
    let header = wiki
        .get(0)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("missing wiki header array"))?;
    let repo_slug = header
        .first()
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing repo slug"))?
        .to_string();
    let commit_sha = header
        .get(1)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let raw_sections = wiki
        .get(1)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("missing sections array"))?;
    let sections = raw_sections
        .iter()
        .enumerate()
        .map(|(i, s)| parse_section(s).with_context(|| format!("section #{i}")))
        .collect::<Result<Vec<_>>>()?;

    Ok(Wiki {
        repo_slug,
        commit_sha,
        sections,
    })
}

fn parse_section(value: &Value) -> Result<Section> {
    let arr = value
        .as_array()
        .ok_or_else(|| anyhow!("section is not an array"))?;
    let title = arr
        .first()
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing title"))?
        .to_string();
    let level = arr
        .get(1)
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("missing level"))? as u8;
    let markdown = arr
        .get(5)
        .and_then(Value::as_str)
        .or_else(|| arr.get(4).and_then(Value::as_str))
        .unwrap_or_default()
        .to_string();
    let diagrams = arr
        .get(7)
        .and_then(Value::as_array)
        .map(|outer| extract_diagrams(outer))
        .unwrap_or_default();

    Ok(Section {
        title,
        level,
        markdown,
        diagrams,
    })
}

fn extract_diagrams(outer: &[Value]) -> Vec<String> {
    let mut out = Vec::new();
    for group in outer {
        let Some(group) = group.as_array() else {
            continue;
        };
        for diagram in group {
            let Some(diagram) = diagram.as_array() else {
                continue;
            };
            if let Some(dot) = diagram.get(4).and_then(Value::as_str) {
                if !dot.is_empty() {
                    out.push(dot.to_string());
                }
            }
        }
    }
    out
}

pub fn render_structure(wiki: &Wiki) -> String {
    let mut out = String::new();
    for s in &wiki.sections {
        let indent = "  ".repeat((s.level.saturating_sub(1)) as usize);
        let _ = writeln!(out, "{indent}- {}", s.title);
    }
    out
}

pub fn render_markdown(wiki: &Wiki) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {} (commit {})\n", wiki.repo_slug, wiki.commit_sha);
    for s in &wiki.sections {
        let hashes = "#".repeat(s.level.clamp(1, 6) as usize);
        let _ = writeln!(out, "{hashes} {}\n", s.title);
        let body = resolve_links(&s.markdown);
        out.push_str(&body);
        if !out.ends_with('\n') {
            out.push('\n');
        }
        for dot in &s.diagrams {
            let _ = writeln!(out, "\n```dot\n{}\n```\n", dot.trim());
        }
        out.push('\n');
    }
    out
}

pub fn resolve_links(markdown: &str) -> String {
    // Replace `](%2F...)` with `](https://github.com/...)`. Markdown link targets
    // start with `](` and end at the next `)`. We URL-decode and prefix only when
    // the target begins with `%2F` (or `/`) to avoid touching real URLs.
    let mut out = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(open) = rest.find("](") {
        out.push_str(&rest[..open]);
        let after = &rest[open + 2..];
        match find_link_end(after) {
            Some(end) => {
                out.push_str("](");
                out.push_str(&rewrite_target(&after[..end]));
                out.push(')');
                rest = &after[end + 1..];
            }
            None => {
                out.push_str("](");
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

fn find_link_end(s: &str) -> Option<usize> {
    // Allow nested parens up to one level (covers `[foo](url(extra))`-style URLs that
    // CodeWiki does not currently produce, but it's cheap insurance).
    let mut depth = 0usize;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' if depth == 0 => return Some(i),
            ')' => depth -= 1,
            _ => {}
        }
    }
    None
}

fn rewrite_target(target: &str) -> String {
    let needs_rewrite =
        target.starts_with("%2F") || target.starts_with("%2f") || target.starts_with('/');
    if !needs_rewrite {
        return target.to_string();
    }
    let decoded = urlencoding::decode(target)
        .map(|c| c.into_owned())
        .unwrap_or_else(|_| target.to_string());
    if decoded.starts_with('/') {
        format!("{GITHUB_PREFIX}{decoded}")
    } else {
        decoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boq;

    fn fixture_payload() -> Value {
        let body = include_str!("../tests/fixtures/vsx6ub_response.txt");
        boq::decode_response(body, "VSX6ub").expect("decode")
    }

    #[test]
    fn parse_extracts_repo_and_sections() {
        let wiki = parse(&fixture_payload()).expect("parse");
        assert_eq!(wiki.repo_slug, "owner/example");
        assert_eq!(wiki.commit_sha, "abc123");
        assert_eq!(wiki.sections.len(), 3);
        assert_eq!(wiki.sections[0].title, "Example Overview");
        assert_eq!(wiki.sections[0].level, 1);
        assert_eq!(wiki.sections[1].title, "Section A");
        assert_eq!(wiki.sections[1].level, 2);
    }

    #[test]
    fn parse_extracts_dot_diagrams() {
        let wiki = parse(&fixture_payload()).expect("parse");
        assert_eq!(wiki.sections[0].diagrams.len(), 1);
        assert!(wiki.sections[0].diagrams[0].contains("digraph G"));
        assert_eq!(wiki.sections[1].diagrams.len(), 0);
    }

    #[test]
    fn render_structure_indents_by_level() {
        let wiki = parse(&fixture_payload()).expect("parse");
        let out = render_structure(&wiki);
        assert!(out.contains("- Example Overview"));
        assert!(out.contains("  - Section A"));
        assert!(out.contains("    - Sub A.1"));
    }

    #[test]
    fn render_markdown_includes_headers_and_dot_blocks() {
        let wiki = parse(&fixture_payload()).expect("parse");
        let out = render_markdown(&wiki);
        assert!(out.contains("# owner/example"));
        assert!(out.contains("# Example Overview"));
        assert!(out.contains("## Section A"));
        assert!(out.contains("### Sub A.1"));
        assert!(out.contains("```dot"));
        assert!(out.contains("digraph G"));
    }

    #[test]
    fn render_markdown_resolves_github_links() {
        let wiki = parse(&fixture_payload()).expect("parse");
        let out = render_markdown(&wiki);
        // The `Section A` body has a link to `%2Fowner%2Fexample%2Fcrates%2Fcli%2Fsrc%2Flib.rs`.
        assert!(
            out.contains("https://github.com/owner/example/crates/cli/src/lib.rs"),
            "expected resolved github link, got:\n{out}"
        );
        // Make sure unrelated `]` characters survive (sanity check the byte walker).
        assert!(out.contains("Body of section A"));
    }

    #[test]
    fn rewrite_target_passes_through_external_urls() {
        assert_eq!(
            rewrite_target("https://example.com/x"),
            "https://example.com/x"
        );
        assert_eq!(rewrite_target("#anchor"), "#anchor");
    }

    #[test]
    fn rewrite_target_resolves_encoded_repo_path() {
        assert_eq!(
            rewrite_target("%2Fowner%2Frepo%2Ffile.rs"),
            "https://github.com/owner/repo/file.rs"
        );
    }
}
