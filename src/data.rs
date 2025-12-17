use std::path::PathBuf;
use tokio::sync::OnceCell;

static DATA_PATH: OnceCell<PathBuf> = OnceCell::const_new();

pub fn get_data_path() -> &'static PathBuf {
    DATA_PATH.get().expect("DATA_PATH not initialized")
}

pub async fn set_data_path(path: PathBuf) {
    DATA_PATH.set(path).expect("DATA_PATH already set");
}
