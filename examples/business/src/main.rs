use async_trait::async_trait;
use oats_framework::{
    Action, ActionContext, ActionResult, OatsError, Object, Priority, System, SystemManager, Trait,
    TraitData,
};
use std::collections::HashMap;

// Custom business actions
struct ProcessOrderAction {
    order_id: String,
    items: Vec<String>,
    total_amount: f64,
}

impl ProcessOrderAction {
    fn new(order_id: String, items: Vec<String>, total_amount: f64) -> Self {
        Self {
            order_id,
            items,
            total_amount,
        }
    }
}

#[async_trait]
impl Action for ProcessOrderAction {
    fn name(&self) -> &str {
        "process_order"
    }

    fn description(&self) -> &str {
        "Processes a customer order"
    }

    async fn execute(&self, context: ActionContext) -> Result<ActionResult, OatsError> {
        let customer = context
            .get_object("customer")
            .ok_or_else(|| OatsError::action_failed("Customer not found"))?;

        // Create order trait
        let mut order_data = HashMap::new();
        order_data.insert(
            "order_id".to_string(),
            serde_json::json!(self.order_id.clone()),
        );
        order_data.insert("items".to_string(), serde_json::json!(self.items.clone()));
        order_data.insert(
            "total_amount".to_string(),
            serde_json::json!(self.total_amount),
        );
        order_data.insert("status".to_string(), serde_json::json!("processing"));

        let order_trait = Trait::new("current_order", TraitData::Object(order_data));

        // Update customer balance
        let current_balance = customer
            .get_trait("balance")
            .and_then(|t| t.data().as_number())
            .unwrap_or(0.0);

        let new_balance = current_balance - self.total_amount;
        let balance_trait = Trait::new("balance", TraitData::Number(new_balance));

        let mut result = ActionResult::success();
        result.add_trait_update(order_trait);
        result.add_trait_update(balance_trait);
        result.add_message(format!(
            "Processed order {} for ${:.2}. New balance: ${:.2}",
            self.order_id, self.total_amount, new_balance
        ));

        Ok(result)
    }
}

struct UpdateInventoryAction {
    product_id: String,
    quantity_change: f64,
}

impl UpdateInventoryAction {
    fn new(product_id: String, quantity_change: f64) -> Self {
        Self {
            product_id,
            quantity_change,
        }
    }
}

#[async_trait]
impl Action for UpdateInventoryAction {
    fn name(&self) -> &str {
        "update_inventory"
    }

    fn description(&self) -> &str {
        "Updates product inventory levels"
    }

    async fn execute(&self, context: ActionContext) -> Result<ActionResult, OatsError> {
        let product = context
            .get_object("product")
            .ok_or_else(|| OatsError::action_failed("Product not found"))?;

        let current_stock = product
            .get_trait("stock")
            .and_then(|t| t.data().as_number())
            .unwrap_or(0.0);

        let new_stock = (current_stock + self.quantity_change).max(0.0);
        let stock_trait = Trait::new("stock", TraitData::Number(new_stock));

        let mut result = ActionResult::success();
        result.add_trait_update(stock_trait);
        result.add_message(format!(
            "Updated {} stock: {:.0} -> {:.0}",
            self.product_id, current_stock, new_stock
        ));

        if new_stock < 10.0 {
            result.add_message(format!("⚠️  Low stock alert for {}!", self.product_id));
        }

        Ok(result)
    }
}

struct ApplyDiscountAction {
    discount_percentage: f64,
}

impl ApplyDiscountAction {
    fn new(discount_percentage: f64) -> Self {
        Self {
            discount_percentage,
        }
    }
}

#[async_trait]
impl Action for ApplyDiscountAction {
    fn name(&self) -> &str {
        "apply_discount"
    }

    fn description(&self) -> &str {
        "Applies a discount to product pricing"
    }

    async fn execute(&self, context: ActionContext) -> Result<ActionResult, OatsError> {
        let product = context
            .get_object("product")
            .ok_or_else(|| OatsError::action_failed("Product not found"))?;

        let current_price = product
            .get_trait("price")
            .and_then(|t| t.data().as_number())
            .unwrap_or(0.0);

        let discount_multiplier = 1.0 - (self.discount_percentage / 100.0);
        let new_price = current_price * discount_multiplier;
        let price_trait = Trait::new("price", TraitData::Number(new_price));

        let mut result = ActionResult::success();
        result.add_trait_update(price_trait);
        result.add_message(format!(
            "Applied {}% discount. Price: ${:.2} -> ${:.2}",
            self.discount_percentage, current_price, new_price
        ));

        Ok(result)
    }
}

// Custom business systems
struct OrderProcessingSystem {
    name: String,
    description: String,
    stats: oats_framework::systems::SystemStats,
}

impl OrderProcessingSystem {
    fn new() -> Self {
        Self {
            name: "order_processing_system".to_string(),
            description: "Handles customer order processing".to_string(),
            stats: oats_framework::systems::SystemStats::default(),
        }
    }
}

#[async_trait]
impl System for OrderProcessingSystem {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn process(
        &mut self,
        objects: Vec<Object>,
        _priority: Priority,
    ) -> Result<Vec<ActionResult>, OatsError> {
        let mut results = Vec::new();
        let start_time = std::time::Instant::now();

        // Process customers with pending orders
        for customer in objects {
            if customer.has_trait("current_order") {
                let order_trait = customer.get_trait("current_order").unwrap();
                if let Some(order_data) = order_trait.data().as_object() {
                    if let Some(status) = order_data.get("status") {
                        if status.as_str() == Some("processing") {
                            // Simulate order processing
                            let order_action = ProcessOrderAction::new(
                                "ORD-001".to_string(),
                                vec!["item1".to_string(), "item2".to_string()],
                                99.99,
                            );

                            let mut context = ActionContext::new();
                            context.add_object("customer", customer);

                            match order_action.execute(context).await {
                                Ok(result) => {
                                    results.push(result);
                                    self.stats.actions_executed += 1;
                                }
                                Err(e) => {
                                    self.stats.errors += 1;
                                    let error_result = ActionResult::failure(format!(
                                        "Order processing failed: {}",
                                        e
                                    ));
                                    results.push(error_result);
                                }
                            }
                        }
                    }
                }
            }
            self.stats.objects_processed += 1;
        }

        self.stats.total_processing_time_ms += start_time.elapsed().as_millis() as u64;
        self.stats.last_processed = Some(chrono::Utc::now());

        Ok(results)
    }

    fn get_stats(&self) -> oats_framework::systems::SystemStats {
        self.stats.clone()
    }
}

struct InventoryManagementSystem {
    name: String,
    description: String,
    stats: oats_framework::systems::SystemStats,
}

impl InventoryManagementSystem {
    fn new() -> Self {
        Self {
            name: "inventory_management_system".to_string(),
            description: "Manages product inventory levels".to_string(),
            stats: oats_framework::systems::SystemStats::default(),
        }
    }
}

#[async_trait]
impl System for InventoryManagementSystem {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn process(
        &mut self,
        objects: Vec<Object>,
        _priority: Priority,
    ) -> Result<Vec<ActionResult>, OatsError> {
        let mut results = Vec::new();
        let start_time = std::time::Instant::now();

        // Process products with low stock
        for product in objects {
            if product.has_trait("stock") {
                let stock_trait = product.get_trait("stock").unwrap();
                if let Some(stock) = stock_trait.data().as_number() {
                    if stock < 20.0 {
                        // Restock action
                        let restock_action = UpdateInventoryAction::new(
                            product.name().to_string(),
                            50.0, // Restock amount
                        );

                        let mut context = ActionContext::new();
                        context.add_object("product", product);

                        match restock_action.execute(context).await {
                            Ok(result) => {
                                results.push(result);
                                self.stats.actions_executed += 1;
                            }
                            Err(e) => {
                                self.stats.errors += 1;
                                let error_result = ActionResult::failure(format!(
                                    "Inventory update failed: {}",
                                    e
                                ));
                                results.push(error_result);
                            }
                        }
                    }
                }
            }
            self.stats.objects_processed += 1;
        }

        self.stats.total_processing_time_ms += start_time.elapsed().as_millis() as u64;
        self.stats.last_processed = Some(chrono::Utc::now());

        Ok(results)
    }

    fn get_stats(&self) -> oats_framework::systems::SystemStats {
        self.stats.clone()
    }
}

struct PricingSystem {
    name: String,
    description: String,
    stats: oats_framework::systems::SystemStats,
}

impl PricingSystem {
    fn new() -> Self {
        Self {
            name: "pricing_system".to_string(),
            description: "Manages product pricing and discounts".to_string(),
            stats: oats_framework::systems::SystemStats::default(),
        }
    }
}

#[async_trait]
impl System for PricingSystem {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn process(
        &mut self,
        objects: Vec<Object>,
        _priority: Priority,
    ) -> Result<Vec<ActionResult>, OatsError> {
        let mut results = Vec::new();
        let start_time = std::time::Instant::now();

        // Apply seasonal discounts to products
        for product in objects {
            if product.has_trait("price") && product.has_trait("category") {
                if let Some(category_trait) = product.get_trait("category") {
                    if let Some(category) = category_trait.data().as_string() {
                        if category == "electronics" {
                            // Apply 10% discount to electronics
                            let discount_action = ApplyDiscountAction::new(10.0);

                            let mut context = ActionContext::new();
                            context.add_object("product", product);

                            match discount_action.execute(context).await {
                                Ok(result) => {
                                    results.push(result);
                                    self.stats.actions_executed += 1;
                                }
                                Err(e) => {
                                    self.stats.errors += 1;
                                    let error_result = ActionResult::failure(format!(
                                        "Pricing update failed: {}",
                                        e
                                    ));
                                    results.push(error_result);
                                }
                            }
                        }
                    }
                }
            }
            self.stats.objects_processed += 1;
        }

        self.stats.total_processing_time_ms += start_time.elapsed().as_millis() as u64;
        self.stats.last_processed = Some(chrono::Utc::now());

        Ok(results)
    }

    fn get_stats(&self) -> oats_framework::systems::SystemStats {
        self.stats.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏢 OATS Business Example");
    println!("========================\n");

    // Create business entities
    println!("1. Creating business entities...");

    // Customer
    let mut customer = Object::new("john_doe", "customer");
    let balance_trait = Trait::new("balance", TraitData::Number(500.0));
    let loyalty_trait = Trait::new("loyalty_points", TraitData::Number(150.0));
    customer.add_trait(balance_trait);
    customer.add_trait(loyalty_trait);

    // Products
    let mut laptop = Object::new("laptop_pro", "product");
    let laptop_price = Trait::new("price", TraitData::Number(999.99));
    let laptop_stock = Trait::new("stock", TraitData::Number(15.0));
    let laptop_category = Trait::new("category", TraitData::String("electronics".to_string()));
    laptop.add_trait(laptop_price);
    laptop.add_trait(laptop_stock);
    laptop.add_trait(laptop_category);

    let mut book = Object::new("rust_book", "product");
    let book_price = Trait::new("price", TraitData::Number(49.99));
    let book_stock = Trait::new("stock", TraitData::Number(45.0));
    let book_category = Trait::new("category", TraitData::String("books".to_string()));
    book.add_trait(book_price);
    book.add_trait(book_stock);
    book.add_trait(book_category);

    println!(
        "   Created customer: {} (balance: ${:.2})",
        customer.name(),
        500.0
    );
    println!(
        "   Created laptop: {} (price: ${:.2}, stock: {:.0})",
        laptop.name(),
        999.99,
        15.0
    );
    println!(
        "   Created book: {} (price: ${:.2}, stock: {:.0})",
        book.name(),
        49.99,
        45.0
    );

    // Create business systems
    println!("\n2. Creating business systems...");

    let order_system = OrderProcessingSystem::new();
    let inventory_system = InventoryManagementSystem::new();
    let pricing_system = PricingSystem::new();

    println!(
        "   Created order processing system: {}",
        order_system.name()
    );
    println!(
        "   Created inventory management system: {}",
        inventory_system.name()
    );
    println!("   Created pricing system: {}", pricing_system.name());

    // Set up business operations
    println!("\n3. Setting up business operations...");

    let mut business_ops = SystemManager::new();
    business_ops.add_system(Box::new(order_system));
    business_ops.add_system(Box::new(inventory_system));
    business_ops.add_system(Box::new(pricing_system));

    // Register entities
    business_ops.register_object(customer).await;
    business_ops.register_object(laptop).await;
    business_ops.register_object(book).await;

    println!("   Registered {} systems", business_ops.systems().len());
    println!("   Registered {} entities", 3);

    // Simulate business operations
    println!("\n4. Simulating business operations...");

    for day in 1..=3 {
        println!("\n   --- Business Day {} ---", day);

        let results = business_ops.process_all(Priority::Normal).await?;

        for result in results {
            if result.is_success() {
                for message in &result.messages {
                    println!("     {}", message);
                }
            } else {
                for message in &result.messages {
                    println!("     ❌ {}", message);
                }
            }
        }

        // Check entity status
        let all_objects = business_ops.get_all_objects().await;
        for obj in all_objects {
            match obj.object_type() {
                "customer" => {
                    if let Some(balance) =
                        obj.get_trait("balance").and_then(|t| t.data().as_number())
                    {
                        println!("     {} balance: ${:.2}", obj.name(), balance);
                    }
                }
                "product" => {
                    if let Some(stock) = obj.get_trait("stock").and_then(|t| t.data().as_number()) {
                        if let Some(price) =
                            obj.get_trait("price").and_then(|t| t.data().as_number())
                        {
                            println!("     {}: ${:.2} (stock: {:.0})", obj.name(), price, stock);
                        }
                    }
                }
                _ => {}
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    }

    // Business analytics
    println!("\n5. Business analytics:");

    let stats = business_ops.get_all_stats();
    for (system_name, stat) in stats {
        println!("   {}:", system_name);
        println!("     Objects processed: {}", stat.objects_processed);
        println!("     Actions executed: {}", stat.actions_executed);
        println!("     Errors: {}", stat.errors);
        println!(
            "     Total processing time: {}ms",
            stat.total_processing_time_ms
        );
    }

    println!("\n💼 Business simulation completed!");
    Ok(())
}
