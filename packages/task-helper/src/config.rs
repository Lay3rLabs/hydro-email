use std::path::PathBuf;

use app_utils::path::repo_root;

pub fn path_deployments() -> PathBuf {
    repo_root().unwrap().join(".deployments")
}

pub fn path_builds() -> PathBuf {
    repo_root().unwrap().join(".builds")
}
