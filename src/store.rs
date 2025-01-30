use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::models::Network;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GStore {
    pub network: Network,
}

impl GStore {
    fn path() -> PathBuf {
        return dirs::data_dir()
            .expect("Unable to get data directory")
            .join("gsqueal/store")
            .with_extension("json");
    }
    pub async fn get() -> Self {
        let store_path = Self::path();

        if store_path.exists() {
            let content = fs::read_to_string(store_path)
                .await
                .expect("Failed to read from store");
            let store: Self = serde_json::from_str(&content).expect("Failed to deserialize data");

            return store;
        } else {
            let store = Self::default();
            store.set().await;

            return store;
        }
    }
    pub async fn set(&self) {
        let store_path = Self::path();

        if let Some(parent) = store_path.parent() {
            fs::create_dir_all(parent)
                .await
                .expect("Failed to create parent directories");
        }

        let serialized_data = serde_json::to_string_pretty(self).expect("Failed to serialize data");
        fs::write(store_path, serialized_data)
            .await
            .expect("Failed to write to store");
    }
}
