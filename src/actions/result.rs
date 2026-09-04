//! What an action returns: proposed trait updates, each naming the object it
//! applies to, plus messages and free-form data.

use crate::objects::ObjectId;
use crate::Trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A proposed change to one trait on one object.
///
/// Actions never mutate. They return `TraitUpdate`s naming the object each new
/// trait value belongs to, and the host decides whether to apply them - see
/// [`crate::SystemManager::apply`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitUpdate {
    /// Object this update is proposed against
    pub target: ObjectId,
    /// Trait value proposed for that object
    pub trait_value: Trait,
}

impl TraitUpdate {
    /// Propose `trait_value` for the object identified by `target`
    #[inline]
    pub fn new(target: ObjectId, trait_value: Trait) -> Self {
        Self {
            target,
            trait_value,
        }
    }

    /// Get the object this update is proposed against
    #[inline]
    pub fn target(&self) -> ObjectId {
        self.target
    }

    /// Get the proposed trait value
    #[inline]
    pub fn trait_value(&self) -> &Trait {
        &self.trait_value
    }

    /// Get the name of the trait being updated
    #[inline]
    pub fn trait_name(&self) -> &str {
        self.trait_value.name()
    }

    /// Consume the update and yield the proposed trait value
    #[inline]
    pub fn into_trait(self) -> Trait {
        self.trait_value
    }
}

/// Result of an action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// Whether the action was successful
    pub success: bool,
    /// Proposed trait updates, each naming the object it applies to
    pub trait_updates: Vec<TraitUpdate>,
    /// Messages or logs from the action
    pub messages: Vec<String>,
    /// Additional data returned by the action
    pub data: HashMap<String, serde_json::Value>,
}

impl ActionResult {
    /// Create a successful action result
    #[inline]
    pub fn success() -> Self {
        Self {
            success: true,
            trait_updates: Vec::new(),
            messages: Vec::new(),
            data: HashMap::new(),
        }
    }

    /// Create a failed action result
    #[inline]
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            trait_updates: Vec::new(),
            messages: vec![message.into()],
            data: HashMap::new(),
        }
    }

    /// Create a successful action result with pre-allocated capacity
    pub fn success_with_capacity(
        trait_capacity: usize,
        message_capacity: usize,
        data_capacity: usize,
    ) -> Self {
        Self {
            success: true,
            trait_updates: Vec::with_capacity(trait_capacity),
            messages: Vec::with_capacity(message_capacity),
            data: HashMap::with_capacity(data_capacity),
        }
    }

    /// Propose a trait value for the given object
    #[inline]
    pub fn add_trait_update(&mut self, target: ObjectId, trait_obj: Trait) {
        self.trait_updates.push(TraitUpdate::new(target, trait_obj));
    }

    /// Add multiple trait updates efficiently
    #[inline]
    pub fn add_trait_updates(&mut self, trait_updates: impl IntoIterator<Item = TraitUpdate>) {
        self.trait_updates.extend(trait_updates);
    }

    /// Iterate the updates proposed against a single object
    #[inline]
    pub fn updates_for(&self, target: ObjectId) -> impl Iterator<Item = &TraitUpdate> {
        self.trait_updates
            .iter()
            .filter(move |update| update.target == target)
    }

    /// Add a message to the result
    #[inline]
    pub fn add_message(&mut self, message: impl Into<String>) {
        self.messages.push(message.into());
    }

    /// Add multiple messages efficiently
    #[inline]
    pub fn add_messages(&mut self, messages: impl IntoIterator<Item = String>) {
        self.messages.extend(messages);
    }

    /// Add data to the result
    #[inline]
    pub fn add_data(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.data.insert(key.into(), value);
    }

    /// Reserve capacity for expected updates
    #[inline]
    pub fn reserve_capacity(&mut self, trait_updates: usize, messages: usize) {
        self.trait_updates.reserve(trait_updates);
        self.messages.reserve(messages);
    }

    /// Check if the action was successful
    #[inline]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Check if the action failed
    #[inline]
    pub fn is_failure(&self) -> bool {
        !self.success
    }

    /// Get trait update count
    #[inline]
    pub fn trait_update_count(&self) -> usize {
        self.trait_updates.len()
    }

    /// Get message count
    #[inline]
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Get data count
    #[inline]
    pub fn data_count(&self) -> usize {
        self.data.len()
    }

    /// Clear all trait updates
    #[inline]
    pub fn clear_trait_updates(&mut self) {
        self.trait_updates.clear();
    }

    /// Clear all messages
    #[inline]
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// Clear all data
    #[inline]
    pub fn clear_data(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::TraitData;
    use crate::Object;

    #[test]
    fn test_action_result() {
        let mut result = ActionResult::success();
        result.add_message("Test message");
        result.add_data("key", serde_json::json!("value"));

        assert!(result.is_success());
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.data.len(), 1);
    }

    #[test]
    fn test_trait_update_names_its_target() {
        let customer = Object::new("john_doe", "customer");
        let product = Object::new("laptop", "product");

        let mut result = ActionResult::success();
        result.add_trait_update(
            customer.id(),
            Trait::new("balance", TraitData::Number(400.0)),
        );
        result.add_trait_update(product.id(), Trait::new("stock", TraitData::Number(65.0)));

        assert_eq!(result.trait_update_count(), 2);

        let for_customer: Vec<_> = result.updates_for(customer.id()).collect();
        assert_eq!(for_customer.len(), 1);
        assert_eq!(for_customer[0].trait_name(), "balance");
        assert_eq!(for_customer[0].target(), customer.id());

        let for_product: Vec<_> = result.updates_for(product.id()).collect();
        assert_eq!(for_product.len(), 1);
        assert_eq!(for_product[0].trait_name(), "stock");

        assert_eq!(result.updates_for(Object::new("x", "y").id()).count(), 0);
    }

    #[test]
    fn test_trait_update_survives_serialization() {
        let target = Object::new("john_doe", "customer").id();
        let mut result = ActionResult::success();
        result.add_trait_update(target, Trait::new("balance", TraitData::Number(400.0)));

        let json = serde_json::to_string(&result).expect("serialize");
        let restored: ActionResult = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.trait_updates[0].target(), target);
        assert_eq!(restored.trait_updates[0].trait_name(), "balance");
    }
}
