use crate::git::run_command;
use crate::git::types::{GitCommit, GitTree};
use anyhow::Result;
use chrono::{TimeZone, Utc};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::str::{FromStr, Split};

trait NextFallible {
    fn next_anyhow(&mut self) -> Result<String>;
}

impl NextFallible for Split<'_, char> {
    fn next_anyhow(&mut self) -> Result<String> {
        Ok(self
            .next()
            .ok_or(anyhow::anyhow!("next token is none"))?
            .to_string())
    }
}

pub fn build_tree_for_repo(repo_path: &str) -> Result<GitTree> {
    let args = vec![
        "--no-pager",
        "log",
        "--all",
        "--format=\"%S;%H;%P;%an;%ae;%at;%cn;%ce;%s\"",
    ];
    let tree_output = run_command::run_command("git", repo_path, &args)?;
    let mut tree = GitTree {
        commits: Vec::new(),
        lookup: HashMap::new(),
    };

    for (ii, line) in tree_output.lines().enumerate() {
        let mut lp = line.split(';');
        let mut commit = GitCommit {
            branch: lp
                .next_anyhow()?
                .replace("\"refs/heads/", "")
                .replace("\"refs/tags/", "")
                .replace("\"refs/remotes/", "")
                .replace("\"refs/remotes/origin/", "")
                .replace("\"refs/heads/origin/", "")
                .replace("\"refs/tags/origin/", "")
                .replace("origin/", ""),
            hash: lp.next_anyhow()?,
            parent_hash: Some(lp.next_anyhow()?),
            author_name: lp.next_anyhow()?,
            author_email: lp.next_anyhow()?,
            time: Utc.timestamp(i64::from_str(&lp.next_anyhow()?)?, 0),
            committer_name: lp.next_anyhow()?,
            committer_email: lp.next_anyhow()?,
            commit_message: lp.next_anyhow()?.replace("\"", ""),
        };
        commit.parent_hash = match commit
            .parent_hash
            .as_ref()
            .ok_or(anyhow::anyhow!("parent hash missing"))?
            .borrow()
        {
            "" => None,
            _ => Some(
                commit
                    .parent_hash
                    .unwrap()
                    .split(" ")
                    .last()
                    .unwrap()
                    .to_string(),
            ),
        };

        tree.commits.push(commit);
    }

    tree.commits
        .sort_by(|aa, bb| aa.time.timestamp().cmp(&bb.time.timestamp()));

    for (ii, commit) in tree.commits.iter().enumerate() {
        tree.lookup.insert(commit.hash.clone(), ii);
    }

    Ok(tree)
}
