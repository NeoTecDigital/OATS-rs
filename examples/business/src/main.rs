//! Demonstrates the full OATS loop: systems propose trait updates, the host
//! inspects them, and `SystemManager::apply` writes the accepted ones back.
//! Balances, stock and prices move because of that second step.

mod actions;
mod systems;

use oats_framework::{
    ActionResult, Decimal, OatsError, Object, Priority, SystemManager, Trait, TraitData,
};
use systems::{place_order, InventoryManagementSystem, OrderSettlementSystem, PricingSystem};

/// Order total for each simulated day, as (units, scale). The third exceeds the
/// remaining balance, so it is refused rather than applied.
const DAILY_ORDERS: [(i64, u32); 3] = [(9999, 2), (14950, 2), (27525, 2)];

/// Standing discount applied to electronics, in percent
const ELECTRONICS_DISCOUNT: i64 = 10;

fn customer() -> Object {
    Object::with_traits(
        "john_doe",
        "customer",
        vec![
            Trait::new("balance", TraitData::Decimal(Decimal::new(50000, 2))),
            Trait::new("loyalty_points", TraitData::Integer(150)),
        ],
    )
}

fn product(name: &str, price: Decimal, stock: i64, category: &str, supplier: &Object) -> Object {
    Object::with_traits(
        name,
        "product",
        vec![
            Trait::new("price", TraitData::Decimal(price)),
            Trait::new("stock", TraitData::Integer(stock)),
            Trait::new("category", TraitData::String(category.to_string())),
            Trait::new("supplier", TraitData::Ref(supplier.id())),
        ],
    )
}

/// Registers the cast and returns the customer's id.
async fn seed(operations: &SystemManager) -> String {
    let supplier = Object::new("acme_supply", "supplier");
    let customer = customer();
    let customer_id = customer.id().to_string();

    let laptop = product(
        "laptop_pro",
        Decimal::new(99999, 2),
        15,
        "electronics",
        &supplier,
    );
    let book = product("rust_book", Decimal::new(4999, 2), 45, "books", &supplier);

    operations.register_object(supplier).await;
    operations.register_object(customer).await;
    operations.register_object(laptop).await;
    operations.register_object(book).await;

    customer_id
}

fn describe_customer(object: &Object) -> String {
    let balance = object
        .get_trait("balance")
        .and_then(|value| value.data().as_decimal())
        .unwrap_or_default();
    let points = object
        .get_trait("loyalty_points")
        .and_then(|value| value.data().as_integer())
        .unwrap_or(0);
    let status = object
        .get_trait("order_status")
        .and_then(|value| value.data().as_string())
        .cloned()
        .unwrap_or_else(|| "none".to_string());

    format!("${balance}, {points} points, order: {status}")
}

fn describe_product(object: &Object) -> String {
    let price = object
        .get_trait("price")
        .and_then(|value| value.data().as_decimal())
        .unwrap_or_default();
    let stock = object
        .get_trait("stock")
        .and_then(|value| value.data().as_integer())
        .unwrap_or(0);

    format!("${price}, stock {stock}")
}

async fn report_state(operations: &SystemManager) {
    let mut objects = operations.get_all_objects().await;
    objects.sort_by(|left, right| left.name().cmp(right.name()));

    for object in objects {
        let line = match object.object_type() {
            "customer" => describe_customer(&object),
            "product" => describe_product(&object),
            _ => continue,
        };
        println!("     {}: {line}", object.name());
    }
}

fn report_messages(results: &[ActionResult]) {
    for result in results {
        let marker = if result.is_success() { ' ' } else { 'x' };
        for message in &result.messages {
            println!("     {marker} {message}");
        }
    }
}

/// One simulated day: the host places an order, the systems propose, the host
/// applies. Nothing changes in the registry except through `apply`.
async fn simulate_day(
    operations: &mut SystemManager,
    customer_id: &str,
    day: usize,
    total: Decimal,
) -> Result<(), OatsError> {
    println!("\n   --- Business Day {day} ---");

    let account = operations
        .get_object(customer_id)
        .await
        .ok_or_else(|| OatsError::object_not_found(customer_id))?;
    let order = place_order(account, format!("ORD-{day:03}"), total).await?;
    report_messages(std::slice::from_ref(&order));
    operations.apply(&[order]).await?;

    let proposals = operations.process_all(Priority::Normal).await?;
    report_messages(&proposals);

    let report = operations.apply(&proposals).await?;
    println!(
        "     -> apply: {} updates / {} objects / {} refused",
        report.traits_applied, report.objects_updated, report.results_skipped
    );
    report_state(operations).await;
    Ok(())
}

fn report_stats(operations: &SystemManager) {
    let mut stats: Vec<_> = operations.get_all_stats().into_iter().collect();
    stats.sort_by(|left, right| left.0.cmp(&right.0));

    for (name, stat) in stats {
        println!(
            "   {name}: {} objects, {} actions, {} errors",
            stat.objects_processed, stat.actions_executed, stat.errors
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("OATS Business Example");
    println!("=====================\n");

    let mut operations = SystemManager::new();
    operations.add_system(Box::new(OrderSettlementSystem::default()));
    operations.add_system(Box::new(InventoryManagementSystem::default()));
    operations.add_system(Box::new(PricingSystem::new(Decimal::from(
        ELECTRONICS_DISCOUNT,
    ))));

    let customer_id = seed(&operations).await;
    println!(
        "1. {} systems, {} entities",
        operations.system_count(),
        operations.object_count().await
    );

    println!("\n2. Opening state:");
    report_state(&operations).await;

    println!("\n3. Simulating business operations...");
    for (index, (units, scale)) in DAILY_ORDERS.iter().enumerate() {
        let total = Decimal::new(*units, *scale);
        simulate_day(&mut operations, &customer_id, index + 1, total).await?;
    }

    println!("\n4. Business analytics:");
    report_stats(&operations);

    println!("\nBusiness simulation completed.");
    Ok(())
}
