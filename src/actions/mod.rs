//! Actions: stateless logic that reads traits and returns proposed updates.
//!
//! An action never mutates. It returns [`TraitUpdate`]s naming the object each
//! new trait value belongs to, and the host applies them in a separate step.

mod context;
mod result;

pub use context::ActionContext;
pub use result::{ActionResult, TraitUpdate};

use crate::{OatsError, Result};
use async_trait::async_trait;

/// Action identifier
pub type ActionId = uuid::Uuid;

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
    use crate::Object;
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
}
