pub use crate::fetch_and_compare::{fetch_and_compare, FetchError, FetchResult};
use crate::{
    config::Credentials,
    git::{GitError, Repository},
};
use gix::ObjectId;
use std::{
    io,
    process::{Command, Stdio},
    time::Duration,
};
use tokio::runtime::Runtime;

#[derive(Debug)]
pub enum PollError {
    Git(GitError),
    Fetch(FetchError),
    Spawn(io::Error),
    Complete(io::Error),
    NonZeroExit {
        stdout: String,
        stderr: String,
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
    program: String,
    arguments: Vec<String>,
    resume: bool,
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
        let mut current_commit_id;
        for _ in 0..5 {
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

                    tracing::debug!("Running `{}` with arguments {:?}", &program, &arguments);

                    let output = Command::new(&program)
                        .args(&arguments)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .map_err(PollError::Spawn)?
                        .wait_with_output()
                        .map_err(PollError::Complete)?;

                    let stdout = String::from_utf8_lossy(output.stdout.as_slice());
                    let stderr = String::from_utf8_lossy(output.stderr.as_slice());

                    if output.status.success() {
                        tracing::info!(%stdout, %stderr, "Process completed successfully");
                    } else {
                        return Err(PollError::NonZeroExit {
                            stdout: stdout.to_string(),
                            stderr: stderr.to_string(),
                        });
                    }

                    let updated_commit_id = repo.current_commit_id()?;

                    if !resume {
                        break;
                    }
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
            tokio::time::sleep(Duration::from_secs(5)).await;
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
        Err(PollError::Fetch(error)) => {
            tracing::error!(?error, "Failed to fetch changes from the remote repository");
        }
        Err(PollError::Spawn(error)) => {
            tracing::error!(
                ?error,
                "Failed to spawn the command `{} {:?}`",
                program,
                arguments
            );
        }
        Err(PollError::Complete(error)) => {
            tracing::error!(
                ?error,
                "Failed to complete the command `{} {:?}`",
                program,
                arguments
            )
        }
        Err(PollError::NonZeroExit { stdout, stderr }) => {
            tracing::error!(%stdout, %stderr, "Process exited with a non-zero exit code");
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
