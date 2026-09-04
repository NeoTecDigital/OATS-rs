use crate::objects::ObjectId;
use crate::{OatsError, Object, Result, Trait};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Action identifier
pub type ActionId = uuid::Uuid;

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

/// An action represents stateless logic that reads traits and returns updates
#[async_trait]
pub trait Action: Send + Sync {
    /// Get the name of this action
    fn name(&self) -> &str;

    /// Get the description of this action
    fn description(&self) -> &str;

    /// Execute the action with the given context.
    ///
    /// This is the implementor's hook and checks no preconditions of its own.
    /// Callers should use [`Action::run`].
    async fn execute(&self, context: ActionContext) -> Result<ActionResult>;

    /// Check preconditions, then execute.
    ///
    /// This is the entry point callers should use: it refuses a context that
    /// does not satisfy [`Action::required_traits`] rather than letting the
    /// action read a missing trait and substitute a default.
    async fn run(&self, context: ActionContext) -> Result<ActionResult> {
        self.validate_context(&context)?;
        self.execute(context).await
    }

    /// Check that a context satisfies this action's declared preconditions.
    ///
    /// The default requires every object in the context to carry every trait
    /// named by [`Action::required_traits`], and refuses an empty context when
    /// any trait is required. Actions whose context holds objects in different
    /// roles with different requirements should override this.
    fn validate_context(&self, context: &ActionContext) -> Result<()> {
        let required = self.required_traits();
        if required.is_empty() {
            return Ok(());
        }

        if context.objects.is_empty() {
            return Err(OatsError::validation_error(format!(
                "Action '{}' requires traits [{}] but its context holds no objects",
                self.name(),
                required.join(", ")
            )));
        }

        let required: Vec<&str> = required.iter().map(String::as_str).collect();
        let mut failures: Vec<String> = context
            .objects
            .iter()
            .filter_map(|(role, object)| {
                object
                    .validate_required_traits(&required)
                    .err()
                    .map(|error| format!("'{role}': {error}"))
            })
            .collect();

        if failures.is_empty() {
            return Ok(());
        }

        failures.sort();
        Err(OatsError::validation_error(format!(
            "Action '{}' preconditions not met - {}",
            self.name(),
            failures.join("; ")
        )))
    }

    /// Get the required trait names for this action
    fn required_traits(&self) -> Vec<String> {
        Vec::new()
    }

    /// Get the optional trait names for this action
    fn optional_traits(&self) -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{Trait, TraitData};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Counts how often `execute` ran, so a refused precondition is provable.
    struct CountingAction {
        required: Vec<String>,
        executions: AtomicUsize,
    }

    impl CountingAction {
        fn new(required: &[&str]) -> Self {
            Self {
                required: required.iter().map(|name| name.to_string()).collect(),
                executions: AtomicUsize::new(0),
            }
        }

        fn executions(&self) -> usize {
            self.executions.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl Action for CountingAction {
        fn name(&self) -> &str {
            "counting"
        }

        fn description(&self) -> &str {
            "Counts executions"
        }

        fn required_traits(&self) -> Vec<String> {
            self.required.clone()
        }

        async fn execute(&self, _context: ActionContext) -> Result<ActionResult> {
            self.executions.fetch_add(1, Ordering::SeqCst);
            Ok(ActionResult::success())
        }
    }

    fn context_with(object: Object) -> ActionContext {
        let mut context = ActionContext::new();
        context.add_object("target", object);
        context
    }

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

    #[test]
    fn test_action_result() {
        let mut result = ActionResult::success();
        result.add_message("Test message");
        result.add_data("key", serde_json::json!("value"));

        assert!(result.is_success());
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.data.len(), 1);
    }

    #[tokio::test]
    async fn run_refuses_a_context_missing_a_required_trait() {
        let action = CountingAction::new(&["balance"]);
        let context = context_with(Object::new("john_doe", "customer"));

        let error = action
            .run(context)
            .await
            .expect_err("missing precondition is refused");

        assert!(matches!(error, OatsError::ValidationError { .. }));
        assert!(error.to_string().contains("balance"), "{error}");
        assert_eq!(action.executions(), 0, "execute must not have been reached");
    }

    #[tokio::test]
    async fn run_executes_when_preconditions_are_met() {
        let action = CountingAction::new(&["balance"]);
        let mut customer = Object::new("john_doe", "customer");
        customer.add_trait(Trait::new("balance", TraitData::Integer(500)));

        let result = action
            .run(context_with(customer))
            .await
            .expect("precondition satisfied");

        assert!(result.is_success());
        assert_eq!(action.executions(), 1);
    }

    #[tokio::test]
    async fn run_refuses_an_empty_context_when_traits_are_required() {
        let action = CountingAction::new(&["balance"]);

        let error = action
            .run(ActionContext::new())
            .await
            .expect_err("empty context cannot satisfy a requirement");

        assert!(matches!(error, OatsError::ValidationError { .. }));
        assert_eq!(action.executions(), 0);
    }

    #[tokio::test]
    async fn run_allows_an_empty_context_when_nothing_is_required() {
        let action = CountingAction::new(&[]);

        action
            .run(ActionContext::new())
            .await
            .expect("no preconditions to meet");

        assert_eq!(action.executions(), 1);
    }

    #[tokio::test]
    async fn run_reports_every_object_that_fails() {
        let action = CountingAction::new(&["balance", "tier"]);
        let mut context = ActionContext::new();
        context.add_object("buyer", Object::new("john_doe", "customer"));
        context.add_object("seller", Object::new("acme", "customer"));

        let error = action.run(context).await.expect_err("both objects fail");
        let message = error.to_string();

        assert!(message.contains("'buyer'"), "{message}");
        assert!(message.contains("'seller'"), "{message}");
        assert_eq!(action.executions(), 0);
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
