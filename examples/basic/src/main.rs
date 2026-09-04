use async_trait::async_trait;
use oats_framework::{
    Action, ActionContext, ActionResult, Object, Priority, System, SystemManager, Trait, TraitData,
};
use std::collections::HashMap;

// Custom heal action
struct HealAction;

#[async_trait]
impl Action for HealAction {
    fn name(&self) -> &str {
        "heal"
    }

    fn description(&self) -> &str {
        "Restores health"
    }

    async fn execute(
        &self,
        context: ActionContext,
    ) -> Result<ActionResult, oats_framework::OatsError> {
        let target = context.get_object("target").unwrap();
        let current_health = target
            .get_trait("health")
            .and_then(|t| t.data().as_number())
            .unwrap_or(0.0);

        let new_health = (current_health + 25.0).min(100.0);
        let health_trait = Trait::new("health", TraitData::Number(new_health));

        let mut result = ActionResult::success();
        result.add_trait_update(target.id(), health_trait);
        result.add_message(format!(
            "Incremented health from {current_health} to {new_health}"
        ));
        Ok(result)
    }
}

// Custom damage action
struct DamageAction;

#[async_trait]
impl Action for DamageAction {
    fn name(&self) -> &str {
        "damage"
    }

    fn description(&self) -> &str {
        "Inflicts damage"
    }

    async fn execute(
        &self,
        context: ActionContext,
    ) -> Result<ActionResult, oats_framework::OatsError> {
        let target = context.get_object("target").unwrap();
        let current_health = target
            .get_trait("health")
            .and_then(|t| t.data().as_number())
            .unwrap_or(0.0);

        let new_health = (current_health - 10.0).max(0.0);
        let health_trait = Trait::new("health", TraitData::Number(new_health));

        let mut result = ActionResult::success();
        result.add_trait_update(target.id(), health_trait);
        result.add_message(format!(
            "Decremented health from {current_health} to {new_health}"
        ));
        Ok(result)
    }
}

// Custom set position action
struct SetPositionAction;

#[async_trait]
impl Action for SetPositionAction {
    fn name(&self) -> &str {
        "set_position"
    }

    fn description(&self) -> &str {
        "Sets position"
    }

    async fn execute(
        &self,
        context: ActionContext,
    ) -> Result<ActionResult, oats_framework::OatsError> {
        let target = context
            .get_object("target")
            .ok_or_else(|| oats_framework::OatsError::action_failed("Target object not found"))?;

        let mut position_data = HashMap::new();
        position_data.insert("x".to_string(), serde_json::json!(10.0));
        position_data.insert("y".to_string(), serde_json::json!(20.0));

        let position_trait = Trait::new("position", TraitData::Object(position_data));

        let mut result = ActionResult::success();
        result.add_trait_update(target.id(), position_trait);
        result.add_message("Set position to Object({})");
        Ok(result)
    }
}

// Custom health system
struct HealthSystem {
    stats: oats_framework::systems::SystemStats,
}

impl HealthSystem {
    fn new() -> Self {
        Self {
            stats: oats_framework::systems::SystemStats::default(),
        }
    }
}

#[async_trait]
impl System for HealthSystem {
    fn name(&self) -> &str {
        "health_system"
    }

    fn description(&self) -> &str {
        "Manages health-related operations"
    }

    async fn process(
        &mut self,
        objects: Vec<Object>,
        _priority: Priority,
    ) -> Result<Vec<ActionResult>, oats_framework::OatsError> {
        let mut results = Vec::new();
        let start_time = std::time::Instant::now();

        for object in objects {
            if object.has_trait("health") {
                let heal_action = HealAction;
                let mut context = ActionContext::new();
                context.add_object("target", object);

                match heal_action.execute(context).await {
                    Ok(result) => {
                        results.push(result);
                        self.stats.actions_executed += 1;
                    }
                    Err(e) => {
                        self.stats.errors += 1;
                        let error_result =
                            ActionResult::failure(format!("Health action failed: {e}"));
                        results.push(error_result);
                    }
                }
            }
            self.stats.objects_processed += 1;
        }

        self.stats
            .update_processing_time(start_time.elapsed().as_millis() as u64);
        self.stats.last_processed = Some(chrono::Utc::now());

        Ok(results)
    }

    fn get_stats(&self) -> oats_framework::systems::SystemStats {
        self.stats.clone()
    }
}

// Custom position system
struct PositionSystem {
    stats: oats_framework::systems::SystemStats,
}

impl PositionSystem {
    fn new() -> Self {
        Self {
            stats: oats_framework::systems::SystemStats::default(),
        }
    }
}

#[async_trait]
impl System for PositionSystem {
    fn name(&self) -> &str {
        "position_system"
    }

    fn description(&self) -> &str {
        "Manages position-related operations"
    }

    async fn process(
        &mut self,
        objects: Vec<Object>,
        _priority: Priority,
    ) -> Result<Vec<ActionResult>, oats_framework::OatsError> {
        let mut results = Vec::new();
        let start_time = std::time::Instant::now();

        for object in objects {
            if object.has_trait("position") {
                let set_position_action = SetPositionAction;
                let context = ActionContext::new();

                match set_position_action.execute(context).await {
                    Ok(result) => {
                        results.push(result);
                        self.stats.actions_executed += 1;
                    }
                    Err(e) => {
                        self.stats.errors += 1;
                        let error_result =
                            ActionResult::failure(format!("Position action failed: {e}"));
                        results.push(error_result);
                    }
                }
            }
            self.stats.objects_processed += 1;
        }

        self.stats
            .update_processing_time(start_time.elapsed().as_millis() as u64);
        self.stats.last_processed = Some(chrono::Utc::now());

        Ok(results)
    }

    fn get_stats(&self) -> oats_framework::systems::SystemStats {
        self.stats.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 OATS Basic Example");
    println!("=====================\n");

    // Create some objects with traits
    println!("1. Creating objects with traits...");

    let mut player = Object::new("player_1", "character");
    let health_trait = Trait::new("health", TraitData::Number(100.0));
    let position_trait = Trait::new("position", TraitData::Object(HashMap::new()));
    player.add_trait(health_trait);
    player.add_trait(position_trait);

    let mut enemy = Object::new("enemy_1", "character");
    let enemy_health = Trait::new("health", TraitData::Number(50.0));
    enemy.add_trait(enemy_health);

    println!(
        "   Created player: {} (type: {})",
        player.name(),
        player.object_type()
    );
    println!(
        "   Created enemy: {} (type: {})",
        enemy.name(),
        enemy.object_type()
    );
    println!("   Player traits: {:?}", player.trait_names());
    println!("   Enemy traits: {:?}", enemy.trait_names());

    // Create actions
    println!("\n2. Creating actions...");

    let heal_action = HealAction;
    let damage_action = DamageAction;
    let set_position_action = SetPositionAction;

    println!("   Created heal action: {}", heal_action.name());
    println!("   Created damage action: {}", damage_action.name());
    println!(
        "   Created set position action: {}",
        set_position_action.name()
    );

    // Test actions directly
    println!("\n3. Testing actions directly...");

    let mut context = ActionContext::new();
    context.add_object("target", player.clone());

    let heal_result = heal_action.execute(context).await?;
    println!("   Heal action result: {}", heal_result.is_success());
    if let Some(message) = heal_result.messages.first() {
        println!("   Message: {message}");
    }

    // Create systems
    println!("\n4. Creating systems...");

    let health_system = HealthSystem::new();
    let position_system = PositionSystem::new();

    println!("   Created health system: {}", health_system.name());
    println!("   Created position system: {}", position_system.name());

    // Create system manager and register everything
    println!("\n5. Setting up system manager...");

    let mut manager = SystemManager::new();
    manager.add_system(Box::new(health_system));
    manager.add_system(Box::new(position_system));

    // Register objects
    manager.register_object(player).await;
    manager.register_object(enemy).await;

    println!("   Registered {} systems", manager.systems().len());
    println!("   Registered objects: player_1, enemy_1");

    // Process objects through systems
    println!("\n6. Processing objects through systems...");

    let results = manager.process_all(Priority::Normal).await?;
    println!("   Processed {} action results", results.len());

    for (i, result) in results.iter().enumerate() {
        println!(
            "   Result {}: {}",
            i + 1,
            if result.is_success() {
                "SUCCESS"
            } else {
                "FAILURE"
            }
        );
        for message in &result.messages {
            println!("     Message: {message}");
        }
    }

    // Get system statistics
    println!("\n7. System statistics:");

    let stats = manager.get_all_stats();
    for (system_name, stat) in stats {
        println!("   {system_name}:");
        println!("     Objects processed: {}", stat.objects_processed);
        println!("     Actions executed: {}", stat.actions_executed);
        println!("     Errors: {}", stat.errors);
        println!(
            "     Total processing time: {}ms",
            stat.total_processing_time_ms
        );
    }

    println!("\n✅ Basic OATS example completed successfully!");
    Ok(())
}
