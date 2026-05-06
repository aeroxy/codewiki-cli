use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

const PREFIX: &str = ")]}'";

pub fn encode_request(rpc_id: &str, inner_json: &str) -> String {
    let envelope = json!([[[rpc_id, inner_json, Value::Null, "generic"]]]);
    let serialized = serde_json::to_string(&envelope).expect("envelope is always serializable");
    format!("f.req={}&", urlencoding::encode(&serialized))
}

pub fn decode_response(body: &str, rpc_id: &str) -> Result<Value> {
    // Boq batchexecute responses are: `)]}'\n` + a stream of `<length>\n<json>`
    // chunks. The `<length>` framing has off-by-some quirks (sometimes counts
    // the trailing newline, sometimes one extra digit), so instead of trusting
    // it we strip the prefix, scan forward to the first `[` (the start of the
    // first JSON value), and use serde_json's streaming Deserializer to walk
    // the concatenated JSON values. This skips length headers as junk between
    // values, which the streaming parser is designed to tolerate when we drive
    // it ourselves via raw_value boundaries.
    let trimmed = body
        .trim_start_matches('\u{feff}')
        .strip_prefix(PREFIX)
        .ok_or_else(|| anyhow!("response missing `)]}}'` prefix"))?;

    let mut cursor = trimmed;
    loop {
        // Skip whitespace and any non-`[`/`{` chars (length headers).
        cursor = cursor.trim_start();
        let Some(start) = cursor.find(['[', '{']) else {
            break;
        };
        cursor = &cursor[start..];

        let mut de = serde_json::Deserializer::from_str(cursor).into_iter::<Value>();
        let Some(next) = de.next() else { break };
        let frame = next.context("decoding chunk JSON")?;
        let consumed = de.byte_offset();
        cursor = &cursor[consumed..];

        if let Some(payload) = find_rpc_payload(&frame, rpc_id)? {
            return Ok(payload);
        }
    }

    Err(anyhow!("no `wrb.fr` frame found for rpc id `{}`", rpc_id))
}

fn find_rpc_payload(frames: &Value, rpc_id: &str) -> Result<Option<Value>> {
    let outer = frames
        .as_array()
        .ok_or_else(|| anyhow!("frame is not an array"))?;
    for frame in outer {
        let arr = match frame.as_array() {
            Some(a) => a,
            None => continue,
        };
        let tag = arr.first().and_then(Value::as_str).unwrap_or_default();
        if tag != "wrb.fr" {
            continue;
        }
        let id = arr.get(1).and_then(Value::as_str).unwrap_or_default();
        if id != rpc_id {
            continue;
        }
        let inner = arr
            .get(2)
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("`wrb.fr` frame missing inner JSON string"))?;
        let parsed: Value =
            serde_json::from_str(inner).context("inner payload is not valid JSON")?;
        return Ok(Some(parsed));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_request_url_encodes_envelope() {
        let body = encode_request("VSX6ub", "[\"https://github.com/owner/repo\"]");
        assert!(body.starts_with("f.req="));
        assert!(body.ends_with('&'));
        let decoded = urlencoding::decode(body.trim_start_matches("f.req=").trim_end_matches('&'))
            .expect("encoded body should round-trip");
        let parsed: Value = serde_json::from_str(&decoded).unwrap();
        assert_eq!(parsed[0][0][0], "VSX6ub");
        assert_eq!(parsed[0][0][1], "[\"https://github.com/owner/repo\"]");
        assert_eq!(parsed[0][0][3], "generic");
    }

    #[test]
    fn decode_egixfe_fixture() {
        let body = include_str!("../tests/fixtures/egixfe_response.txt");
        let payload = decode_response(body, "EgIxfe").expect("decode");
        let answer = payload[0].as_str().expect("string answer");
        assert!(answer.starts_with("Hello world"));
        assert!(answer.contains("[link](%2Fast-grep"));
    }

    #[test]
    fn decode_vsx6ub_fixture() {
        let body = include_str!("../tests/fixtures/vsx6ub_response.txt");
        let payload = decode_response(body, "VSX6ub").expect("decode");
        // payload = [ wiki, [null, githubUrl], true, 3 ]
        // wiki    = [ [repo, sha], [sections...], ... ]
        let repo = payload[0][0][0].as_str().unwrap();
        assert_eq!(repo, "owner/example");
        let sections = payload[0][1].as_array().unwrap();
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0][0], "Example Overview");
        assert_eq!(sections[0][1], 1);
    }

    #[test]
    fn decode_returns_err_when_rpc_id_missing() {
        let body = ")]}'\n14\n[[\"e\",4,null]]\n";
        let err = decode_response(body, "VSX6ub").unwrap_err();
        assert!(
            err.to_string().contains("no `wrb.fr` frame"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn decode_rejects_missing_prefix() {
        let err = decode_response("not a boq response", "VSX6ub").unwrap_err();
        assert!(err.to_string().contains("missing"));
    }
}
