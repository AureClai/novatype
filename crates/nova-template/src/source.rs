//! Template sources (local, GitHub, marketplace).

use crate::cache::TemplateCache;
use crate::error::{Error, Result};
use crate::template::InstalledTemplate;
use std::io::Read;
use tempfile::TempDir;
use tracing::{debug, info};
use url::Url;

/// Template source types.
#[derive(Debug, Clone)]
pub enum TemplateSource {
    /// Local directory.
    Local(String),

    /// GitHub repository (owner/repo).
    GitHub {
        owner: String,
        repo: String,
        branch: Option<String>,
        path: Option<String>,
    },

    /// Direct URL to a ZIP file.
    Url(String),
}

impl TemplateSource {
    /// Parse a source string into a TemplateSource.
    ///
    /// Supported formats:
    /// - `/path/to/template` or `./template` - Local path
    /// - `github:owner/repo` - GitHub repository (default branch)
    /// - `github:owner/repo#branch` - GitHub repository with branch
    /// - `github:owner/repo/path/to/template` - Subdirectory in repo
    /// - `https://...` - Direct URL to ZIP
    pub fn parse(source: &str) -> Result<Self> {
        // Local path
        if source.starts_with('/') || source.starts_with('.') || source.starts_with('\\') {
            return Ok(Self::Local(source.to_string()));
        }

        // Windows absolute path (C:\...)
        if source.len() > 2 && source.chars().nth(1) == Some(':') {
            return Ok(Self::Local(source.to_string()));
        }

        // GitHub shorthand
        if let Some(github_ref) = source.strip_prefix("github:") {
            return Self::parse_github(github_ref);
        }

        // URL
        if source.starts_with("https://") || source.starts_with("http://") {
            // Check if it's a GitHub URL
            if let Ok(url) = Url::parse(source) {
                if url.host_str() == Some("github.com") {
                    return Self::parse_github_url(&url);
                }
            }
            return Ok(Self::Url(source.to_string()));
        }

        // Assume it's a GitHub owner/repo shorthand
        if source.contains('/') && !source.contains(' ') {
            return Self::parse_github(source);
        }

        Err(Error::InvalidSourceUrl {
            url: source.to_string(),
        })
    }

    /// Parse GitHub shorthand (owner/repo or owner/repo#branch).
    fn parse_github(github_ref: &str) -> Result<Self> {
        let (repo_part, branch) = if let Some((repo, branch)) = github_ref.split_once('#') {
            (repo, Some(branch.to_string()))
        } else {
            (github_ref, None)
        };

        let parts: Vec<&str> = repo_part.split('/').collect();
        if parts.len() < 2 {
            return Err(Error::InvalidSourceUrl {
                url: github_ref.to_string(),
            });
        }

        let owner = parts[0].to_string();
        let repo = parts[1].to_string();
        let path = if parts.len() > 2 {
            Some(parts[2..].join("/"))
        } else {
            None
        };

        Ok(Self::GitHub {
            owner,
            repo,
            branch,
            path,
        })
    }

    /// Parse a GitHub URL.
    fn parse_github_url(url: &Url) -> Result<Self> {
        let path_segments: Vec<&str> = url.path_segments().map_or(vec![], |s| s.collect());

        if path_segments.len() < 2 {
            return Err(Error::InvalidSourceUrl {
                url: url.to_string(),
            });
        }

        let owner = path_segments[0].to_string();
        let repo = path_segments[1].trim_end_matches(".git").to_string();

        // Check for tree/branch/path pattern
        let (branch, path) = if path_segments.len() > 3 && path_segments[2] == "tree" {
            let branch = Some(path_segments[3].to_string());
            let path = if path_segments.len() > 4 {
                Some(path_segments[4..].join("/"))
            } else {
                None
            };
            (branch, path)
        } else {
            (None, None)
        };

        Ok(Self::GitHub {
            owner,
            repo,
            branch,
            path,
        })
    }

    /// Get a display name for this source.
    pub fn display_name(&self) -> String {
        match self {
            Self::Local(path) => format!("local:{}", path),
            Self::GitHub { owner, repo, .. } => format!("github:{}/{}", owner, repo),
            Self::Url(url) => url.clone(),
        }
    }
}

/// Template installer that handles different sources.
pub struct TemplateInstaller {
    client: reqwest::Client,
}

impl TemplateInstaller {
    /// Create a new template installer.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Install a template from a source.
    pub async fn install(
        &self,
        cache: &mut TemplateCache,
        source: &TemplateSource,
    ) -> Result<InstalledTemplate> {
        match source {
            TemplateSource::Local(path) => {
                info!(path = %path, "Installing from local directory");
                cache.install_from_directory(path, &source.display_name())
            }

            TemplateSource::GitHub {
                owner,
                repo,
                branch,
                path,
            } => {
                info!(owner = %owner, repo = %repo, "Installing from GitHub");
                self.install_from_github(cache, owner, repo, branch.as_deref(), path.as_deref())
                    .await
            }

            TemplateSource::Url(url) => {
                info!(url = %url, "Installing from URL");
                self.install_from_url(cache, url).await
            }
        }
    }

    /// Install from GitHub repository.
    async fn install_from_github(
        &self,
        cache: &mut TemplateCache,
        owner: &str,
        repo: &str,
        branch: Option<&str>,
        subpath: Option<&str>,
    ) -> Result<InstalledTemplate> {
        let branch = branch.unwrap_or("main");

        // Download ZIP archive
        let zip_url = format!(
            "https://github.com/{}/{}/archive/refs/heads/{}.zip",
            owner, repo, branch
        );

        debug!(url = %zip_url, "Downloading repository archive");
        let response = self.client.get(&zip_url).send().await?;

        if !response.status().is_success() {
            // Try 'master' branch if 'main' fails
            if branch == "main" {
                let zip_url = format!(
                    "https://github.com/{}/{}/archive/refs/heads/master.zip",
                    owner, repo
                );
                let response = self.client.get(&zip_url).send().await?;

                if !response.status().is_success() {
                    return Err(Error::Source {
                        source_name: "GitHub".to_string(),
                        message: format!("Failed to download {}/{}: {}", owner, repo, response.status()),
                    });
                }

                return self
                    .extract_and_install(cache, response, &format!("{}-master", repo), subpath, &format!("github:{}/{}", owner, repo))
                    .await;
            }

            return Err(Error::Source {
                source_name: "GitHub".to_string(),
                message: format!("Failed to download {}/{}: {}", owner, repo, response.status()),
            });
        }

        self.extract_and_install(
            cache,
            response,
            &format!("{}-{}", repo, branch),
            subpath,
            &format!("github:{}/{}", owner, repo),
        )
        .await
    }

    /// Install from a direct URL.
    async fn install_from_url(
        &self,
        cache: &mut TemplateCache,
        url: &str,
    ) -> Result<InstalledTemplate> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(Error::Source {
                source_name: "URL".to_string(),
                message: format!("Failed to download {}: {}", url, response.status()),
            });
        }

        self.extract_and_install(cache, response, "", None, url)
            .await
    }

    /// Extract ZIP and install template.
    async fn extract_and_install(
        &self,
        cache: &mut TemplateCache,
        response: reqwest::Response,
        root_dir_prefix: &str,
        subpath: Option<&str>,
        source_name: &str,
    ) -> Result<InstalledTemplate> {
        let bytes = response.bytes().await?;

        // Create temp directory for extraction
        let temp_dir = TempDir::new()?;

        // Extract ZIP
        let reader = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader)?;

        debug!(
            files = archive.len(),
            "Extracting {} files",
            archive.len()
        );

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            // Skip directories
            if name.ends_with('/') {
                continue;
            }

            // Calculate target path
            let relative_path = if !root_dir_prefix.is_empty() {
                // Strip the root directory (e.g., "repo-main/")
                name.strip_prefix(&format!("{}/", root_dir_prefix))
                    .unwrap_or(&name)
            } else {
                &name
            };

            // If subpath specified, filter and strip
            let relative_path = if let Some(sp) = subpath {
                if let Some(stripped) = relative_path.strip_prefix(&format!("{}/", sp)) {
                    stripped
                } else if relative_path == sp {
                    continue; // Skip the directory entry itself
                } else {
                    continue; // Not in subpath
                }
            } else {
                relative_path
            };

            let target_path = temp_dir.path().join(relative_path);

            // Create parent directories
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Extract file
            let mut content = Vec::new();
            file.read_to_end(&mut content)?;
            std::fs::write(&target_path, content)?;
        }

        // Install from extracted directory
        cache.install_from_directory(temp_dir.path(), source_name)
    }
}

impl Default for TemplateInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_local_path() {
        let source = TemplateSource::parse("./my-template").unwrap();
        assert!(matches!(source, TemplateSource::Local(_)));
    }

    #[test]
    fn parse_github_shorthand() {
        let source = TemplateSource::parse("github:owner/repo").unwrap();
        match source {
            TemplateSource::GitHub { owner, repo, branch, path } => {
                assert_eq!(owner, "owner");
                assert_eq!(repo, "repo");
                assert!(branch.is_none());
                assert!(path.is_none());
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn parse_github_with_branch() {
        let source = TemplateSource::parse("github:owner/repo#develop").unwrap();
        match source {
            TemplateSource::GitHub { branch, .. } => {
                assert_eq!(branch, Some("develop".to_string()));
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn parse_github_with_path() {
        let source = TemplateSource::parse("github:owner/repo/templates/article").unwrap();
        match source {
            TemplateSource::GitHub { path, .. } => {
                assert_eq!(path, Some("templates/article".to_string()));
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn parse_owner_repo_shorthand() {
        let source = TemplateSource::parse("owner/repo").unwrap();
        assert!(matches!(source, TemplateSource::GitHub { .. }));
    }

    #[test]
    fn parse_url() {
        let source = TemplateSource::parse("https://example.com/template.zip").unwrap();
        assert!(matches!(source, TemplateSource::Url(_)));
    }

    #[test]
    fn source_display_name() {
        let local = TemplateSource::Local("./template".to_string());
        assert_eq!(local.display_name(), "local:./template");

        let github = TemplateSource::GitHub {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            branch: None,
            path: None,
        };
        assert_eq!(github.display_name(), "github:owner/repo");
    }
}
