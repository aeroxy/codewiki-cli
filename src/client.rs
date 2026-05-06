use anyhow::{anyhow, Context, Result};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, ORIGIN, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{boq, wiki};

const SITE: &str = "https://codewiki.google";
const ENDPOINT: &str = "https://codewiki.google/_/BoqAngularSdlcAgentsUi/data/batchexecute";
const USER_AGENT_VALUE: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36";
const CACHE_TTL_SECS: u64 = 6 * 60 * 60;

// Last-known good fallback values, captured 2026-05-06. Used only if the live
// bootstrap GET fails outright (network down, page restructured) — the disk
// cache covers the normal warm-start path.
const FALLBACK_BL: &str = "boq_sdlc-agents-ui_20260504.02_p0";
const FALLBACK_SID: &str = "-8491411211174446345";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Bootstrap {
    bl: String,
    sid: String,
    fetched_at: u64,
}

pub struct CodeWikiClient {
    http: reqwest::Client,
    bootstrap: Bootstrap,
}

impl CodeWikiClient {
    pub async fn connect() -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT_VALUE)
            .build()
            .context("building HTTP client")?;
        let bootstrap = load_bootstrap(&http).await?;
        Ok(Self { http, bootstrap })
    }

    pub async fn read_wiki(&self, repo: &str) -> Result<wiki::Wiki> {
        let inner = serde_json::to_string(&json!([github_url(repo)]))?;
        let payload = self.call("VSX6ub", repo, &inner).await?;
        wiki::parse(&payload)
    }

    pub async fn ask(&self, repo: &str, question: &str) -> Result<String> {
        let inner = serde_json::to_string(&json!([
            [[question, "user"]],
            [serde_json::Value::Null, github_url(repo)]
        ]))?;
        let payload = self.call("EgIxfe", repo, &inner).await?;
        payload
            .get(0)
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .ok_or_else(|| anyhow!("EgIxfe response did not contain an answer string"))
    }

    async fn call(&self, rpc_id: &str, repo: &str, inner_json: &str) -> Result<serde_json::Value> {
        let body = boq::encode_request(rpc_id, inner_json);
        let url = format!(
            "{ENDPOINT}?rpcids={rpc}&source-path={path}&bl={bl}&f.sid={sid}&hl=en-US&_reqid={reqid}&rt=c",
            rpc = rpc_id,
            path = urlencoding::encode(&format!("/github.com/{repo}")),
            bl = urlencoding::encode(&self.bootstrap.bl),
            sid = urlencoding::encode(&self.bootstrap.sid),
            reqid = pseudo_reqid(),
        );

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert(ORIGIN, HeaderValue::from_static(SITE));
        let referer = format!("{SITE}/github.com/{repo}");
        headers.insert(REFERER, HeaderValue::from_str(&referer)?);
        headers.insert("X-Same-Domain", HeaderValue::from_static("1"));

        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded;charset=UTF-8",
            )
            .body(body)
            .send()
            .await
            .context("posting batchexecute request")?;
        let status = resp.status();
        let text = resp.text().await.context("reading response body")?;
        if !status.is_success() {
            return Err(anyhow!(
                "batchexecute returned HTTP {}: {}",
                status,
                text.chars().take(200).collect::<String>()
            ));
        }
        boq::decode_response(&text, rpc_id)
    }
}

fn github_url(repo: &str) -> String {
    format!("https://github.com/{repo}")
}

fn pseudo_reqid() -> u32 {
    // Boq accepts any positive integer; some webapp shards use a timestamp tail.
    let now = now_secs();
    100_000 + (now as u32 % 900_000)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

async fn load_bootstrap(http: &reqwest::Client) -> Result<Bootstrap> {
    let cache_path = cache_path();
    if let Some(cached) = read_cache(cache_path.as_deref()) {
        if now_secs().saturating_sub(cached.fetched_at) < CACHE_TTL_SECS {
            return Ok(cached);
        }
    }

    match fetch_bootstrap(http).await {
        Ok(fresh) => {
            write_cache(cache_path.as_deref(), &fresh);
            Ok(fresh)
        }
        Err(e) => {
            eprintln!(
                "codewiki: live bootstrap failed ({e:#}); falling back to compiled-in defaults"
            );
            Ok(Bootstrap {
                bl: FALLBACK_BL.to_string(),
                sid: FALLBACK_SID.to_string(),
                fetched_at: now_secs(),
            })
        }
    }
}

fn cache_path() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("CODEWIKI_CACHE_DIR") {
        return Some(PathBuf::from(dir).join("bootstrap.json"));
    }
    Some(dirs::cache_dir()?.join("codewiki").join("bootstrap.json"))
}

fn read_cache(path: Option<&std::path::Path>) -> Option<Bootstrap> {
    let path = path?;
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn write_cache(path: Option<&std::path::Path>, bs: &Bootstrap) {
    let Some(path) = path else { return };
    let Some(parent) = path.parent() else { return };
    if let Err(e) = std::fs::create_dir_all(parent) {
        eprintln!("codewiki: cache dir create failed: {e}");
        return;
    }
    let tmp = path.with_extension("json.tmp");
    let bytes = match serde_json::to_vec_pretty(bs) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("codewiki: cache serialize failed: {e}");
            return;
        }
    };
    if let Err(e) = std::fs::write(&tmp, &bytes) {
        eprintln!("codewiki: cache write failed: {e}");
        return;
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        eprintln!("codewiki: cache rename failed: {e}");
    }
}

async fn fetch_bootstrap(http: &reqwest::Client) -> Result<Bootstrap> {
    let resp = http
        .get(SITE)
        .header(USER_AGENT, USER_AGENT_VALUE)
        .header(ACCEPT_LANGUAGE, "en-US,en;q=0.9")
        .send()
        .await
        .context("GET codewiki.google")?
        .error_for_status()
        .context("codewiki.google returned non-2xx")?;
    let html = resp.text().await.context("reading bootstrap HTML")?;
    extract_bootstrap(&html)
}

fn extract_bootstrap(html: &str) -> Result<Bootstrap> {
    // `WIZ_global_data = {...};` is one inlined JS object literal. We pull it as
    // text via a regex, then parse it as JSON. The object is JSON-shaped (Boq
    // wraps it in JSON for SSR), so `serde_json` accepts it.
    let re = Regex::new(r#"WIZ_global_data\s*=\s*(\{[\s\S]*?\});"#).expect("static regex");
    let cap = re
        .captures(html)
        .ok_or_else(|| anyhow!("WIZ_global_data not found in page HTML"))?;
    let raw = cap.get(1).unwrap().as_str();
    let json: serde_json::Value =
        serde_json::from_str(raw).context("WIZ_global_data is not valid JSON")?;
    let bl = json
        .get("cfb2h")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("missing cfb2h (build label) in WIZ_global_data"))?
        .to_string();
    let sid = json
        .get("FdrFJe")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("missing FdrFJe (session id) in WIZ_global_data"))?
        .to_string();
    Ok(Bootstrap {
        bl,
        sid,
        fetched_at: now_secs(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_bootstrap_pulls_bl_and_sid() {
        let html = r#"<html><script>window.WIZ_global_data = {"cfb2h":"boq_test_x","FdrFJe":"-12345","other":1};</script></html>"#;
        let bs = extract_bootstrap(html).expect("parse");
        assert_eq!(bs.bl, "boq_test_x");
        assert_eq!(bs.sid, "-12345");
    }

    #[test]
    fn extract_bootstrap_errors_when_missing() {
        let err = extract_bootstrap("<html>nothing here</html>").unwrap_err();
        assert!(err.to_string().contains("WIZ_global_data"));
    }
}
