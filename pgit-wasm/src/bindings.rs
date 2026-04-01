use pgit_core::{hash, manifest, purl::Purl, version};
use wasm_bindgen::prelude::*;

// ── PURL ─────────────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn purl_parse(s: &str) -> Result<JsPurl, JsValue> {
    Purl::parse(s).map(JsPurl).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub struct JsPurl(Purl);

#[wasm_bindgen]
impl JsPurl {
    pub fn pkg_type(&self) -> String          { self.0.pkg_type.clone() }
    pub fn namespace(&self) -> Option<String> { self.0.namespace.clone() }
    pub fn name(&self) -> String              { self.0.name.clone() }
    pub fn version(&self) -> Option<String>   { self.0.version.clone() }
    pub fn to_string(&self) -> String         { self.0.to_string() }
}

// ── Manifests ─────────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn parse_registry_manifest(toml_str: &str) -> Result<String, JsValue> {
    manifest::RegistryManifest::parse(toml_str)
        .map(|m| serde_json::to_string(&m).unwrap())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn parse_package_manifest(toml_str: &str) -> Result<String, JsValue> {
    manifest::PackageManifest::parse(toml_str)
        .map(|m| serde_json::to_string(&m).unwrap())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

// ── Version resolution ────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn resolve_version(req: &str, available_json: &str) -> Result<String, JsValue> {
    let available: Vec<String> = serde_json::from_str(available_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let refs: Vec<&str> = available.iter().map(String::as_str).collect();
    version::resolve(req, &refs).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ── Hashing ───────────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn sha256_bytes(data: &[u8]) -> String {
    hash::sha256_bytes(data)
}

/// Hash a package tree from a JSON array of `{ "path": "...", "content_b64": "..." }` objects.
#[wasm_bindgen]
pub fn sha256_tree_json(entries_json: &str) -> Result<String, JsValue> {
    #[derive(serde::Deserialize)]
    struct Entry { path: String, content_b64: String }

    let raw: Vec<Entry> = serde_json::from_str(entries_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut entries: Vec<(String, Vec<u8>)> = raw
        .into_iter()
        .map(|e| {
            let decoded = base64_decode(&e.content_b64)
                .map_err(|err| JsValue::from_str(&err))?;
            Ok((e.path, decoded))
        })
        .collect::<Result<_, JsValue>>()?;

    Ok(hash::sha256_tree(&mut entries))
}

fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    let decoded = js_sys::eval(&format!("atob('{}')", s))
        .map_err(|e| format!("{:?}", e))?;
    let str_val = decoded.as_string().ok_or_else(|| "atob returned non-string".to_string())?;
    Ok(str_val.as_bytes().to_vec())
}
