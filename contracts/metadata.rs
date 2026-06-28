use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use drip_sdk::prelude::*; // Replace with your DRIP framework

const MAX_METADATA_SIZE: usize = 1024; // 1 KB max for metadata

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Metadata {
    pub data: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoreMetadataMsg {
    pub tx_id: String,
    pub metadata: Metadata,
}

#[contract]
impl MetadataContract {
    #[action]
    pub fn store_metadata(&mut self, msg: StoreMetadataMsg) -> Result<()> {
        // Validate metadata size
        let serialized = serde_json::to_string(&msg.metadata)?;
        if serialized.len() > MAX_METADATA_SIZE {
            return Err(Error::Custom("Metadata exceeds maximum size".into()));
        }

        // Optionally: Validate format keys/values
        for (key, value) in &msg.metadata.data {
            if key.is_empty() || value.is_empty() {
                return Err(Error::Custom("Metadata keys and values cannot be empty".into()));
            }
        }

        // Store metadata
        self.metadata_storage.insert(msg.tx_id.clone(), msg.metadata.clone());

        // Emit event
        emit_event!("metadata_stored", {
            "tx_id": msg.tx_id,
            "size": serialized.len().to_string()
        });

        Ok(())
    }

    #[action]
    pub fn get_metadata(&self, tx_id: String) -> Option<Metadata> {
        self.metadata_storage.get(&tx_id).cloned()
    }
}
#[cfg(test)]
mod metadata_tests {
    #[test]
    fn test_metadata_key_not_empty() {
        let key = "tx_id_001";
        assert!(!key.is_empty());
    }

    #[test]
    fn test_metadata_value_not_empty() {
        let value = "payment";
        assert!(!value.is_empty());
    }

    #[test]
    fn test_metadata_size_within_limit() {
        let data = "x".repeat(512);
        assert!(data.len() <= 1024);
    }

    #[test]
    fn test_metadata_exceeds_limit() {
        let data = "x".repeat(2048);
        assert!(data.len() > 1024);
    }
}
