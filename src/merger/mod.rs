use crate::git::run_command::{run_command, run_command_with_envs};
use crate::git::tree::build_tree_for_repo;
use crate::git::types::{GitCommit, GitRepo, GitRepoInput};
use anyhow::Result;
use fs_extra::dir::CopyOptions;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs;

pub struct MergerConfig {
    pub repos: Vec<GitRepoInput>,
    pub into: GitRepoInput,
}

fn create_tmp_dir() -> Result<()> {
    let tmp_path = "./tmp";
    if let Ok(meta) = fs::metadata(tmp_path) {
        if meta.is_dir() {
            fs::remove_dir_all(tmp_path)?;
        } else if meta.is_file() {
            fs::remove_file(tmp_path)?;
        }
    }

    fs::create_dir(tmp_path)?;
    Ok(())
}

fn get_repo_path(repo: &GitRepo) -> String {
    format!("./tmp/{}", repo.name)
}

fn get_repo_dest_path(into: &GitRepo, repo: &GitRepo) -> String {
    format!("./tmp/{}/{}", into.name, repo.name)
}

fn clone_repo_to_temp(repo: &GitRepoInput) -> Result<GitRepo> {
    println!("Cloning repo : {} ...", &repo.url);
    run_command("git", "./tmp", &vec!["clone", &repo.url])?;
    println!("Building tree : {} ...", &repo.name);
    let tree = build_tree_for_repo(&format!("./tmp/{}", repo.name))?;
    let head = if tree.commits.len() > 0 {
        tree.commits[0].hash.clone()
    } else {
        "main".to_string()
    };

    println!("Repo prepared {}", &repo.name);
    Ok(GitRepo {
        name: repo.name.clone(),
        url: repo.url.clone(),
        tree,
        head,
    })
}

fn build_full_commit_history(repos: &Vec<GitRepo>) -> Vec<(usize, GitCommit)> {
    let mut full = Vec::new();
    for (ii, repo) in repos.iter().enumerate() {
        for commit in repo.tree.commits.iter() {
            full.push((ii, commit.clone()));
        }
    }

    full.sort_by(|aa, bb| aa.1.time.timestamp().cmp(&bb.1.time.timestamp()));
    for aa in full.iter() {
        println!("b:{}", &aa.1.branch);
    }
    full
}

fn checkout_branch(repo: &mut GitRepo, branch: &str, hash: bool) -> Result<()> {
    println!("Trying to switch out: {} - on: {}", branch, &repo.name);
    run_command(
        "git",
        &get_repo_path(repo),
        &if hash {
            vec!["checkout", branch]
        } else {
            vec!["checkout", "-B", branch]
        },
    )?;
    repo.head = branch.to_string();
    Ok(())
}

fn get_into_branch(commit: &GitCommit, repo: &GitRepo) -> String {
    commit.branch.clone()
}

fn get_into_parent_branch(commit: &GitCommit, repo: &GitRepo) -> String {
    if let Some(parent_hash) = &commit.parent_hash {
        println!("parent : '{}'", parent_hash);
        let par_ind = repo.tree.lookup.get(parent_hash).unwrap().clone();
        let parent_commit = &repo.tree.commits[par_ind];
        get_into_branch(parent_commit, repo)
    } else {
        "main".to_string()
    }
}

fn fs_recursive_delete(path: &str) -> Result<()> {
    for dir_entry in fs::read_dir(path).expect("expected to be a dir source") {
        let dir_entry = dir_entry?;
        if !dir_entry.path().ends_with(".git") {
            fs_extra::remove_items(&[dir_entry.path()])?;
        }
    }

    Ok(())
}

fn fs_recursive_copy(from: &str, dest_path: &str) -> Result<()> {
    for dir_entry in fs::read_dir(from).expect("expected to be a dir source") {
        let dir_entry = dir_entry?;
        if !dir_entry.path().ends_with(".git") {
            fs_extra::copy_items(&[dir_entry.path()], &dest_path, &CopyOptions::default())?;
        }
    }

    Ok(())
}

fn git_stage_all(repo: &GitRepo) -> Result<()> {
    run_command("git", &get_repo_path(repo), &vec!["add", "-A"])?;

    Ok(())
}

fn git_commit(repo: &GitRepo, commit: &GitCommit) -> Result<()> {
    let time = format!("{}+0000", &commit.time.timestamp());
    let mut envs = HashMap::new();
    let vars = vec![
        ("GIT_AUTHOR_NAME", &commit.author_name),
        ("GIT_AUTHOR_EMAIL", &commit.author_name),
        ("GIT_AUTHOR_DATE", &time),
        ("GIT_COMMITTER_NAME", &commit.author_name),
        ("GIT_COMMITTER_EMAIL", &commit.author_name),
        ("GIT_COMMITTER_DATE", &time),
    ];

    for var in vars {
        envs.insert(var.0, var.1.borrow());
    }

    run_command_with_envs(
        "git",
        &get_repo_path(repo),
        &vec!["commit", "--allow-empty", "-m", &commit.commit_message],
        &envs,
    )?;

    Ok(())
}

fn get_last_commit_on_branch_with_other_commit<'a>(
    repo: &'a GitRepo,
    other: &GitCommit,
) -> &'a GitCommit {
    for commit in repo.tree.commits.iter() {
        panic!("asd");
    }
}

pub fn merge_repositories(cfg: MergerConfig) -> Result<()> {
    println!("Mono merger in action !");
    create_tmp_dir()?;
    let mut repos = Vec::new();
    let mut into_repo = clone_repo_to_temp(&cfg.into)?;

    for repo in cfg.repos.iter() {
        let new_repo = clone_repo_to_temp(repo)?;
        let pth = get_repo_dest_path(&into_repo, &new_repo);
        fs::create_dir_all(&pth)?;
        repos.push(new_repo);
    }

    checkout_branch(&mut into_repo, "main", false)?;

    let full_history = build_full_commit_history(&repos);
    for (repo_ind, commit) in full_history {
        println!("AC: {:?}", &commit);
        let at_repo = &mut repos[repo_ind];
        let parent_branch = get_into_parent_branch(&commit, at_repo);
        if into_repo.head != parent_branch {
            checkout_branch(&mut into_repo, &parent_branch, false)?;
        }

        let at_into_branch = get_into_branch(&commit, at_repo);
        if into_repo.head != at_into_branch {
            checkout_branch(&mut into_repo, &parent_branch, false)?;
        }

        checkout_branch(at_repo, &commit.hash, true)?;
        // checkout last branches from other repos

        fs_recursive_delete(&get_repo_dest_path(&into_repo, &at_repo))?;
        fs_recursive_copy(
            &get_repo_path(&at_repo),
            &get_repo_dest_path(&into_repo, &at_repo),
        )?;

        git_stage_all(&into_repo)?;
        println!(
            "Committing: {} - from: {}",
            commit.commit_message, at_repo.name
        );
        git_commit(&into_repo, &commit)?;
    }

    Ok(())
}
