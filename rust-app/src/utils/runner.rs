use std::{
    fs::{read_to_string, remove_dir_all, write},
    process::{Command, Stdio},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tokio::time::timeout;

use crate::utils::env::{aider, datadir, git, model, npm, projectsdir};

#[derive(Deserialize)]
struct Assignment {
    project: String,
    instructions: String,
    version: Option<String>,
}

#[derive(Serialize, Debug)]
struct Output {
    git_hash: String,
}

pub async fn execute_pending_deployment() {
    let path: std::path::PathBuf = datadir().join("assignment.json");
    if let Some(assignment) = read_to_string(&path)
        .ok()
        .and_then(|assignment| serde_json::from_str::<Assignment>(&assignment).ok())
    {
        let path = projectsdir().join(&assignment.project);

        let mut cli_command = Command::new(format!("{}git", git()));
        cli_command
            .arg("clone")
            .arg(format!(
                "github:miniapp-factory/{project}",
                project = assignment.project
            ))
            .arg(&path);
        if let Err(e) = cli_command.output() {
            panic!(
                "Could not clone remote repo to {path}: {e}",
                path = path.display()
            );
        }

        if let Some(version) = &assignment.version {
            let mut cli_command = Command::new(format!("{}git", git()));
            cli_command
                .arg("-C")
                .arg(&path)
                .arg("reset")
                .arg("--hard")
                .arg(version);
            if let Err(e) = cli_command.output() {
                log::error!(
                    "Could not reset {path} to {version}: {e}",
                    path = path.display()
                );
            }
        }

        let project_path = path.join("mini-app");
        let mut cli_command = tokio::process::Command::new(format!("{}aider", aider()));
        cli_command
            .env("OLLAMA_API_BASE", "http://127.0.0.1:11434")
            .env("HOME", datadir())
            .current_dir(&project_path)
            .arg("--model")
            .arg(format!("ollama_chat/{model}", model = model()))
            .arg("--model-settings-file")
            .arg(datadir().join(".aider.model.settings.yml"))
            .arg("--restore-chat-history")
            .arg("--no-gitignore")
            .arg("--test-cmd")
            .arg(format!(
                "{npm} i --cwd {path} --no-save && {npm} run --cwd {path} build",
                path = project_path.display(),
                npm = npm()
            ))
            .arg("--auto-test")
            .arg("--read")
            .arg(path.join("documentation").join("index.md"))
            .arg("--file")
            .arg(project_path.join("lib").join("metadata.ts"))
            .arg("--disable-playwright")
            .arg("--no-detect-urls")
            .arg("--no-suggest-shell-commands")
            .arg("--edit-format")
            .arg("diff")
            .arg("--message")
            .arg(&assignment.instructions);
        match cli_command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(mut child) => {
                match timeout(Duration::from_secs(20 * 60), child.wait()).await {
                    Ok(output) => {
                        if let Err(e) = output {
                            log::error!(
                                "Could not perform requested change {instructions} on {project}: {e}",
                                instructions = assignment.instructions,
                                project = assignment.project
                            );
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "Hit the timeout for requested change {instructions} on {project}: {e}",
                            instructions = assignment.instructions,
                            project = assignment.project
                        );
                        if let Err(e) = child.kill().await {
                            log::error!("Could not kill child process: {e}");
                        }
                    }
                };
            }
            Err(e) => {
                log::error!(
                    "Could not spawn child for requested change {instructions} on {project}: {e}",
                    instructions = assignment.instructions,
                    project = assignment.project
                );
            }
        }

        let mut cli_command = Command::new(format!("{}git", git()));
        cli_command.arg("-C").arg(&path).arg("add").arg("-A");
        if let Err(e) = cli_command.output() {
            log::error!(
                "Could not add aider chat history to {path}: {e}",
                path = path.display()
            );
        }

        let mut cli_command = Command::new(format!("{}git", git()));
        cli_command
            .arg("-C")
            .arg(&path)
            .arg("commit")
            .arg("-m")
            .arg("aider chat history");
        if let Err(e) = cli_command.output() {
            log::error!(
                "Could not commit aider chat history to {path}: {e}",
                path = path.display()
            );
        }

        let mut cli_command = Command::new(format!("{}git", git()));
        cli_command.arg("-C").arg(&path).arg("push").arg("-f");
        if let Err(e) = cli_command.output() {
            log::error!(
                "Could not push {path} to remote repo: {e}",
                path = path.display()
            );
        }

        let mut cli_command = Command::new(format!("{}git", git()));
        cli_command
            .arg("-C")
            .arg(&path)
            .arg("rev-parse")
            .arg("HEAD");
        let git_hash = match cli_command.output() {
            Ok(output) => match str::from_utf8(&output.stdout) {
                Ok(git_hash) => git_hash.to_string(),
                Err(e) => {
                    panic!(
                        "Could convert git hash of {path} to utf8 string: {e}",
                        path = path.display()
                    );
                }
            },
            Err(e) => {
                panic!(
                    "Could not get git hash of {path}: {e}",
                    path = path.display()
                );
            }
        };

        if let Err(e) = remove_dir_all(&path) {
            log::error!("Could not remove {path}: {e}", path = path.display());
        }

        let output = Output { git_hash };
        let file_content = match serde_json::to_string(&output) {
            Ok(file_content) => file_content,
            Err(e) => {
                panic!("Could not convert {output:?} to string: {e}");
            }
        };

        if let Err(e) = write(datadir().join("assignment.json"), &file_content) {
            log::error!(
                "Could not write {file_content} to {path}: {e}",
                path = path.display()
            );
        }
    }
}
