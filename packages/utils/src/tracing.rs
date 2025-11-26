use std::sync::LazyLock;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::path::repo_root;

static TRACING_INIT: LazyLock<std::sync::Mutex<bool>> =
    LazyLock::new(|| std::sync::Mutex::new(false));
static ENV_INIT: LazyLock<std::sync::Mutex<bool>> = LazyLock::new(|| std::sync::Mutex::new(false));

// just initialize once for all threads
pub fn tracing_init() {
    let mut init = TRACING_INIT.lock().unwrap();

    if !*init {
        *init = true;

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .without_time()
                    .with_target(false),
            )
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .try_init()
            .unwrap();
    }
}

pub fn env_init() {
    let mut init = ENV_INIT.lock().unwrap();

    if !*init {
        *init = true;

        let path = repo_root().expect("could not get repo root").join(".env");

        dotenvy::from_path(path).ok();
    }
}
