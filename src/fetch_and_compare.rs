use crate::{
    config::Credentials,
    git::{Branch, GitError, Repository},
};
use gix::{protocol::handshake::Ref, ObjectId};

#[derive(Debug)]
pub enum FetchResult {
    UpToDate,
    OutOfDate { remote_commit_id: ObjectId },
}

#[derive(Debug)]
pub enum FetchError {
    GitError(GitError),
    FetchRemoteMissing,
}

impl From<GitError> for FetchError {
    fn from(value: GitError) -> Self {
        Self::GitError(value)
    }
}

pub async fn fetch_and_compare(
    repository: &Repository,
    branch: &Branch,
    current_id: ObjectId,
    credentials: Option<&Credentials>,
) -> Result<FetchResult, FetchError> {
    let res = repository.fetch(credentials)?;

    let full_ref_name_on_remote = branch.as_reference().local().full_name();

    let latest_remote_id = res
        .ref_map
        .remote_refs
        .iter()
        .find_map(|r| match r {
            Ref::Direct {
                full_ref_name,
                object,
            } => {
                if &full_ref_name_on_remote == full_ref_name {
                    Some(object)
                } else {
                    None
                }
            }
            Ref::Peeled { .. } | Ref::Symbolic { .. } | Ref::Unborn { .. } => None,
        })
        .ok_or(FetchError::FetchRemoteMissing)?;

    if current_id == *latest_remote_id {
        Ok(FetchResult::UpToDate)
    } else {
        Ok(FetchResult::OutOfDate {
            remote_commit_id: *latest_remote_id,
        })
    }
}
