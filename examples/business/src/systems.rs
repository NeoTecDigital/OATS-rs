//! Business systems. Systems decide which actions to run and collect the
//! proposals; they never write to the registry.

use crate::actions::{
    ApplyDiscountAction, PlaceOrderAction, RestockAction, SettleOrderAction, PENDING,
};
use async_trait::async_trait;
use oats_framework::systems::SystemStats;
use oats_framework::{
    Action, ActionContext, ActionResult, Decimal, OatsError, Object, Priority, System,
};
use std::time::Instant;

/// Stock at or below this level triggers a restock proposal
const RESTOCK_THRESHOLD: i64 = 20;
/// Units added by a single restock
const RESTOCK_QUANTITY: i64 = 50;

/// Runs an action and records the outcome against a system's statistics.
async fn dispatch(
    action: &dyn Action,
    context: ActionContext,
    stats: &mut SystemStats,
    results: &mut Vec<ActionResult>,
) {
    match action.run(context).await {
        Ok(result) => {
            stats.actions_executed += 1;
            results.push(result);
        }
        Err(error) => {
            stats.errors += 1;
            results.push(ActionResult::failure(format!(
                "{} failed: {error}",
                action.name()
            )));
        }
    }
}

fn finish(stats: &mut SystemStats, started: Instant) {
    stats.total_processing_time_ms += started.elapsed().as_millis() as u64;
    stats.last_processed = Some(chrono::Utc::now());
}

fn has_status(object: &Object, status: &str) -> bool {
    object
        .get_trait("order_status")
        .and_then(|value| value.data().as_string())
        .map(|value| value == status)
        .unwrap_or(false)
}

/// Resolves a product's `supplier` reference against the objects in play.
fn supplier_name(objects: &[Object], product: &Object) -> String {
    product
        .get_trait("supplier")
        .and_then(|value| value.data().as_ref_id())
        .and_then(|id| objects.iter().find(|candidate| candidate.id() == id))
        .map(|supplier| supplier.name().to_string())
        .unwrap_or_else(|| "an unlisted supplier".to_string())
}

/// Charges customers whose order is still pending.
#[derive(Default)]
pub struct OrderSettlementSystem {
    stats: SystemStats,
}

#[async_trait]
impl System for OrderSettlementSystem {
    fn name(&self) -> &str {
        "order_settlement_system"
    }

    fn description(&self) -> &str {
        "Charges customers for pending orders"
    }

    fn priority(&self) -> Priority {
        Priority::High
    }

    async fn process(
        &mut self,
        objects: Vec<Object>,
        _priority: Priority,
    ) -> Result<Vec<ActionResult>, OatsError> {
        let started = Instant::now();
        let mut results = Vec::new();

        for object in objects {
            if has_status(&object, PENDING) {
                let mut context = ActionContext::new();
                context.add_object("customer", object);
                dispatch(&SettleOrderAction, context, &mut self.stats, &mut results).await;
            }
            self.stats.objects_processed += 1;
        }

        finish(&mut self.stats, started);
        Ok(results)
    }

    fn get_stats(&self) -> SystemStats {
        self.stats.clone()
    }
}

/// Restocks products that have fallen below the threshold.
#[derive(Default)]
pub struct InventoryManagementSystem {
    stats: SystemStats,
}

#[async_trait]
impl System for InventoryManagementSystem {
    fn name(&self) -> &str {
        "inventory_management_system"
    }

    fn description(&self) -> &str {
        "Keeps product stock above the restock threshold"
    }

    async fn process(
        &mut self,
        objects: Vec<Object>,
        _priority: Priority,
    ) -> Result<Vec<ActionResult>, OatsError> {
        let started = Instant::now();
        let mut results = Vec::new();

        for product in &objects {
            let low = product
                .get_trait("stock")
                .and_then(|value| value.data().as_integer())
                .is_some_and(|stock| stock < RESTOCK_THRESHOLD);

            if low {
                let action = RestockAction::new(RESTOCK_QUANTITY, supplier_name(&objects, product));
                let mut context = ActionContext::new();
                context.add_object("product", product.clone());
                dispatch(&action, context, &mut self.stats, &mut results).await;
            }
            self.stats.objects_processed += 1;
        }

        finish(&mut self.stats, started);
        Ok(results)
    }

    fn get_stats(&self) -> SystemStats {
        self.stats.clone()
    }
}

/// Applies the standing electronics discount.
pub struct PricingSystem {
    stats: SystemStats,
    discount: Decimal,
}

impl PricingSystem {
    pub fn new(discount: Decimal) -> Self {
        Self {
            stats: SystemStats::default(),
            discount,
        }
    }
}

#[async_trait]
impl System for PricingSystem {
    fn name(&self) -> &str {
        "pricing_system"
    }

    fn description(&self) -> &str {
        "Applies the standing electronics discount"
    }

    fn priority(&self) -> Priority {
        Priority::Low
    }

    async fn process(
        &mut self,
        objects: Vec<Object>,
        _priority: Priority,
    ) -> Result<Vec<ActionResult>, OatsError> {
        let started = Instant::now();
        let mut results = Vec::new();

        for product in objects {
            let electronics = product
                .get_trait("category")
                .and_then(|value| value.data().as_string())
                .is_some_and(|category| category == "electronics");

            if electronics {
                let action = ApplyDiscountAction::new(self.discount);
                let mut context = ActionContext::new();
                context.add_object("product", product);
                dispatch(&action, context, &mut self.stats, &mut results).await;
            }
            self.stats.objects_processed += 1;
        }

        finish(&mut self.stats, started);
        Ok(results)
    }

    fn get_stats(&self) -> SystemStats {
        self.stats.clone()
    }
}

/// Places the day's order for a customer. Kept out of the system loop because
/// placing an order is a host decision, not something a system discovers.
pub async fn place_order(
    customer: Object,
    order_id: impl Into<String>,
    total: Decimal,
) -> Result<ActionResult, OatsError> {
    let mut context = ActionContext::new();
    context.add_object("customer", customer);
    PlaceOrderAction::new(order_id, total).run(context).await
}
