use crate::api::sync::{SyncedEdge, SyncedNode};
use crate::sync::VectorClock;
use anyhow::Result;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Conflict resolution strategies for graph synchronization
pub struct ConflictResolver {
    instance_id: String,
}

impl ConflictResolver {
    pub fn new(instance_id: String) -> Self {
        Self { instance_id }
    }

    /// Resolve a node conflict using vector clock merge and property reconciliation
    pub fn resolve_node_conflict(
        &self,
        _incoming: &SyncedNode,
        _our_vector_clock: &mut VectorClock,
    ) -> Result<bool> {
        // For now, we'll implement a simple last-write-wins strategy
        // In a real system, you'd want more sophisticated merging
        // based on timestamps and property-level resolution

        // This is a placeholder - in production you'd:
        // 1. Compare timestamps
        // 2. Merge JSON properties intelligently
        // 3. Use application-specific merge rules
        // 4. Log conflicts for manual review

        // For now, just accept the incoming change
        Ok(true)
    }

    /// Resolve an edge conflict
    pub fn resolve_edge_conflict(
        &self,
        _incoming: &SyncedEdge,
        _our_vector_clock: &mut VectorClock,
    ) -> Result<bool> {
        // Similar to node conflict resolution
        Ok(true)
    }

    /// Merge two JSON objects, preferring newer values based on timestamps
    #[allow(dead_code)]
    pub fn merge_json_properties(
        &self,
        local: &JsonValue,
        remote: &JsonValue,
        local_timestamp: chrono::DateTime<chrono::Utc>,
        remote_timestamp: chrono::DateTime<chrono::Utc>,
    ) -> JsonValue {
        match (local, remote) {
            (JsonValue::Object(local_map), JsonValue::Object(remote_map)) => {
                let mut merged = serde_json::Map::new();

                // Start with local properties
                for (key, value) in local_map {
                    merged.insert(key.clone(), value.clone());
                }

                // Merge remote properties
                for (key, remote_value) in remote_map {
                    if let Some(local_value) = local_map.get(key) {
                        // Key exists in both - recursively merge or use timestamp
                        if local_value.is_object() && remote_value.is_object() {
                            merged.insert(
                                key.clone(),
                                self.merge_json_properties(
                                    local_value,
                                    remote_value,
                                    local_timestamp,
                                    remote_timestamp,
                                ),
                            );
                        } else {
                            // Use timestamp to decide
                            if remote_timestamp > local_timestamp {
                                merged.insert(key.clone(), remote_value.clone());
                            }
                        }
                    } else {
                        // Key only in remote, add it
                        merged.insert(key.clone(), remote_value.clone());
                    }
                }

                JsonValue::Object(merged)
            }
            (JsonValue::Array(local_arr), JsonValue::Array(remote_arr)) => {
                // For arrays, merge and deduplicate
                let mut merged_arr = local_arr.clone();
                for item in remote_arr {
                    if !merged_arr.contains(item) {
                        merged_arr.push(item.clone());
                    }
                }
                JsonValue::Array(merged_arr)
            }
            _ => {
                // For scalar values, use timestamp
                if remote_timestamp > local_timestamp {
                    remote.clone()
                } else {
                    local.clone()
                }
            }
        }
    }

    /// Detect semantic conflicts (application-specific logic)
    #[allow(dead_code)]
    pub fn detect_semantic_conflicts(
        &self,
        local: &SyncedNode,
        remote: &SyncedNode,
    ) -> Vec<String> {
        let mut conflicts = Vec::new();

        // Example: Check if critical fields differ
        if local.label != remote.label {
            conflicts.push(format!(
                "Label mismatch: '{}' vs '{}'",
                local.label, remote.label
            ));
        }

        if local.node_type != remote.node_type {
            conflicts.push(format!(
                "Node type mismatch: {:?} vs {:?}",
                local.node_type, remote.node_type
            ));
        }

        conflicts
    }

    /// Apply a merge strategy based on node type
    #[allow(dead_code)]
    pub fn apply_type_specific_merge(
        &self,
        node_type: &str,
        local: &JsonValue,
        remote: &JsonValue,
    ) -> JsonValue {
        // Application-specific merge rules
        match node_type {
            "entity" => {
                // For entities, merge properties but preserve local identifiers
                self.merge_preserving_keys(local, remote, &["id", "created_by"])
            }
            "concept" => {
                // For concepts, prefer remote definitions
                remote.clone()
            }
            "fact" => {
                // For facts, combine evidence from both
                self.merge_combining_arrays(local, remote, &["evidence", "sources"])
            }
            _ => {
                // Default: prefer newer (remote in conflict scenarios)
                remote.clone()
            }
        }
    }

    fn merge_preserving_keys(
        &self,
        local: &JsonValue,
        remote: &JsonValue,
        preserve_keys: &[&str],
    ) -> JsonValue {
        if let (JsonValue::Object(local_map), JsonValue::Object(remote_map)) = (local, remote) {
            let mut merged = remote_map.clone();
            for key in preserve_keys {
                if let Some(value) = local_map.get(*key) {
                    merged.insert(key.to_string(), value.clone());
                }
            }
            JsonValue::Object(merged)
        } else {
            remote.clone()
        }
    }

    fn merge_combining_arrays(
        &self,
        local: &JsonValue,
        remote: &JsonValue,
        array_keys: &[&str],
    ) -> JsonValue {
        if let (JsonValue::Object(local_map), JsonValue::Object(remote_map)) = (local, remote) {
            let mut merged = local_map.clone();

            for (key, remote_value) in remote_map {
                if array_keys.contains(&key.as_str()) {
                    // Combine arrays
                    if let Some(JsonValue::Array(local_arr)) = merged.get(key) {
                        if let JsonValue::Array(remote_arr) = remote_value {
                            let mut combined = local_arr.clone();
                            for item in remote_arr {
                                if !combined.contains(item) {
                                    combined.push(item.clone());
                                }
                            }
                            merged.insert(key.clone(), JsonValue::Array(combined));
                        }
                    } else {
                        merged.insert(key.clone(), remote_value.clone());
                    }
                } else {
                    // Overwrite with remote value
                    merged.insert(key.clone(), remote_value.clone());
                }
            }

            JsonValue::Object(merged)
        } else {
            remote.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_json_objects() {
        let resolver = ConflictResolver::new("test-instance".to_string());

        let local = json!({
            "name": "Alice",
            "age": 30,
            "city": "NYC"
        });

        let remote = json!({
            "name": "Alice",
            "age": 31,
            "country": "USA"
        });

        let local_time = chrono::Utc::now();
        let remote_time = local_time + chrono::Duration::seconds(10);

        let merged = resolver.merge_json_properties(&local, &remote, local_time, remote_time);

        assert_eq!(merged["name"], "Alice");
        assert_eq!(merged["age"], 31); // Remote is newer
        assert_eq!(merged["city"], "NYC"); // Preserved from local
        assert_eq!(merged["country"], "USA"); // Added from remote
    }

    #[test]
    fn test_merge_arrays() {
        let resolver = ConflictResolver::new("test-instance".to_string());

        let local = json!(["a", "b", "c"]);
        let remote = json!(["b", "c", "d"]);

        let local_time = chrono::Utc::now();
        let remote_time = local_time + chrono::Duration::seconds(10);

        let merged = resolver.merge_json_properties(&local, &remote, local_time, remote_time);

        if let JsonValue::Array(arr) = merged {
            assert!(arr.contains(&json!("a")));
            assert!(arr.contains(&json!("b")));
            assert!(arr.contains(&json!("c")));
            assert!(arr.contains(&json!("d")));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_preserve_keys() {
        let resolver = ConflictResolver::new("test-instance".to_string());

        let local = json!({
            "id": "123",
            "name": "Local",
            "created_by": "user1"
        });

        let remote = json!({
            "id": "456",
            "name": "Remote",
            "created_by": "user2"
        });

        let merged = resolver.merge_preserving_keys(&local, &remote, &["id", "created_by"]);

        assert_eq!(merged["id"], "123"); // Preserved
        assert_eq!(merged["name"], "Remote"); // From remote
        assert_eq!(merged["created_by"], "user1"); // Preserved
    }
}
