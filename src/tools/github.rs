use anyhow::{Context, Result};
use octocrab::models::issues::Issue;
use octocrab::models::pulls::PullRequest;
use octocrab::Octocrab;
use serde::Serialize;
use tracing::info;

fn base64_encode(s: &str) -> String {
    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut encoder = Base64Encoder::new(&mut buf);
        encoder.write_all(s.as_bytes()).unwrap();
    }
    String::from_utf8(buf).unwrap()
}

/// Minimal base64 encoder (avoids adding a crate).
struct Base64Encoder<'a> {
    out: &'a mut Vec<u8>,
}

impl<'a> Base64Encoder<'a> {
    fn new(out: &'a mut Vec<u8>) -> Self {
        Self { out }
    }
}

impl std::io::Write for Base64Encoder<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        for chunk in buf.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
            let n = (b0 << 16) | (b1 << 8) | b2;
            self.out.push(CHARS[((n >> 18) & 63) as usize]);
            self.out.push(CHARS[((n >> 12) & 63) as usize]);
            if chunk.len() > 1 {
                self.out.push(CHARS[((n >> 6) & 63) as usize]);
            } else {
                self.out.push(b'=');
            }
            if chunk.len() > 2 {
                self.out.push(CHARS[(n & 63) as usize]);
            } else {
                self.out.push(b'=');
            }
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Lightweight wrappers around octocrab for the operations we need.
#[derive(Clone)]
pub struct GitHubClient {
    octo: Octocrab,
}

/// Summary of a repo scan.
#[derive(Debug, Serialize)]
pub struct RepoScanResult {
    pub repo: String,
    pub open_issues: Vec<IssueSummary>,
    pub open_prs: Vec<PrSummary>,
}

#[derive(Debug, Serialize)]
pub struct IssueSummary {
    pub number: u64,
    pub title: String,
    pub labels: Vec<String>,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct PrSummary {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub url: String,
    pub check_status: Option<String>,
}

impl GitHubClient {
    pub fn new(token: &str) -> Result<Self> {
        let octo = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .context("Failed to create GitHub client")?;
        Ok(Self { octo })
    }

    fn parse_repo(repo: &str) -> Result<(&str, &str)> {
        let parts: Vec<&str> = repo.splitn(2, '/').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid repo format: {repo} (expected owner/repo)");
        }
        Ok((parts[0], parts[1]))
    }

    pub async fn list_open_issues(&self, repo: &str) -> Result<Vec<Issue>> {
        let (owner, name) = Self::parse_repo(repo)?;
        let page = self
            .octo
            .issues(owner, name)
            .list()
            .state(octocrab::params::State::Open)
            .per_page(30)
            .send()
            .await
            .context("Failed to list issues")?;
        Ok(page.items)
    }

    pub async fn list_open_prs(&self, repo: &str) -> Result<Vec<PullRequest>> {
        let (owner, name) = Self::parse_repo(repo)?;
        let page = self
            .octo
            .pulls(owner, name)
            .list()
            .state(octocrab::params::State::Open)
            .per_page(30)
            .send()
            .await
            .context("Failed to list PRs")?;
        Ok(page.items)
    }

    #[allow(dead_code)]
    pub async fn get_file_content(
        &self,
        repo: &str,
        path: &str,
        branch: &str,
    ) -> Result<String> {
        let (owner, name) = Self::parse_repo(repo)?;
        let content = self
            .octo
            .repos(owner, name)
            .get_content()
            .path(path)
            .r#ref(branch)
            .send()
            .await
            .context("Failed to get file content")?;

        // Extract the file content from the response
        let item = content
            .items
            .into_iter()
            .next()
            .context("No file found at path")?;

        item.decoded_content()
            .context("Failed to decode file content")
    }

    pub async fn get_issue(&self, repo: &str, number: u64) -> Result<Issue> {
        let (owner, name) = Self::parse_repo(repo)?;
        self.octo
            .issues(owner, name)
            .get(number)
            .await
            .context("Failed to get issue")
    }

    pub async fn create_branch(
        &self,
        repo: &str,
        branch_name: &str,
        from_branch: &str,
    ) -> Result<()> {
        let (owner, name) = Self::parse_repo(repo)?;

        // Get the SHA of the source branch
        let reference = self
            .octo
            .repos(owner, name)
            .get_ref(&octocrab::params::repos::Reference::Branch(
                from_branch.to_string(),
            ))
            .await
            .context("Failed to get source branch ref")?;

        let sha = match reference.object {
            octocrab::models::repos::Object::Commit { sha, .. } => sha,
            octocrab::models::repos::Object::Tag { sha, .. } => sha,
            _ => anyhow::bail!("Unexpected ref object type"),
        };

        // Create new branch
        self.octo
            .repos(owner, name)
            .create_ref(
                &octocrab::params::repos::Reference::Branch(branch_name.to_string()),
                sha,
            )
            .await
            .context("Failed to create branch")?;

        info!(%branch_name, %from_branch, "Created branch");
        Ok(())
    }

    pub async fn commit_file(
        &self,
        repo: &str,
        path: &str,
        content: &str,
        message: &str,
        branch: &str,
    ) -> Result<()> {
        let (owner, name) = Self::parse_repo(repo)?;

        // Try to get existing file SHA (for updates)
        let existing_sha: Option<String> = self
            .octo
            .repos(owner, name)
            .get_content()
            .path(path)
            .r#ref(branch)
            .send()
            .await
            .ok()
            .and_then(|c| c.items.into_iter().next())
            .map(|item| item.sha);

        // Use the update endpoint which handles both create and update
        let mut body = serde_json::json!({
            "message": message,
            "content": base64_encode(content),
            "branch": branch,
        });
        if let Some(sha) = existing_sha {
            body["sha"] = serde_json::json!(sha);
        }

        self.octo
            ._put(
                format!("/repos/{owner}/{name}/contents/{path}"),
                Some(&body),
            )
            .await
            .context("Failed to commit file")?;

        info!(%path, %branch, "Committed file");
        Ok(())
    }

    pub async fn create_pr(
        &self,
        repo: &str,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
    ) -> Result<String> {
        let (owner, name) = Self::parse_repo(repo)?;
        let pr = self
            .octo
            .pulls(owner, name)
            .create(title, head, base)
            .body(body)
            .send()
            .await
            .context("Failed to create PR")?;

        let url = pr
            .html_url
            .map(|u| u.to_string())
            .unwrap_or_else(|| format!("https://github.com/{repo}/pull/{}", pr.number));

        info!(%url, "Created PR");
        Ok(url)
    }

    /// Scan a repo and produce a summary.
    pub async fn scan_repo(&self, repo: &str) -> Result<RepoScanResult> {
        let issues = self.list_open_issues(repo).await.unwrap_or_default();
        let prs = self.list_open_prs(repo).await.unwrap_or_default();

        let open_issues: Vec<IssueSummary> = issues
            .into_iter()
            // Filter out PRs (GitHub returns PRs in issues endpoint too)
            .filter(|i| i.pull_request.is_none())
            .map(|i| IssueSummary {
                number: i.number,
                title: i.title,
                labels: i.labels.iter().map(|l| l.name.clone()).collect(),
                url: i.html_url.to_string(),
            })
            .collect();

        let open_prs: Vec<PrSummary> = prs
            .into_iter()
            .map(|pr| PrSummary {
                number: pr.number,
                title: pr.title.clone().unwrap_or_default(),
                state: pr
                    .state
                    .as_ref()
                    .map(|s| format!("{s:?}"))
                    .unwrap_or_else(|| "unknown".to_string()),
                url: pr
                    .html_url
                    .map(|u| u.to_string())
                    .unwrap_or_default(),
                check_status: None, // Would need separate API call
            })
            .collect();

        Ok(RepoScanResult {
            repo: repo.to_string(),
            open_issues,
            open_prs,
        })
    }

    /// Format scan results into a Discord-friendly message.
    pub fn format_scan_report(results: &[RepoScanResult]) -> String {
        let mut out = String::from("**GitHub Scan Report**\n");

        for result in results {
            out.push_str(&format!("\n📦 **{}**\n", result.repo));

            if result.open_issues.is_empty() {
                out.push_str("  Issues: none\n");
            } else {
                out.push_str(&format!("  Issues ({}):\n", result.open_issues.len()));
                for issue in result.open_issues.iter().take(10) {
                    let labels = if issue.labels.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", issue.labels.join(", "))
                    };
                    out.push_str(&format!(
                        "    #{} {}{}\n",
                        issue.number, issue.title, labels
                    ));
                }
                if result.open_issues.len() > 10 {
                    out.push_str(&format!(
                        "    ... and {} more\n",
                        result.open_issues.len() - 10
                    ));
                }
            }

            if result.open_prs.is_empty() {
                out.push_str("  PRs: none\n");
            } else {
                out.push_str(&format!("  PRs ({}):\n", result.open_prs.len()));
                for pr in result.open_prs.iter().take(10) {
                    out.push_str(&format!("    #{} {}\n", pr.number, pr.title));
                }
            }
        }

        out
    }
}
