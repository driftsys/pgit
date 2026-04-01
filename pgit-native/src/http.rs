use anyhow::{Context, Result};

/// Download a URL to bytes, optionally with a Bearer token.
pub fn get_bytes(url: &str, token: Option<&str>) -> Result<Vec<u8>> {
    let mut req = ureq::get(url);
    if let Some(t) = token {
        req = req.header("Authorization", &format!("Bearer {}", t));
    }
    let response = req.call().with_context(|| format!("GET {}", url))?;
    let buf = response
        .into_body()
        .read_to_vec()
        .context("reading response body")?;
    Ok(buf)
}

/// Fetch the latest release tag for a GitHub repo via the GitHub API.
pub fn github_latest_tag(owner: &str, repo: &str, token: Option<&str>) -> Result<String> {
    let url = format!("https://api.github.com/repos/{}/{}/releases/latest", owner, repo);
    let bytes = get_bytes(&url, token)?;
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).context("parsing GitHub release JSON")?;
    json["tag_name"]
        .as_str()
        .map(str::to_string)
        .with_context(|| "tag_name missing from GitHub release response")
}

/// Fetch the latest release tag for a GitLab project via the GitLab API.
pub fn gitlab_latest_tag(host: &str, project: &str, token: Option<&str>) -> Result<String> {
    let encoded = urlencoding(project);
    let url = format!("https://{}/api/v4/projects/{}/releases?per_page=1", host, encoded);
    let bytes = get_bytes(&url, token)?;
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).context("parsing GitLab release JSON")?;
    json[0]["tag_name"]
        .as_str()
        .map(str::to_string)
        .with_context(|| "tag_name missing from GitLab release response")
}

fn urlencoding(s: &str) -> String {
    s.replace('/', "%2F")
}
