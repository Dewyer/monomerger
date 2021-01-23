extern crate anyhow;

use crate::git::types::*;
use crate::merger::{merge_repositories, MergerConfig};

pub mod git;
pub mod merger;

fn main() {
    merge_repositories(MergerConfig {
        repos: vec![
            GitRepoInput {
                name: "repo1".to_string(),
                url: "repo1.git".to_string(),
            },
            GitRepoInput {
                name: "repo2".to_string(),
                url: "repo3.git".to_string(),
            },
        ],
        into: GitRepoInput {
            name: "into".to_string(),
            url: "into.git".to_string(),
        },
    })
    .unwrap();
}
