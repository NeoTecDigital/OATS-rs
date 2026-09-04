use crate::traits::{Trait, TraitId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Object identifier
pub type ObjectId = Uuid;

/// An object is an identity container that composes traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    /// Unique identifier for this object
    pub id: ObjectId,
    /// Name of the object
    pub name: String,
    /// Type of the object
    pub object_type: String,
    /// Traits associated with this object
    pub traits: HashMap<String, Trait>,
    /// Metadata about the object
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Object {
    /// Create a new object with the given name and type
    #[inline]
    pub fn new(name: impl Into<String>, object_type: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            object_type: object_type.into(),
            traits: HashMap::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new object with initial traits
    pub fn with_traits(
        name: impl Into<String>,
        object_type: impl Into<String>,
        traits: Vec<Trait>,
    ) -> Self {
        let mut obj = Self::new(name, object_type);
        obj.add_traits_bulk(traits);
        obj
    }

    /// Create a new object with pre-allocated capacity
    pub fn with_capacity(
        name: impl Into<String>,
        object_type: impl Into<String>,
        trait_capacity: usize,
        metadata_capacity: usize,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            object_type: object_type.into(),
            traits: HashMap::with_capacity(trait_capacity),
            metadata: HashMap::with_capacity(metadata_capacity),
            created_at: now,
            updated_at: now,
        }
    }

    /// Get the object name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the object type
    pub fn object_type(&self) -> &str {
        &self.object_type
    }

    /// Get the object ID
    pub fn id(&self) -> ObjectId {
        self.id
    }

    /// Add a trait to this object
    pub fn add_trait(&mut self, trait_obj: Trait) {
        self.traits.insert(trait_obj.name().to_string(), trait_obj);
        self.updated_at = chrono::Utc::now();
    }

    /// Add multiple traits efficiently
    pub fn add_traits(&mut self, traits: impl IntoIterator<Item = Trait>) {
        let mut updated = false;
        for trait_obj in traits {
            self.traits.insert(trait_obj.name().to_string(), trait_obj);
            updated = true;
        }
        if updated {
            self.updated_at = chrono::Utc::now();
        }
    }

    /// Add multiple traits without timestamp updates (for bulk operations)
    pub fn add_traits_bulk(&mut self, traits: impl IntoIterator<Item = Trait>) {
        for trait_obj in traits {
            self.traits.insert(trait_obj.name().to_string(), trait_obj);
        }
        // Don't update timestamp for bulk operations
    }

    /// Add a trait without timestamp update (for internal operations)
    pub fn add_trait_internal(&mut self, trait_obj: Trait) {
        self.traits.insert(trait_obj.name().to_string(), trait_obj);
        // Don't update timestamp for internal operations
    }

    /// Apply a proposed trait value, superseding any trait of the same name.
    ///
    /// This is the writeback primitive: actions propose a [`Trait`] and the host
    /// calls this to accept it. The applied trait's version is set to one past
    /// the version it supersedes, so versions stay monotonic across a chain of
    /// writebacks. Returns the superseded trait, if there was one.
    pub fn apply_trait(&mut self, mut trait_obj: Trait) -> Option<Trait> {
        if let Some(existing) = self.traits.get(trait_obj.name()) {
            trait_obj.version = existing.version.saturating_add(1);
        }
        let superseded = self.traits.insert(trait_obj.name().to_string(), trait_obj);
        self.updated_at = chrono::Utc::now();
        superseded
    }

    /// Remove a trait from this object
    #[inline]
    pub fn remove_trait(&mut self, trait_name: &str) -> Option<Trait> {
        let result = self.traits.remove(trait_name);
        if result.is_some() {
            self.updated_at = chrono::Utc::now();
        }
        result
    }

    /// Get a trait by name
    #[inline]
    pub fn get_trait(&self, trait_name: &str) -> Option<&Trait> {
        self.traits.get(trait_name)
    }

    /// Get a trait by name (mutable)
    #[inline]
    pub fn get_trait_mut(&mut self, trait_name: &str) -> Option<&mut Trait> {
        self.traits.get_mut(trait_name)
    }

    /// Get trait data by name (zero-copy access)
    #[inline]
    pub fn get_trait_data(&self, trait_name: &str) -> Option<&crate::traits::TraitData> {
        self.traits.get(trait_name).map(|t| t.data())
    }

    /// Get trait data by name (mutable, zero-copy access)
    #[inline]
    pub fn get_trait_data_mut(
        &mut self,
        trait_name: &str,
    ) -> Option<&mut crate::traits::TraitData> {
        self.traits.get_mut(trait_name).map(|t| t.data_mut())
    }

    /// Get all traits
    #[inline]
    pub fn traits(&self) -> &HashMap<String, Trait> {
        &self.traits
    }

    /// Check if the object has a specific trait
    #[inline]
    pub fn has_trait(&self, trait_name: &str) -> bool {
        self.traits.contains_key(trait_name)
    }

    /// Check if the object has multiple traits (efficient batch check)
    #[inline]
    pub fn has_traits(&self, trait_names: &[&str]) -> bool {
        trait_names
            .iter()
            .all(|name| self.traits.contains_key(*name))
    }

    /// Check if the object has any traits
    #[inline]
    pub fn has_any_traits(&self) -> bool {
        !self.traits.is_empty()
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
        self.updated_at = chrono::Utc::now();
    }

    /// Get all metadata
    #[inline]
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Get the creation timestamp
    #[inline]
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }

    /// Get the last update timestamp
    #[inline]
    pub fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.updated_at
    }

    /// Get trait names as a vector
    #[inline]
    pub fn trait_names(&self) -> Vec<&String> {
        self.traits.keys().collect()
    }

    /// Get trait IDs as a vector
    #[inline]
    pub fn trait_ids(&self) -> Vec<TraitId> {
        self.traits.values().map(|t| t.id).collect()
    }

    /// Get the number of traits
    #[inline]
    pub fn trait_count(&self) -> usize {
        self.traits.len()
    }

    /// Get the number of metadata entries
    #[inline]
    pub fn metadata_count(&self) -> usize {
        self.metadata.len()
    }

    /// Reserve capacity for traits
    #[inline]
    pub fn reserve_traits(&mut self, additional: usize) {
        self.traits.reserve(additional);
    }

    /// Reserve capacity for metadata
    #[inline]
    pub fn reserve_metadata(&mut self, additional: usize) {
        self.metadata.reserve(additional);
    }

    /// Validate that the object has required traits
    pub fn validate_required_traits(
        &self,
        required_traits: &[&str],
    ) -> Result<(), crate::OatsError> {
        let missing: Vec<_> = required_traits
            .iter()
            .filter(|trait_name| !self.has_trait(trait_name))
            .map(|s| s.to_string())
            .collect();

        if !missing.is_empty() {
            return Err(crate::OatsError::trait_not_found(format!(
                "Missing required traits: {}",
                missing.join(", ")
            )));
        }
        Ok(())
    }

    /// Check if the object is valid (has required fields)
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.object_type.is_empty()
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Object {}

impl std::hash::Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{Trait, TraitData};

    #[test]
    fn test_object_creation() {
        let obj = Object::new("test_object", "test_type");

        assert_eq!(obj.name(), "test_object");
        assert_eq!(obj.object_type(), "test_type");
        assert_eq!(obj.trait_count(), 0);
        assert!(!obj.has_any_traits());
    }

    #[test]
    fn test_object_with_traits() {
        let trait1 = Trait::new("health", TraitData::Number(100.0));
        let trait2 = Trait::new("position", TraitData::Object(HashMap::new()));

        let obj = Object::with_traits("player", "character", vec![trait1, trait2]);

        assert_eq!(obj.trait_count(), 2);
        assert!(obj.has_trait("health"));
        assert!(obj.has_trait("position"));
        assert!(!obj.has_trait("nonexistent"));
    }

    #[test]
    fn test_add_remove_trait() {
        let mut obj = Object::new("test", "type");
        let trait_obj = Trait::new("test_trait", TraitData::String("value".to_string()));

        obj.add_trait_internal(trait_obj);
        assert_eq!(obj.trait_count(), 1);
        assert!(obj.has_trait("test_trait"));

        let removed = obj.remove_trait("test_trait");
        assert!(removed.is_some());
        assert_eq!(obj.trait_count(), 0);
        assert!(!obj.has_trait("test_trait"));
    }

    #[test]
    fn test_apply_trait_supersedes_and_bumps_version() {
        let mut obj = Object::new("john_doe", "customer");
        obj.add_trait(Trait::new("balance", TraitData::Number(500.0)));
        assert_eq!(obj.get_trait("balance").unwrap().version(), 1);

        let superseded = obj.apply_trait(Trait::new("balance", TraitData::Number(400.01)));

        let superseded = superseded.expect("previous balance returned");
        assert_eq!(superseded.data().as_number(), Some(500.0));

        let current = obj.get_trait("balance").expect("balance still present");
        assert_eq!(current.data().as_number(), Some(400.01));
        assert_eq!(current.version(), 2);
        assert_eq!(obj.trait_count(), 1);

        obj.apply_trait(Trait::new("balance", TraitData::Number(300.02)));
        assert_eq!(obj.get_trait("balance").unwrap().version(), 3);
    }

    #[test]
    fn test_apply_trait_adds_when_absent() {
        let mut obj = Object::new("john_doe", "customer");

        assert!(obj
            .apply_trait(Trait::new("balance", TraitData::Number(500.0)))
            .is_none());
        assert_eq!(obj.get_trait("balance").unwrap().version(), 1);
    }

    #[test]
    fn test_metadata() {
        let mut obj = Object::new("test", "type");
        obj.set_metadata("key", "value");

        assert_eq!(obj.get_metadata("key"), Some(&"value".to_string()));
        assert_eq!(obj.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_trait_names_and_ids() {
        let trait1 = Trait::new("health", TraitData::Number(100.0));
        let trait2 = Trait::new("position", TraitData::Object(HashMap::new()));

        let obj = Object::with_traits("player", "character", vec![trait1, trait2]);

        let names = obj.trait_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&&"health".to_string()));
        assert!(names.contains(&&"position".to_string()));

        let ids = obj.trait_ids();
        assert_eq!(ids.len(), 2);
    }
}
