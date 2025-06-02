use anyhow::{Context, Result};
use git2::{FetchOptions, RemoteCallbacks, Repository};
use std::fs;
use std::path::Path;
use std::sync::Arc;

pub struct DmmRepoManager {
    pub repo_url: String,
    pub local_path: String,
}

impl DmmRepoManager {
    pub fn new(repo_url: impl Into<String>, local_path: impl Into<String>) -> Self {
        Self {
            repo_url: repo_url.into(),
            local_path: local_path.into(),
        }
    }

    pub fn sync_repo(&self) -> Result<()> {
        let path = Path::new(&self.local_path);

        if path.exists() && path.join(".git").exists() {
            self.pull_existing_repo(path)
        } else {
            self.clone_repo(path)
        }
    }

    fn clone_repo(&self, path: &Path) -> Result<()> {
        if path.exists() {
            fs::remove_dir_all(path).context("Failed to clean old repo directory")?;
        }

        tracing::debug!("Cloning repo from {} into {:?}", self.repo_url, path);

        let username = std::env::var("ZILEAN_GITHUB_USERNAME").ok();
        let token = std::env::var("ZILEAN_GITHUB_TOKEN").ok();

        let mut cb = RemoteCallbacks::new();
        cb.credentials(move |_, _, _| {
            if let (Some(user), Some(tok)) = (&username, &token) {
                git2::Cred::userpass_plaintext(user, tok)
            } else {
                git2::Cred::default()
            }
        });

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(cb);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_opts);

        builder
            .clone(&self.repo_url, path)
            .context("Failed to clone repo with credentials")?;

        Ok(())
    }

    fn pull_existing_repo(&self, path: &Path) -> Result<()> {
        let repo = Repository::open(path).context("Failed to open existing repo")?;
        let mut remote = repo
            .find_remote("origin")
            .context("No 'origin' remote found")?;

        let config = Arc::new(repo.config().context("Failed to get git config")?);
        let username = std::env::var("ZILEAN_GITHUB_USERNAME").ok();
        let token = std::env::var("ZILEAN_GITHUB_TOKEN").ok();

        let mut cb = RemoteCallbacks::new();
        cb.credentials(move |url, username_from_url, _| {
            let config = Arc::clone(&config);
            if let (Some(user), Some(tok)) = (&username, &token) {
                git2::Cred::userpass_plaintext(user, tok)
            } else {
                git2::Cred::credential_helper(&config, url, username_from_url)
                    .or_else(|_| git2::Cred::default())
            }
        });

        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);

        tracing::debug!("Fetching from origin/main");
        remote
            .fetch(&["main"], Some(&mut fo), None)
            .context("Failed to fetch from origin")?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

        let mut refspec = repo
            .find_reference("refs/heads/main")
            .or_else(|_| repo.head())?;

        let analysis = repo.merge_analysis(&[&fetch_commit])?;
        if analysis.0.is_up_to_date() {
            tracing::debug!("Already up to date.");
            return Ok(());
        }

        if analysis.0.is_fast_forward() {
            tracing::debug!("Performing fast-forward merge...");
            refspec.set_target(fetch_commit.id(), "Fast-Forward")?;
            repo.set_head("refs/heads/main")?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        }

        Ok(())
    }
}
