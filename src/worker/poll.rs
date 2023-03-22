pub use crate::fetch_and_compare::{fetch_and_compare, FetchError, FetchResult};
use crate::{
    config::Credentials,
    git::{GitError, Repository},
};
use gix::ObjectId;
use std::{fs::File, io, path::PathBuf, process::Command, time::Duration};
use time::macros::format_description;
use tokio::runtime::Runtime;

#[derive(Debug)]
pub enum PollError {
    Git(GitError),
    Fetch(FetchError),
    Directory(io::Error),
    File(io::Error),
    Spawn(io::Error),
    Complete(io::Error),
    NonZeroExit {
        path: String,
    },
    BranchWasNotUpdated,
    UnexpectedCommitId {
        remote_commit_id: ObjectId,
        updated_commit_id: ObjectId,
    },
}

impl From<GitError> for PollError {
    fn from(value: GitError) -> Self {
        Self::Git(value)
    }
}

impl From<FetchError> for PollError {
    fn from(value: FetchError) -> Self {
        Self::Fetch(value)
    }
}

pub fn poll(
    on_update: PathBuf,
    updates: PathBuf,
    interval: Duration,
    iterations: Option<usize>,
    credentials: Option<Credentials>,
) -> Result<(), PollError> {
    let repo = Repository::discover()?;

    let current_branch = repo.current_branch()?;

    tracing::debug!(
        "Branch detected: {}",
        current_branch.as_reference().full_name()
    );

    let remote_branch = repo.remote_branch(&current_branch)?;

    tracing::debug!(
        "Local branch `{}` tracks remote branch `{}`.",
        current_branch.as_reference().full_name(),
        remote_branch.as_reference().full_name(),
    );

    let runtime = Runtime::new().unwrap();
    let future = async {
        if !updates.exists() {
            std::fs::create_dir(&updates).map_err(PollError::Directory)?;
        }

        let mut current_commit_id;
        // TODO: use actual `loop` when `iterations` is not set
        for _ in 1..iterations.unwrap_or(usize::MAX) {
            current_commit_id = repo.current_commit_id()?;
            match fetch_and_compare(
                &repo,
                &remote_branch,
                current_commit_id,
                credentials.as_ref(),
            )
            .await?
            {
                FetchResult::UpToDate { .. } => {
                    tracing::info!("Up to date.");
                }
                FetchResult::OutOfDate {
                    remote_commit_id, ..
                } => {
                    tracing::info!("Update found.");

                    let format =
                        format_description!("[year]-[month]-[day]_[hour]-[minute]-[second]");
                    let name = time::OffsetDateTime::now_utc()
                        .format(format)
                        .expect("invalid format");

                    let path = updates.join(name);

                    tracing::debug!("Creating `{}`", path.display());

                    std::fs::create_dir(&path).map_err(PollError::Directory)?;
                    let stdout = File::create(path.join("stdout")).map_err(PollError::File)?;
                    let stderr = File::create(path.join("stderr")).map_err(PollError::File)?;

                    tracing::debug!("Running `{}`", &on_update.display());

                    let output = Command::new(&on_update)
                        .stdout(stdout)
                        .stderr(stderr)
                        .spawn()
                        .map_err(PollError::Spawn)?
                        .wait_with_output()
                        .map_err(PollError::Complete)?;

                    if output.status.success() {
                        let path = path.display();
                        tracing::info!(
                            %path,
                            "Process completed successfully"
                        );
                    } else {
                        return Err(PollError::NonZeroExit {
                            path: path.display().to_string(),
                        });
                    }

                    let updated_commit_id = repo.current_commit_id()?;

                    if current_commit_id == updated_commit_id {
                        return Err(PollError::BranchWasNotUpdated);
                    }
                    if remote_commit_id != updated_commit_id {
                        return Err(PollError::UnexpectedCommitId {
                            remote_commit_id,
                            updated_commit_id,
                        });
                    }
                }
            };
            // TODO: should not sleep on the last iteration
            tokio::time::sleep(interval).await;
        }

        Ok(())
    };

    match runtime.block_on(future) {
        Ok(()) => {
            tracing::info!("Polling finished.");
        }
        Err(PollError::Git(error)) => {
            tracing::error!(
                ?error,
                "Error encountered while trying to access local git repository"
            );
        }
        Err(PollError::Directory(error)) => {
            tracing::error!(?error, "Failed to create update directory");
        }
        Err(PollError::File(error)) => {
            tracing::error!(?error, "Failed to create update file(s)");
        }
        Err(PollError::Fetch(error)) => {
            tracing::error!(?error, "Failed to fetch changes from the remote repository");
        }
        Err(PollError::Spawn(error)) => {
            tracing::error!(?error, "Failed to execute `{}`", on_update.display());
        }
        Err(PollError::Complete(error)) => {
            tracing::error!(
                ?error,
                "Failed to complete the command `{}`",
                on_update.display(),
            )
        }
        Err(PollError::NonZeroExit { path }) => {
            tracing::error!(%path, "Process exited with a non-zero exit code");
        }
        Err(PollError::BranchWasNotUpdated) => {
            tracing::error!("The current branch has not been updated. Exiting the process.");
        }
        Err(PollError::UnexpectedCommitId {
            remote_commit_id,
            updated_commit_id,
        }) => {
            tracing::error!(
                %remote_commit_id,
                %updated_commit_id,
                "The current branch has been updated, but it is not the expected commit. Exiting the process.");
        }
    }

    Ok(())
}
