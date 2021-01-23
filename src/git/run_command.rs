use anyhow::Result;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};

fn spawn_child(
    name: &str,
    work_dir: &str,
    args: &Vec<&str>,
    envs: &HashMap<&str, &str>,
) -> Result<Child> {
    Ok(Command::new(name)
        .current_dir(work_dir)
        .args(args)
        .envs(envs)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?)
}

pub fn get_output_from_child(res: Child) -> Result<String> {
    let output = res.wait_with_output()?;
    if output.status.success() {
        let raw_output = String::from_utf8(output.stdout)?;
        Ok(raw_output)
    } else {
        let err = String::from_utf8(output.stderr)?;
        let raw_output = String::from_utf8(output.stdout)?;
        Err(anyhow::anyhow!(format!("{} {}", &err, &raw_output)))
    }
}

pub fn run_command_with_envs(
    name: &str,
    work_dir: &str,
    args: &Vec<&str>,
    envs: &HashMap<&str, &str>,
) -> Result<String> {
    let res = spawn_child(name, work_dir, args, envs)?;
    get_output_from_child(res)
}

pub fn run_command(name: &str, work_dir: &str, args: &Vec<&str>) -> Result<String> {
    let res = spawn_child(name, work_dir, args, &HashMap::new())?;
    get_output_from_child(res)
}
