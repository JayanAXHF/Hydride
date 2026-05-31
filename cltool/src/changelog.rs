use std::path::Path;
use std::str::FromStr;

use git_cliff_core::changelog::Changelog;
use git_cliff_core::commit::Commit as CliffCommit;
use git_cliff_core::config::Config as CliffConfig;
use git_cliff_core::release::Release;
use git_cliff_core::repo::Repository;

use crate::error::AppError;

pub fn generate(
    repo_path: &Path,
    cliff_config_path: &Path,
    range: Option<&str>,
) -> Result<String, AppError> {
    let mut config = load_cliff_config(cliff_config_path)?;
    config.remote.offline = true;
    let commit_limit = range.and_then(parse_recent_commit_limit);

    let repository =
        Repository::discover(repo_path.to_path_buf()).map_err(|source| AppError::Repository {
            path: repo_path.display().to_string(),
            source,
        })?;

    let commits = if let Some(limit) = commit_limit {
        repository
            .commits(None, None, None, config.git.topo_order_commits)
            .map_err(|source| AppError::Repository {
                path: repo_path.display().to_string(),
                source,
            })?
            .into_iter()
            .take(limit)
            .map(|commit| CliffCommit::from(&commit))
            .collect::<Vec<_>>()
    } else {
        repository
            .commits(range, None, None, config.git.topo_order_commits)
            .map_err(|source| AppError::Repository {
                path: repo_path.display().to_string(),
                source,
            })?
            .into_iter()
            .map(|commit| CliffCommit::from(&commit))
            .collect::<Vec<_>>()
    };

    let mut release = Release::default();
    release.repository = Some(
        repository
            .root_path()
            .map_err(|source| AppError::Repository {
                path: repo_path.display().to_string(),
                source,
            })?
            .display()
            .to_string(),
    );
    release.commits = commits;

    let mut rendered = Vec::new();
    let changelog = Changelog::new(vec![release], config, range)
        .map_err(|source| AppError::Changelog { source })?;
    changelog
        .generate(&mut rendered)
        .map_err(|source| AppError::Changelog { source })?;

    String::from_utf8(rendered).map_err(|source| AppError::Utf8 { source })
}

fn load_cliff_config(path: &Path) -> Result<CliffConfig, AppError> {
    if path.exists() {
        CliffConfig::load(path).map_err(|source| AppError::CliffConfig {
            path: path.display().to_string(),
            source,
        })
    } else {
        CliffConfig::from_str("").map_err(|source| AppError::CliffConfig {
            path: path.display().to_string(),
            source,
        })
    }
}

fn parse_recent_commit_limit(range: &str) -> Option<usize> {
    let head_suffix = "..HEAD";
    let head_prefix = "HEAD~";

    let limit = range
        .strip_prefix(head_prefix)?
        .strip_suffix(head_suffix)?
        .parse::<usize>()
        .ok()?;

    if limit == 0 { None } else { Some(limit) }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use git2::{Repository as GitRepository, Signature};
    use tempfile::TempDir;

    use super::*;

    fn commit(repo: &GitRepository, file_name: &str, contents: &str, message: &str) {
        let workdir = repo.workdir().expect("repo should have a workdir");
        let path = workdir.join(file_name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create dirs");
        }
        fs::write(&path, contents).expect("write file");

        let mut index = repo.index().expect("index");
        index.add_path(Path::new(file_name)).expect("add path");
        index.write().expect("write index");
        let tree_id = index.write_tree().expect("tree");
        let tree = repo.find_tree(tree_id).expect("tree obj");
        let sig = Signature::now("Test", "test@example.com").expect("sig");
        let parent = repo
            .head()
            .ok()
            .and_then(|head| head.target())
            .and_then(|oid| repo.find_commit(oid).ok());
        match parent {
            Some(parent) => {
                repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])
                    .expect("commit");
            }
            None => {
                repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])
                    .expect("commit");
            }
        }
    }

    #[test]
    fn generates_changelog_from_git_history() {
        let dir = TempDir::new().expect("tempdir");
        let repo = GitRepository::init(dir.path()).expect("init repo");
        commit(&repo, "one.txt", "one", "feat: first");
        commit(&repo, "two.txt", "two", "fix: second");

        let cliff_path = dir.path().join("cliff.toml");
        fs::write(
            &cliff_path,
            r#"
[changelog]
body = "commits={{ commits | length }}"
"#,
        )
        .expect("write cliff config");

        let changelog = generate(dir.path(), &cliff_path, None).expect("generate changelog");

        assert!(changelog.contains("commits=2"), "{changelog}");
    }

    #[test]
    fn parses_recent_commit_range() {
        assert_eq!(parse_recent_commit_limit("HEAD~10..HEAD"), Some(10));
        assert_eq!(parse_recent_commit_limit("HEAD~0..HEAD"), None);
        assert_eq!(parse_recent_commit_limit("abc"), None);
    }
}
