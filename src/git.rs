#![allow(clippy::result_large_err)]

use gix::{
    discover, reference,
    remote::{
        connect,
        fetch::{self, prepare, Outcome},
        Direction,
    },
    Head, ObjectId, Remote,
};

#[derive(Debug)]
pub enum GitError {
    RepositoryNotFound(discover::Error),
    RepositoryHeadMissing(reference::find::existing::Error),
    RepositoryHeadDetached,
    RepositoryHeadUninitialized,
    RepositoryReferenceError(reference::find::Error),
    RepositoryRemoteNotFound,
    RepositoryRemoteInvalid,
    RepositoryDefaultRemoteNotConfigured,
    RepositoryDefaultRemoteMissing,
    FetchConnect(connect::Error),
    FetchHandshake(prepare::Error),
    FetchReceive(fetch::Error),
}

pub struct Repository(gix::Repository);

#[derive(Debug, Clone)]
pub enum Branch {
    Local(String),
    Remote(String),
}

impl Branch {
    pub fn short_name(&self) -> &str {
        match self {
            Self::Local(name) | Self::Remote(name) => name.as_ref(),
        }
    }

    pub fn as_reference(&self) -> Reference<'_> {
        match self {
            Self::Local(name) => Reference::Local(name),
            Self::Remote(name) => Reference::Remote(name),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Reference<'a> {
    Local(&'a str),
    Remote(&'a str),
}

impl<'a> Reference<'a> {
    pub fn short_name(&self) -> &str {
        match self {
            Self::Local(name) | Self::Remote(name) => name,
        }
    }

    pub fn full_name(&self) -> String {
        match self {
            Self::Local(name) => format!("refs/heads/{name}"),
            Self::Remote(name) => format!("refs/remotes/{name}"),
        }
    }

    pub fn local(self) -> Self {
        match self {
            Self::Local(name) | Self::Remote(name) => Self::Local(name),
        }
    }
}

impl Repository {
    pub fn discover() -> Result<Self, GitError> {
        gix::discover(".")
            .map(Self)
            .map_err(GitError::RepositoryNotFound)
    }

    fn head(&self) -> Result<Head, GitError> {
        self.0.head().map_err(GitError::RepositoryHeadMissing)
    }

    pub fn current_branch(&self) -> Result<Branch, GitError> {
        self.head()?
            .referent_name()
            .map(|r| Branch::Local(r.shorten().to_string()))
            .ok_or(GitError::RepositoryHeadDetached)
    }

    pub fn current_commit_id(&self) -> Result<ObjectId, GitError> {
        self.head()?
            .id()
            .map(|id| id.detach())
            .ok_or(GitError::RepositoryHeadDetached)
    }

    pub fn remote_branch(&self, branch: &Branch) -> Result<Branch, GitError> {
        debug_assert!(matches!(branch, Branch::Local(..)));
        self.0
            .branch_remote_ref(branch.short_name())
            .ok_or(GitError::RepositoryRemoteNotFound)?
            .map(|r| Branch::Remote(r.shorten().to_string()))
            .map_err(|_| GitError::RepositoryRemoteInvalid)
    }

    pub fn remote(&self) -> Result<Remote, GitError> {
        self.0
            .find_default_remote(Direction::Fetch)
            .ok_or(GitError::RepositoryDefaultRemoteNotConfigured)?
            .map_err(|_| GitError::RepositoryDefaultRemoteMissing)
    }

    pub fn fetch(&self) -> Result<Outcome, GitError> {
        self.remote()?
            .connect(Direction::Fetch, gix::progress::Discard)
            .map_err(GitError::FetchConnect)?
            .prepare_fetch(Default::default())
            .map_err(GitError::FetchHandshake)?
            .with_dry_run(true)
            .receive(&gix::interrupt::IS_INTERRUPTED)
            .map_err(GitError::FetchReceive)
    }
}
