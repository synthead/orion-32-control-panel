use std::path::Path;

use crate::constants::FilePaths;

pub(crate) struct OrionPaths {
    config_dir_path: std::path::PathBuf,
}

impl OrionPaths {
    pub(crate) fn new() -> Self {
        let config_dir_path = directories::BaseDirs::new()
            .expect("Failed to resolve local config directory for OS")
            .config_dir()
            .join(FilePaths::CONFIG_DIR);

        Self { config_dir_path }
    }

    pub(crate) fn config_dir(&self) -> &Path {
        let config_dir_path = &self.config_dir_path;

        std::fs::create_dir_all(config_dir_path).unwrap_or_else(|error| {
            panic!(
                "Failed to create config directory '{}': {error}",
                config_dir_path.display()
            )
        });

        config_dir_path
    }

    pub(crate) fn persistence_file_path(&self) -> std::path::PathBuf {
        self.config_dir().join(FilePaths::PERSISTENCE_FILE)
    }

    pub(crate) fn create_file_dialog(&self) -> rfd::FileDialog {
        let config_dir = self.config_dir();

        rfd::FileDialog::new()
            .add_filter(
                FilePaths::ROUTING_FILE_DESCRIPTION,
                FilePaths::ROUTING_FILE_EXTENSIONS,
            )
            .set_directory(config_dir)
    }
}
