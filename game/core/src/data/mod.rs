use directories::ProjectDirs;
use std::path::PathBuf;

pub fn save_dir() -> PathBuf {
    let project_dirs = ProjectDirs::from(
        "com",          // qualifier
        "somerandomdev", // organization
        "agameproject", // application
    )
    .expect("Failed to get project directory");

    project_dirs.data_dir().to_path_buf()
}
