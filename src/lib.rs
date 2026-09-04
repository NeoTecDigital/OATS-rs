//! OATS - Objects • Actions • Traits • Systems
//!
//! Universal architecture pattern for infinite scale across any domain.
//!
//! This library provides the core abstractions and implementations for the OATS pattern:
//! - **Objects**: Identity containers that compose traits
//! - **Actions**: Stateless logic that reads traits and returns updates
//! - **Traits**: Immutable state containing domain data
//! - **Systems**: Orchestration that coordinates actions and manages resources

pub mod actions;
pub mod error;
pub mod objects;
pub mod systems;
pub mod traits;

// Re-export main types for convenience
pub use actions::{Action, ActionContext, ActionResult, TraitUpdate};
pub use error::OatsError;
pub use objects::Object;
pub use systems::{ApplyReport, Priority, System, SystemManager};
pub use traits::{Trait, TraitData};

/// Result type for OATS operations
pub type Result<T> = std::result::Result<T, OatsError>;

/// Core OATS system that orchestrates all components
#[derive(Default)]
pub struct OatsSystem {
    objects: Vec<Object>,
    actions: Vec<Box<dyn Action>>,
    systems: Vec<Box<dyn System>>,
}

impl OatsSystem {
    /// Create a new OATS system
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new OATS system with pre-allocated capacity
    pub fn with_capacity(objects: usize, actions: usize, systems: usize) -> Self {
        Self {
            objects: Vec::with_capacity(objects),
            actions: Vec::with_capacity(actions),
            systems: Vec::with_capacity(systems),
        }
    }

    /// Add an object to the system
    #[inline]
    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }

    /// Add multiple objects efficiently
    pub fn add_objects(&mut self, objects: impl IntoIterator<Item = Object>) {
        self.objects.extend(objects);
    }

    /// Add an action to the system
    #[inline]
    pub fn add_action(&mut self, action: Box<dyn Action>) {
        self.actions.push(action);
    }

    /// Add a system to the system
    #[inline]
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    /// Get all objects in the system
    #[inline]
    pub fn objects(&self) -> &[Object] {
        &self.objects
    }

    /// Get all actions in the system
    #[inline]
    pub fn actions(&self) -> &[Box<dyn Action>] {
        &self.actions
    }

    /// Get all systems in the system
    #[inline]
    pub fn systems(&self) -> &[Box<dyn System>] {
        &self.systems
    }

    /// Get object count
    #[inline]
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// Get action count
    #[inline]
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    /// Get system count
    #[inline]
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Clear all objects
    #[inline]
    pub fn clear_objects(&mut self) {
        self.objects.clear();
    }

    /// Clear all actions
    #[inline]
    pub fn clear_actions(&mut self) {
        self.actions.clear();
    }

    /// Clear all systems
    #[inline]
    pub fn clear_systems(&mut self) {
        self.systems.clear();
    }

    /// Reserve capacity for objects
    #[inline]
    pub fn reserve_objects(&mut self, additional: usize) {
        self.objects.reserve(additional);
    }

    /// Reserve capacity for actions
    #[inline]
    pub fn reserve_actions(&mut self, additional: usize) {
        self.actions.reserve(additional);
    }

    /// Reserve capacity for systems
    #[inline]
    pub fn reserve_systems(&mut self, additional: usize) {
        self.systems.reserve(additional);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oats_system_creation() {
        let system = OatsSystem::new();
        assert_eq!(system.object_count(), 0);
        assert_eq!(system.action_count(), 0);
        assert_eq!(system.system_count(), 0);
    }

    #[test]
    fn test_oats_system_with_capacity() {
        let system = OatsSystem::with_capacity(100, 10, 5);
        assert_eq!(system.object_count(), 0);
        assert_eq!(system.action_count(), 0);
        assert_eq!(system.system_count(), 0);
    }

    #[test]
    fn test_oats_system_operations() {
        let mut system = OatsSystem::new();

        // Test object operations
        let obj = Object::new("test", "type");
        system.add_object(obj);
        assert_eq!(system.object_count(), 1);

        // Test bulk object operations
        let objects = vec![Object::new("obj1", "type"), Object::new("obj2", "type")];
        system.add_objects(objects);
        assert_eq!(system.object_count(), 3);

        // Test capacity reservation
        system.reserve_objects(100);
        assert!(system.objects.capacity() >= 103);
    }
}
