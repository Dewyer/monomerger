use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GitCommit {
    pub branch: String,
    pub hash: String,
    pub parent_hash: Option<String>,
    pub author_name: String,
    pub author_email: String,
    pub time: DateTime<Utc>,
    pub committer_name: String,
    pub committer_email: String,
    pub commit_message: String,
}

#[derive(Debug)]
pub struct GitTree {
    pub commits: Vec<GitCommit>,
    pub lookup: HashMap<String, usize>,
}

#[derive(Debug)]
pub struct GitRepoInput {
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub struct GitRepo {
    pub name: String,
    pub url: String,
    pub tree: GitTree,
    pub head: String,
}
