use std::fs::{create_dir_all, write};

use crate::utils::{
    env::{datadir, model},
    runner::execute_pending_deployments,
};

mod utils;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Create data directories
    {
        let dir = datadir();
        if let Err(e) = create_dir_all(&dir) {
            log::error!(
                "Could not create data dir at {dir}: {e}",
                dir = dir.display()
            )
        };
    }

    // Write settings
    {
        let path = datadir().join(".aider.model.settings.yml");
        if let Err(e) = write(
            &path,
            format!(
                "\
- name: ollama_chat/{model}
    ",
                model = model()
            ),
        ) {
            log::error!(
                "Could not set model settings at {path}: {e}",
                path = path.display()
            )
        };
    }

    execute_pending_deployments().await;
}
