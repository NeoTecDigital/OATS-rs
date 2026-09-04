//! Proves the OATS loop closes: an action proposes, the registry is untouched
//! until the host applies, and the trait value differs afterwards.

use async_trait::async_trait;
use oats_framework::{
    Action, ActionContext, ActionResult, OatsError, Object, SystemManager, Trait, TraitData,
};

/// Debits a numeric trait by a fixed amount and proposes the new value.
struct DebitAction {
    trait_name: String,
    amount: f64,
}

#[async_trait]
impl Action for DebitAction {
    fn name(&self) -> &str {
        "debit"
    }

    fn description(&self) -> &str {
        "Debits a numeric trait"
    }

    fn required_traits(&self) -> Vec<String> {
        vec![self.trait_name.clone()]
    }

    async fn execute(&self, context: ActionContext) -> Result<ActionResult, OatsError> {
        let account = context
            .get_object("account")
            .ok_or_else(|| OatsError::action_failed("account not in context"))?;

        let current = account
            .get_trait(&self.trait_name)
            .and_then(|t| t.data().as_number())
            .ok_or_else(|| OatsError::trait_not_found(&self.trait_name))?;

        let mut result = ActionResult::success();
        result.add_trait_update(
            account.id(),
            Trait::new(&self.trait_name, TraitData::Number(current - self.amount)),
        );
        Ok(result)
    }
}

async fn registered_balance(manager: &SystemManager, id: &str) -> f64 {
    manager
        .get_object(id)
        .await
        .expect("object registered")
        .get_trait("balance")
        .and_then(|t| t.data().as_number())
        .expect("balance is numeric")
}

#[tokio::test]
async fn apply_writes_proposed_traits_back_to_the_registry() {
    let account = Object::with_traits(
        "john_doe",
        "customer",
        vec![Trait::new("balance", TraitData::Number(500.0))],
    );
    let account_id = account.id().to_string();

    let manager = SystemManager::new();
    manager.register_object(account.clone()).await;

    let mut context = ActionContext::new();
    context.add_object("account", account);

    let action = DebitAction {
        trait_name: "balance".to_string(),
        amount: 99.5,
    };
    let result = action.run(context).await.expect("action succeeds");

    // Proposing must not mutate.
    assert_eq!(registered_balance(&manager, &account_id).await, 500.0);

    let report = manager.apply(&[result]).await.expect("apply succeeds");
    assert_eq!(report.traits_applied, 1);
    assert_eq!(report.objects_updated, 1);
    assert_eq!(report.results_skipped, 0);

    // Applying must mutate.
    assert_eq!(registered_balance(&manager, &account_id).await, 400.5);

    let stored = manager.get_object(&account_id).await.expect("registered");
    assert_eq!(
        stored.get_trait("balance").unwrap().version(),
        2,
        "writeback supersedes the previous trait version"
    );
}

#[tokio::test]
async fn repeated_apply_passes_keep_moving_the_value() {
    let account = Object::with_traits(
        "john_doe",
        "customer",
        vec![Trait::new("balance", TraitData::Number(500.0))],
    );
    let account_id = account.id().to_string();

    let manager = SystemManager::new();
    manager.register_object(account).await;

    for _ in 0..3 {
        let current = manager.get_object(&account_id).await.expect("registered");
        let mut context = ActionContext::new();
        context.add_object("account", current);

        let action = DebitAction {
            trait_name: "balance".to_string(),
            amount: 100.0,
        };
        let result = action.run(context).await.expect("action succeeds");
        manager.apply(&[result]).await.expect("apply succeeds");
    }

    assert_eq!(registered_balance(&manager, &account_id).await, 200.0);
}

#[tokio::test]
async fn apply_refuses_updates_for_unregistered_objects() {
    let known = Object::with_traits(
        "known",
        "customer",
        vec![Trait::new("balance", TraitData::Number(500.0))],
    );
    let known_id = known.id().to_string();
    let stranger = Object::new("stranger", "customer");

    let manager = SystemManager::new();
    manager.register_object(known.clone()).await;

    let mut result = ActionResult::success();
    result.add_trait_update(known.id(), Trait::new("balance", TraitData::Number(1.0)));
    result.add_trait_update(stranger.id(), Trait::new("balance", TraitData::Number(2.0)));

    let error = manager
        .apply(&[result])
        .await
        .expect_err("unknown target is refused");
    assert!(matches!(error, OatsError::ObjectNotFound { .. }));

    // All-or-nothing: the valid half of the batch was not written either.
    assert_eq!(registered_balance(&manager, &known_id).await, 500.0);
}

#[tokio::test]
async fn apply_skips_updates_carried_by_failed_results() {
    let account = Object::with_traits(
        "john_doe",
        "customer",
        vec![Trait::new("balance", TraitData::Number(500.0))],
    );
    let account_id = account.id().to_string();

    let manager = SystemManager::new();
    manager.register_object(account.clone()).await;

    let mut failed = ActionResult::failure("insufficient funds");
    failed.add_trait_update(account.id(), Trait::new("balance", TraitData::Number(-1.0)));

    let report = manager.apply(&[failed]).await.expect("apply succeeds");
    assert_eq!(report.traits_applied, 0);
    assert_eq!(report.results_skipped, 1);
    assert!(report.is_empty());
    assert_eq!(registered_balance(&manager, &account_id).await, 500.0);
}

#[tokio::test]
async fn apply_updates_touches_every_named_object() {
    let customer = Object::with_traits(
        "john_doe",
        "customer",
        vec![Trait::new("balance", TraitData::Number(500.0))],
    );
    let product = Object::with_traits(
        "laptop",
        "product",
        vec![Trait::new("stock", TraitData::Number(15.0))],
    );
    let customer_id = customer.id().to_string();
    let product_id = product.id().to_string();

    let manager = SystemManager::new();
    manager.register_object(customer.clone()).await;
    manager.register_object(product.clone()).await;

    let report = manager
        .apply_updates(vec![
            oats_framework::TraitUpdate::new(
                customer.id(),
                Trait::new("balance", TraitData::Number(400.01)),
            ),
            oats_framework::TraitUpdate::new(
                product.id(),
                Trait::new("stock", TraitData::Number(65.0)),
            ),
        ])
        .await
        .expect("apply succeeds");

    assert_eq!(report.traits_applied, 2);
    assert_eq!(report.objects_updated, 2);
    assert_eq!(registered_balance(&manager, &customer_id).await, 400.01);
    assert_eq!(
        manager
            .get_object(&product_id)
            .await
            .unwrap()
            .get_trait("stock")
            .and_then(|t| t.data().as_number()),
        Some(65.0)
    );
}
