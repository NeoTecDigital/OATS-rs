use crate::objects::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Trait identifier
pub type TraitId = Uuid;

/// A trait represents immutable domain state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trait {
    /// Unique identifier for this trait
    pub id: TraitId,
    /// Name of the trait
    pub name: String,
    /// Version of the trait
    pub version: u32,
    /// The actual trait data
    pub data: TraitData,
    /// Metadata about the trait
    pub metadata: HashMap<String, String>,
}

/// The actual data contained in a trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraitData {
    /// Simple string value
    String(String),
    /// Numeric value
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Complex structured data
    Object(HashMap<String, serde_json::Value>),
    /// Array of values
    Array(Vec<serde_json::Value>),
    /// Binary data
    Binary(Vec<u8>),
    /// Reference to another object.
    ///
    /// Vocabularies are full of `<kind>` references. Without this variant a
    /// reference has to degrade to a `String`, which loses the fact that it is
    /// a reference and cannot be resolved against a registry.
    Ref(ObjectId),
}

impl Trait {
    /// Create a new trait with the given name and data
    #[inline]
    pub fn new(name: impl Into<String>, data: TraitData) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: 1,
            data,
            metadata: HashMap::new(),
        }
    }

    /// Create a new trait with metadata
    pub fn with_metadata(
        name: impl Into<String>,
        data: TraitData,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: 1,
            data,
            metadata,
        }
    }

    /// Create a new trait with pre-allocated capacity
    pub fn with_capacity(
        name: impl Into<String>,
        data: TraitData,
        metadata_capacity: usize,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: 1,
            data,
            metadata: HashMap::with_capacity(metadata_capacity),
        }
    }

    /// Get the trait name
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the trait data
    #[inline]
    pub fn data(&self) -> &TraitData {
        &self.data
    }

    /// Get mutable trait data
    #[inline]
    pub fn data_mut(&mut self) -> &mut TraitData {
        &mut self.data
    }

    /// Get the trait ID
    #[inline]
    pub fn id(&self) -> TraitId {
        self.id
    }

    /// Get the trait version
    #[inline]
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Get a metadata value
    #[inline]
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Set a metadata value
    #[inline]
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Create a new version of this trait
    pub fn new_version(&self, data: TraitData) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: self.name.clone(),
            version: self.version + 1,
            data,
            metadata: self.metadata.clone(),
        }
    }

    /// Get metadata count
    #[inline]
    pub fn metadata_count(&self) -> usize {
        self.metadata.len()
    }

    /// Reserve capacity for metadata
    #[inline]
    pub fn reserve_metadata(&mut self, additional: usize) {
        self.metadata.reserve(additional);
    }

    /// Clear all metadata
    #[inline]
    pub fn clear_metadata(&mut self) {
        self.metadata.clear();
    }
}

impl TraitData {
    /// Check if this trait data is a string
    pub fn is_string(&self) -> bool {
        matches!(self, TraitData::String(_))
    }

    /// Check if this trait data is a number
    pub fn is_number(&self) -> bool {
        matches!(self, TraitData::Number(_))
    }

    /// Check if this trait data is a boolean
    pub fn is_boolean(&self) -> bool {
        matches!(self, TraitData::Boolean(_))
    }

    /// Check if this trait data is an object
    pub fn is_object(&self) -> bool {
        matches!(self, TraitData::Object(_))
    }

    /// Check if this trait data is an array
    pub fn is_array(&self) -> bool {
        matches!(self, TraitData::Array(_))
    }

    /// Check if this trait data is binary
    pub fn is_binary(&self) -> bool {
        matches!(self, TraitData::Binary(_))
    }

    /// Check if this trait data is a reference to another object
    pub fn is_ref(&self) -> bool {
        matches!(self, TraitData::Ref(_))
    }

    /// Try to get the string value
    pub fn as_string(&self) -> Option<&String> {
        match self {
            TraitData::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get the number value
    pub fn as_number(&self) -> Option<f64> {
        match self {
            TraitData::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to get the boolean value
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            TraitData::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get the object value
    pub fn as_object(&self) -> Option<&HashMap<String, serde_json::Value>> {
        match self {
            TraitData::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Try to get the array value
    pub fn as_array(&self) -> Option<&Vec<serde_json::Value>> {
        match self {
            TraitData::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Try to get the binary value
    pub fn as_binary(&self) -> Option<&Vec<u8>> {
        match self {
            TraitData::Binary(b) => Some(b),
            _ => None,
        }
    }

    /// Try to get the referenced object id
    pub fn as_ref_id(&self) -> Option<ObjectId> {
        match self {
            TraitData::Ref(id) => Some(*id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_creation() {
        let trait_data = TraitData::String("test".to_string());
        let trait_obj = Trait::new("test_trait", trait_data);

        assert_eq!(trait_obj.name(), "test_trait");
        assert_eq!(trait_obj.version, 1);
        assert!(trait_obj.data.is_string());
    }

    #[test]
    fn test_trait_data_methods() {
        let string_data = TraitData::String("hello".to_string());
        let number_data = TraitData::Number(42.0);
        let bool_data = TraitData::Boolean(true);

        assert!(string_data.is_string());
        assert!(number_data.is_number());
        assert!(bool_data.is_boolean());

        assert_eq!(string_data.as_string(), Some(&"hello".to_string()));
        assert_eq!(number_data.as_number(), Some(42.0));
        assert_eq!(bool_data.as_boolean(), Some(true));
    }

    #[test]
    fn test_trait_data_ref() {
        use crate::objects::Object;

        let supplier = Object::new("acme_supply", "supplier");
        let reference = TraitData::Ref(supplier.id());

        assert!(reference.is_ref());
        assert!(!reference.is_string());
        assert_eq!(reference.as_ref_id(), Some(supplier.id()));
        assert_eq!(reference.as_string(), None);

        // A reference must stay a reference, not degrade to a string.
        let as_string = TraitData::String(supplier.id().to_string());
        assert!(!as_string.is_ref());
        assert_eq!(as_string.as_ref_id(), None);
    }

    #[test]
    fn test_trait_data_ref_round_trips() {
        use crate::objects::Object;

        let supplier = Object::new("acme_supply", "supplier");
        let trait_obj = Trait::new("supplier", TraitData::Ref(supplier.id()));

        let json = serde_json::to_string(&trait_obj).expect("serialize");
        let restored: Trait = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.data().as_ref_id(), Some(supplier.id()));
    }

    #[test]
    fn test_trait_metadata() {
        let mut trait_obj = Trait::new("test", TraitData::String("value".to_string()));
        trait_obj.set_metadata("key", "value");

        assert_eq!(trait_obj.get_metadata("key"), Some(&"value".to_string()));
        assert_eq!(trait_obj.get_metadata("nonexistent"), None);
    }
}
