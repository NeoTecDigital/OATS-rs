//! The read-only view an action is given: the objects it may inspect plus any
//! parameters the caller passes in.

use crate::Object;
use std::collections::HashMap;

/// Context passed to actions containing relevant objects and traits
#[derive(Debug, Clone)]
pub struct ActionContext {
    /// Objects relevant to this action
    pub objects: HashMap<String, Object>,
    /// Additional parameters for the action
    pub parameters: HashMap<String, serde_json::Value>,
    /// Metadata about the action execution
    pub metadata: HashMap<String, String>,
}

impl ActionContext {
    /// Create a new action context
    #[inline]
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            parameters: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new action context with expected capacity
    #[inline]
    pub fn with_capacity(expected_objects: usize, expected_parameters: usize) -> Self {
        Self {
            objects: HashMap::with_capacity(expected_objects),
            parameters: HashMap::with_capacity(expected_parameters),
            metadata: HashMap::new(),
        }
    }

    /// Add an object to the context
    #[inline]
    pub fn add_object(&mut self, name: impl Into<String>, object: Object) {
        self.objects.insert(name.into(), object);
    }

    /// Get an object from the context
    #[inline]
    pub fn get_object(&self, name: &str) -> Option<&Object> {
        self.objects.get(name)
    }

    /// Get multiple objects efficiently
    #[inline]
    pub fn get_objects(&self, names: &[&str]) -> HashMap<String, &Object> {
        names
            .iter()
            .filter_map(|name| self.objects.get(*name).map(|obj| (name.to_string(), obj)))
            .collect()
    }

    /// Add a parameter to the context
    #[inline]
    pub fn add_parameter(&mut self, name: impl Into<String>, value: serde_json::Value) {
        self.parameters.insert(name.into(), value);
    }

    /// Get a parameter from the context
    #[inline]
    pub fn get_parameter(&self, name: &str) -> Option<&serde_json::Value> {
        self.parameters.get(name)
    }

    /// Add metadata to the context
    #[inline]
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Get metadata from the context
    #[inline]
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Get object count
    #[inline]
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// Get parameter count
    #[inline]
    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    /// Get metadata count
    #[inline]
    pub fn metadata_count(&self) -> usize {
        self.metadata.len()
    }

    /// Reserve capacity for objects
    #[inline]
    pub fn reserve_objects(&mut self, additional: usize) {
        self.objects.reserve(additional);
    }

    /// Reserve capacity for parameters
    #[inline]
    pub fn reserve_parameters(&mut self, additional: usize) {
        self.parameters.reserve(additional);
    }

    /// Clear all objects
    #[inline]
    pub fn clear_objects(&mut self) {
        self.objects.clear();
    }

    /// Clear all parameters
    #[inline]
    pub fn clear_parameters(&mut self) {
        self.parameters.clear();
    }

    /// Clear all metadata
    #[inline]
    pub fn clear_metadata(&mut self) {
        self.metadata.clear();
    }
}

impl Default for ActionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_context() {
        let mut context = ActionContext::new();
        let test_object = Object::new("test", "type");

        context.add_object("test_obj", test_object);
        context.add_parameter("param", serde_json::json!("value"));
        context.add_metadata("key", "value");

        assert!(context.get_object("test_obj").is_some());
        assert!(context.get_parameter("param").is_some());
        assert_eq!(context.get_metadata("key"), Some(&"value".to_string()));
    }
}
