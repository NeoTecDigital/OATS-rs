//! Business actions. Each one reads traits and returns proposals; none of them
//! mutate. The host applies the proposals in `main`.

use async_trait::async_trait;
use oats_framework::{Action, ActionContext, ActionResult, Decimal, OatsError, Trait, TraitData};
use rust_decimal::prelude::ToPrimitive;

/// An order that has been placed but not yet paid for
pub const PENDING: &str = "pending";
/// An order the customer has been charged for
pub const SETTLED: &str = "settled";

fn decimal_trait(object: &oats_framework::Object, name: &str) -> Result<Decimal, OatsError> {
    object
        .get_trait(name)
        .and_then(|value| value.data().as_decimal())
        .ok_or_else(|| OatsError::validation_error(format!("'{name}' is not an exact decimal")))
}

/// Records a new pending order against a customer.
pub struct PlaceOrderAction {
    order_id: String,
    total: Decimal,
}

impl PlaceOrderAction {
    pub fn new(order_id: impl Into<String>, total: Decimal) -> Self {
        Self {
            order_id: order_id.into(),
            total,
        }
    }
}

#[async_trait]
impl Action for PlaceOrderAction {
    fn name(&self) -> &str {
        "place_order"
    }

    fn description(&self) -> &str {
        "Records a new pending order against a customer"
    }

    fn required_traits(&self) -> Vec<String> {
        vec!["balance".to_string()]
    }

    async fn execute(&self, context: ActionContext) -> Result<ActionResult, OatsError> {
        let customer = context
            .get_object("customer")
            .ok_or_else(|| OatsError::action_failed("customer not in context"))?;
        let target = customer.id();

        let mut result = ActionResult::success();
        result.add_trait_update(
            target,
            Trait::new("order_id", TraitData::String(self.order_id.clone())),
        );
        result.add_trait_update(
            target,
            Trait::new("order_total", TraitData::Decimal(self.total)),
        );
        result.add_trait_update(
            target,
            Trait::new("order_status", TraitData::String(PENDING.to_string())),
        );
        result.add_trait_update(
            target,
            Trait::new("order_placed_at", TraitData::Timestamp(chrono::Utc::now())),
        );
        result.add_message(format!(
            "Placed order {} for ${} against {}",
            self.order_id,
            self.total,
            customer.name()
        ));
        Ok(result)
    }
}

/// Charges a customer for their pending order and awards loyalty points.
pub struct SettleOrderAction;

#[async_trait]
impl Action for SettleOrderAction {
    fn name(&self) -> &str {
        "settle_order"
    }

    fn description(&self) -> &str {
        "Charges a customer for their pending order"
    }

    fn required_traits(&self) -> Vec<String> {
        vec![
            "balance".to_string(),
            "loyalty_points".to_string(),
            "order_id".to_string(),
            "order_status".to_string(),
            "order_total".to_string(),
        ]
    }

    async fn execute(&self, context: ActionContext) -> Result<ActionResult, OatsError> {
        let customer = context
            .get_object("customer")
            .ok_or_else(|| OatsError::action_failed("customer not in context"))?;

        let balance = decimal_trait(customer, "balance")?;
        let total = decimal_trait(customer, "order_total")?;
        let order_id = customer
            .get_trait("order_id")
            .and_then(|value| value.data().as_string())
            .cloned()
            .ok_or_else(|| OatsError::validation_error("'order_id' is not a string"))?;

        if balance < total {
            return Ok(ActionResult::failure(format!(
                "Refused order {order_id}: balance ${balance} does not cover ${total}"
            )));
        }

        let points = customer
            .get_trait("loyalty_points")
            .and_then(|value| value.data().as_integer())
            .unwrap_or(0);
        let earned = total.trunc().to_i64().unwrap_or(0);
        let remaining = balance - total;
        let target = customer.id();

        let mut result = ActionResult::success();
        result.add_trait_update(target, Trait::new("balance", TraitData::Decimal(remaining)));
        result.add_trait_update(
            target,
            Trait::new("loyalty_points", TraitData::Integer(points + earned)),
        );
        result.add_trait_update(
            target,
            Trait::new("order_status", TraitData::String(SETTLED.to_string())),
        );
        result.add_message(format!(
            "Settled order {order_id} for ${total}. Balance ${balance} -> ${remaining}, +{earned} points"
        ));
        Ok(result)
    }
}

/// Raises a product's stock level.
pub struct RestockAction {
    quantity: i64,
    supplier: String,
}

impl RestockAction {
    pub fn new(quantity: i64, supplier: impl Into<String>) -> Self {
        Self {
            quantity,
            supplier: supplier.into(),
        }
    }
}

#[async_trait]
impl Action for RestockAction {
    fn name(&self) -> &str {
        "restock"
    }

    fn description(&self) -> &str {
        "Raises a product's stock level"
    }

    fn required_traits(&self) -> Vec<String> {
        vec!["stock".to_string()]
    }

    async fn execute(&self, context: ActionContext) -> Result<ActionResult, OatsError> {
        let product = context
            .get_object("product")
            .ok_or_else(|| OatsError::action_failed("product not in context"))?;

        let stock = product
            .get_trait("stock")
            .and_then(|value| value.data().as_integer())
            .ok_or_else(|| OatsError::validation_error("'stock' is not a whole number"))?;
        let restocked = stock.saturating_add(self.quantity);

        let mut result = ActionResult::success();
        result.add_trait_update(
            product.id(),
            Trait::new("stock", TraitData::Integer(restocked)),
        );
        result.add_message(format!(
            "Restocked {} from {}: {stock} -> {restocked}",
            product.name(),
            self.supplier
        ));
        Ok(result)
    }
}

/// Discounts a product's price by a percentage.
pub struct ApplyDiscountAction {
    percentage: Decimal,
}

impl ApplyDiscountAction {
    pub fn new(percentage: Decimal) -> Self {
        Self { percentage }
    }
}

#[async_trait]
impl Action for ApplyDiscountAction {
    fn name(&self) -> &str {
        "apply_discount"
    }

    fn description(&self) -> &str {
        "Discounts a product's price"
    }

    fn required_traits(&self) -> Vec<String> {
        vec!["price".to_string()]
    }

    async fn execute(&self, context: ActionContext) -> Result<ActionResult, OatsError> {
        let product = context
            .get_object("product")
            .ok_or_else(|| OatsError::action_failed("product not in context"))?;

        let price = decimal_trait(product, "price")?;
        let multiplier = Decimal::ONE - (self.percentage / Decimal::from(100));
        let discounted = (price * multiplier).round_dp(2);

        let mut result = ActionResult::success();
        result.add_trait_update(
            product.id(),
            Trait::new("price", TraitData::Decimal(discounted)),
        );
        result.add_message(format!(
            "Discounted {} by {}%: ${price} -> ${discounted}",
            product.name(),
            self.percentage
        ));
        Ok(result)
    }
}
