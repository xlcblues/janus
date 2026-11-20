//! Release candidate helpers for the M8 milestone.
//!
//! Version parsing/comparison and a lightweight GitHub Releases check are kept
//! pure (unit-testable); the network call is a thin `ureq` GET with explicit
//! timeouts and a user-facing Chinese error on failure.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub html_url: String,
    #[serde(default)]
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateCheck {
    pub current: String,
    pub latest: String,
    pub has_update: bool,
    pub download_url: String,
    pub published_at: Option<String>,
}

/// 把 "v1.2.3-beta.1" 这类版本串拆成数字段：去掉 v 前缀，
/// 再按 `.` `-` `+` 切分，只保留能解析为数字的部分。
pub fn parse_version(value: &str) -> Vec<u32> {
    value
        .trim()
        .trim_start_matches(['v', 'V'])
        .split(['.', '-', '+'])
        .filter_map(|part| part.parse::<u32>().ok())
        .collect()
}

/// 逐段比较两个版本号，缺失段按 0 处理（1.0 < 1.0.1）。
pub fn compare_versions(a: &str, b: &str) -> Ordering {
    let left = parse_version(a);
    let right = parse_version(b);
    let len = left.len().max(right.len());
    for index in 0..len {
        let lhs = left.get(index).copied().unwrap_or(0);
        let rhs = right.get(index).copied().unwrap_or(0);
        let ordering = lhs.cmp(&rhs);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

pub fn parse_release_payload(payload: &str) -> Result<ReleaseInfo, String> {
    serde_json::from_str(payload)
        .map_err(|error| format!("无法解析版本信息（{error}）。"))
}

pub fn resolve_update(current: &str, release: &ReleaseInfo) -> UpdateCheck {
    let latest = release.tag_name.trim_start_matches(['v', 'V']).to_string();
    let has_update = compare_versions(&latest, current) == Ordering::Greater;
    UpdateCheck {
        current: current.to_string(),
        latest,
        has_update,
        download_url: release.html_url.clone(),
        published_at: release.published_at.clone(),
    }
}

/// 查询 GitHub Releases 最新版本。失败时返回用户可理解的中文提示。
pub fn check_for_update(current: &str, repo: &str) -> Result<UpdateCheck, String> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(8))
        .timeout_read(Duration::from_secs(15))
        .timeout_write(Duration::from_secs(15))
        .build();
    let body = agent
        .get(&url)
        .set("User-Agent", "Janus/0.1.0")
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|error| format!("检查更新失败，请检查网络后重试（{error}）。"))?
        .into_string()
        .map_err(|error| format!("检查更新失败：{error}"))?;
    let release = parse_release_payload(&body)?;
    Ok(resolve_update(current, &release))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_strips_prefix_and_splits() {
        assert_eq!(parse_version("v1.2.3"), vec![1, 2, 3]);
        assert_eq!(parse_version("0.10.0"), vec![0, 10, 0]);
        assert_eq!(parse_version("1.2.3-beta.1"), vec![1, 2, 3, 1]);
    }

    #[test]
    fn compare_versions_orders_correctly() {
        assert_eq!(compare_versions("1.2.3", "1.2.2"), Ordering::Greater);
        assert_eq!(compare_versions("1.10.0", "1.9.9"), Ordering::Greater);
        assert_eq!(compare_versions("0.1.0", "0.1.0"), Ordering::Equal);
        assert_eq!(compare_versions("1.0", "1.0.1"), Ordering::Less);
        assert_eq!(compare_versions("v1.2.3", "1.2.3"), Ordering::Equal);
    }

    #[test]
    fn parse_release_payload_reads_fields() {
        let payload = r#"{
            "tag_name": "v0.2.0",
            "html_url": "https://github.com/xlcblues/janus/releases/tag/v0.2.0",
            "published_at": "2026-11-20T00:00:00Z"
        }"#;
        let release = parse_release_payload(payload).unwrap();
        assert_eq!(release.tag_name, "v0.2.0");
        assert_eq!(release.published_at.as_deref(), Some("2026-11-20T00:00:00Z"));
    }

    #[test]
    fn resolve_update_flags_newer_release() {
        let release = ReleaseInfo {
            tag_name: "v0.2.0".to_string(),
            html_url: "https://github.com/xlcblues/janus/releases/tag/v0.2.0".to_string(),
            published_at: None,
        };
        let result = resolve_update("0.1.0", &release);
        assert!(result.has_update);
        assert_eq!(result.latest, "0.2.0");
        assert_eq!(result.download_url, release.html_url);
    }

    #[test]
    fn resolve_update_is_false_for_older_or_equal_release() {
        let same = ReleaseInfo {
            tag_name: "v0.1.0".to_string(),
            html_url: "https://github.com/xlcblues/janus/releases/tag/v0.1.0".to_string(),
            published_at: None,
        };
        assert!(!resolve_update("0.1.0", &same).has_update);

        let older = ReleaseInfo {
            tag_name: "0.0.9".to_string(),
            html_url: "https://github.com/xlcblues/janus/releases/tag/0.0.9".to_string(),
            published_at: None,
        };
        assert!(!resolve_update("0.1.0", &older).has_update);
    }

    #[test]
    fn malformed_payload_is_reported() {
        assert!(parse_release_payload("not-json").is_err());
    }
}
