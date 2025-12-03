//! Git Integration
//!
//! Provides git operations for document version control.
//! Works with Obsidian Git plugin or standalone git repositories.

use std::path::Path;
use std::process::Command;

use super::IntegrationStatus;

/// Git integration for document operations
pub struct GitIntegration {
    /// Root path of the git repository
    repo_root: Option<std::path::PathBuf>,
}

impl GitIntegration {
    /// Create a new git integration, detecting the repository root
    pub fn new() -> Self {
        let repo_root = Self::find_repo_root();
        Self { repo_root }
    }

    /// Create git integration for a specific path
    pub fn for_path(path: &Path) -> Self {
        let repo_root = Self::find_repo_root_from(path);
        Self { repo_root }
    }

    /// Check if git is available
    pub fn check_availability(&self) -> IntegrationStatus {
        if self.repo_root.is_some() && Self::git_available() {
            IntegrationStatus::available("git")
        } else {
            IntegrationStatus::unavailable(
                "git",
                "Git repository not detected. Initialize with 'git init' or install Obsidian Git plugin.",
            )
        }
    }

    /// Check if git command is available
    fn git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Find git repository root from current directory
    fn find_repo_root() -> Option<std::path::PathBuf> {
        Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                std::path::PathBuf::from(String::from_utf8_lossy(&o.stdout).trim().to_string())
            })
    }

    /// Find git repository root from a specific path
    fn find_repo_root_from(path: &Path) -> Option<std::path::PathBuf> {
        let dir = if path.is_file() {
            path.parent()?
        } else {
            path
        };

        Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(dir)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                std::path::PathBuf::from(String::from_utf8_lossy(&o.stdout).trim().to_string())
            })
    }

    /// Get relative path from repo root
    fn relative_path(&self, path: &Path) -> Option<String> {
        let repo_root = self.repo_root.as_ref()?;
        path.strip_prefix(repo_root)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    }

    /// Create a safety snapshot commit before editing
    pub fn snapshot_before_edit(
        &self,
        path: &Path,
        message: Option<&str>,
    ) -> Result<SnapshotResult, GitError> {
        let repo_root = self
            .repo_root
            .as_ref()
            .ok_or(GitError::NotARepository)?;

        let rel_path = self
            .relative_path(path)
            .ok_or(GitError::PathOutsideRepo)?;

        // Check if file has changes
        let status = Command::new("git")
            .args(["status", "--porcelain", &rel_path])
            .current_dir(repo_root)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        let has_changes = !status.stdout.is_empty();

        if !has_changes {
            return Ok(SnapshotResult {
                commit_hash: None,
                message: "No changes to snapshot".to_string(),
                files_changed: 0,
            });
        }

        // Stage the file
        Command::new("git")
            .args(["add", &rel_path])
            .current_dir(repo_root)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        // Commit with message
        let commit_msg = message.unwrap_or("Auto-snapshot before Doc-Doctor edit");
        let full_msg = format!("[doc-doctor] {}", commit_msg);

        let commit = Command::new("git")
            .args(["commit", "-m", &full_msg])
            .current_dir(repo_root)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !commit.status.success() {
            return Err(GitError::CommitFailed(
                String::from_utf8_lossy(&commit.stderr).to_string(),
            ));
        }

        // Get commit hash
        let hash = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo_root)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

        Ok(SnapshotResult {
            commit_hash: hash,
            message: full_msg,
            files_changed: 1,
        })
    }

    /// Commit with a stub resolution message
    pub fn commit_stub_resolution(
        &self,
        path: &Path,
        stub_type: &str,
        stub_description: &str,
        additional_message: Option<&str>,
    ) -> Result<CommitResult, GitError> {
        let repo_root = self
            .repo_root
            .as_ref()
            .ok_or(GitError::NotARepository)?;

        let rel_path = self
            .relative_path(path)
            .ok_or(GitError::PathOutsideRepo)?;

        // Stage the file
        Command::new("git")
            .args(["add", &rel_path])
            .current_dir(repo_root)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        // Build commit message
        let mut msg = format!("[doc-doctor] Resolved {}:{}", stub_type, stub_description);
        if let Some(additional) = additional_message {
            msg.push_str(&format!("\n\n{}", additional));
        }

        let commit = Command::new("git")
            .args(["commit", "-m", &msg])
            .current_dir(repo_root)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !commit.status.success() {
            let stderr = String::from_utf8_lossy(&commit.stderr);
            // Check if it's "nothing to commit"
            if stderr.contains("nothing to commit") {
                return Ok(CommitResult {
                    commit_hash: None,
                    message: msg,
                    success: true,
                    note: Some("No changes to commit".to_string()),
                });
            }
            return Err(GitError::CommitFailed(stderr.to_string()));
        }

        // Get commit hash
        let hash = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo_root)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

        Ok(CommitResult {
            commit_hash: hash,
            message: msg,
            success: true,
            note: None,
        })
    }

    /// Get commit history for a document
    pub fn get_document_history(
        &self,
        path: &Path,
        limit: usize,
    ) -> Result<Vec<CommitInfo>, GitError> {
        let repo_root = self
            .repo_root
            .as_ref()
            .ok_or(GitError::NotARepository)?;

        let rel_path = self
            .relative_path(path)
            .ok_or(GitError::PathOutsideRepo)?;

        // Get git log for the file
        let output = Command::new("git")
            .args([
                "log",
                &format!("-{}", limit),
                "--format=%H|%an|%ae|%aI|%s",
                "--",
                &rel_path,
            ])
            .current_dir(repo_root)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let log = String::from_utf8_lossy(&output.stdout);
        let commits: Vec<CommitInfo> = log
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(5, '|').collect();
                if parts.len() == 5 {
                    Some(CommitInfo {
                        hash: parts[0].to_string(),
                        author_name: parts[1].to_string(),
                        author_email: parts[2].to_string(),
                        date: parts[3].to_string(),
                        message: parts[4].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(commits)
    }

    /// Get diff between document versions
    pub fn diff_document_versions(
        &self,
        path: &Path,
        from_commit: &str,
        to_commit: &str,
    ) -> Result<DiffResult, GitError> {
        let repo_root = self
            .repo_root
            .as_ref()
            .ok_or(GitError::NotARepository)?;

        let rel_path = self
            .relative_path(path)
            .ok_or(GitError::PathOutsideRepo)?;

        // Get the diff
        let output = Command::new("git")
            .args([
                "diff",
                "--stat",
                &format!("{}..{}", from_commit, to_commit),
                "--",
                &rel_path,
            ])
            .current_dir(repo_root)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        let stat = String::from_utf8_lossy(&output.stdout).to_string();

        // Get the full diff
        let diff_output = Command::new("git")
            .args([
                "diff",
                &format!("{}..{}", from_commit, to_commit),
                "--",
                &rel_path,
            ])
            .current_dir(repo_root)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        let diff = String::from_utf8_lossy(&diff_output.stdout).to_string();

        // Count additions and deletions
        let (additions, deletions) = diff
            .lines()
            .fold((0, 0), |(add, del), line| {
                if line.starts_with('+') && !line.starts_with("+++") {
                    (add + 1, del)
                } else if line.starts_with('-') && !line.starts_with("---") {
                    (add, del + 1)
                } else {
                    (add, del)
                }
            });

        Ok(DiffResult {
            from_commit: from_commit.to_string(),
            to_commit: to_commit.to_string(),
            diff,
            stat,
            additions,
            deletions,
        })
    }
}

impl Default for GitIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a snapshot operation
#[derive(Debug, Clone)]
pub struct SnapshotResult {
    pub commit_hash: Option<String>,
    pub message: String,
    pub files_changed: usize,
}

/// Result of a commit operation
#[derive(Debug, Clone)]
pub struct CommitResult {
    pub commit_hash: Option<String>,
    pub message: String,
    pub success: bool,
    pub note: Option<String>,
}

/// Information about a commit
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub author_name: String,
    pub author_email: String,
    pub date: String,
    pub message: String,
}

/// Result of a diff operation
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub from_commit: String,
    pub to_commit: String,
    pub diff: String,
    pub stat: String,
    pub additions: usize,
    pub deletions: usize,
}

/// Git operation errors
#[derive(Debug)]
pub enum GitError {
    NotARepository,
    PathOutsideRepo,
    CommandFailed(String),
    CommitFailed(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::NotARepository => write!(f, "Not a git repository"),
            GitError::PathOutsideRepo => write!(f, "Path is outside the repository"),
            GitError::CommandFailed(msg) => write!(f, "Git command failed: {}", msg),
            GitError::CommitFailed(msg) => write!(f, "Commit failed: {}", msg),
        }
    }
}

impl std::error::Error for GitError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_available() {
        // This test checks if git is available in the test environment
        let available = GitIntegration::git_available();
        // Don't assert - git may or may not be available
        println!("Git available: {}", available);
    }
}
