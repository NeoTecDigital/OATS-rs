use oats_framework::{
    Action, ActionContext, ActionResult, Object, Priority, System, SystemManager, Trait, TraitData,
};
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::collections::HashMap;

// Custom game actions
struct CombatAction {
    damage: f64,
}

impl CombatAction {
    fn new(damage: f64) -> Self {
        Self { damage }
    }
}

#[async_trait::async_trait]
impl Action for CombatAction {
    fn name(&self) -> &str {
        "combat"
    }

    fn description(&self) -> &str {
        "Deals damage to target"
    }

    async fn execute(
        &self,
        context: ActionContext,
    ) -> Result<ActionResult, oats_framework::OatsError> {
        let target = context
            .get_object("target")
            .ok_or_else(|| oats_framework::OatsError::action_failed("Target not found"))?;

        let health_trait = target
            .get_trait("health")
            .ok_or_else(|| oats_framework::OatsError::trait_not_found("health"))?;

        let current_health = health_trait.data().as_number().ok_or_else(|| {
            oats_framework::OatsError::action_failed("Health trait is not numeric")
        })?;

        let new_health = (current_health - self.damage).max(0.0);
        let new_health_trait = Trait::new("health", TraitData::Number(new_health));

        let mut result = ActionResult::success();
        result.add_trait_update(new_health_trait);
        result.add_message(format!(
            "Dealt {:.1} damage to {}. Health: {:.1} -> {:.1}",
            self.damage,
            target.name(),
            current_health,
            new_health
        ));

        if new_health <= 0.0 {
            result.add_message(format!("{} has been defeated!", target.name()));
        }

        Ok(result)
    }
}

struct MovementAction {
    new_x: f64,
    new_y: f64,
}

impl MovementAction {
    fn new(x: f64, y: f64) -> Self {
        Self { new_x: x, new_y: y }
    }
}

#[async_trait::async_trait]
impl Action for MovementAction {
    fn name(&self) -> &str {
        "movement"
    }

    fn description(&self) -> &str {
        "Moves character to new position"
    }

    async fn execute(
        &self,
        context: ActionContext,
    ) -> Result<ActionResult, oats_framework::OatsError> {
        let target = context
            .get_object("target")
            .ok_or_else(|| oats_framework::OatsError::action_failed("Target not found"))?;

        let mut position_data = HashMap::new();
        position_data.insert("x".to_string(), serde_json::json!(self.new_x));
        position_data.insert("y".to_string(), serde_json::json!(self.new_y));

        let new_position_trait = Trait::new("position", TraitData::Object(position_data));

        let mut result = ActionResult::success();
        result.add_trait_update(new_position_trait);
        result.add_message(format!(
            "{} moved to position ({:.1}, {:.1})",
            target.name(),
            self.new_x,
            self.new_y
        ));

        Ok(result)
    }
}

// Custom game systems
struct CombatSystem {
    name: String,
    description: String,
    stats: oats_framework::systems::SystemStats,
}

impl CombatSystem {
    fn new() -> Self {
        Self {
            name: "combat_system".to_string(),
            description: "Handles combat between characters".to_string(),
            stats: oats_framework::systems::SystemStats::default(),
        }
    }
}

#[async_trait::async_trait]
impl System for CombatSystem {
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
    ) -> Result<Vec<ActionResult>, oats_framework::OatsError> {
        let mut results = Vec::new();
        let start_time = std::time::Instant::now();

        // Find all characters with health and position traits
        let characters: Vec<_> = objects
            .into_iter()
            .filter(|obj| obj.has_trait("health") && obj.has_trait("position"))
            .collect();

        // Simple combat logic: characters attack each other if they're close
        for i in 0..characters.len() {
            for j in (i + 1)..characters.len() {
                let _char1 = &characters[i];
                let char2 = &characters[j];

                // Check if characters are close enough to fight (simplified)
                let mut rng = StdRng::from_entropy();
                let distance = rng.gen_range(0.0..10.0); // Random distance for demo
                let damage = rng.gen_range(5.0..15.0);

                if distance < 3.0 {
                    let combat_action = CombatAction::new(damage);

                    let mut context = ActionContext::new();
                    context.add_object("target", char2.clone());

                    match combat_action.execute(context).await {
                        Ok(result) => {
                            results.push(result);
                            self.stats.actions_executed += 1;
                        }
                        Err(e) => {
                            self.stats.errors += 1;
                            let error_result = ActionResult::failure(format!("Combat failed: {e}"));
                            results.push(error_result);
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

struct MovementSystem {
    name: String,
    description: String,
    stats: oats_framework::systems::SystemStats,
}

impl MovementSystem {
    fn new() -> Self {
        Self {
            name: "movement_system".to_string(),
            description: "Handles character movement".to_string(),
            stats: oats_framework::systems::SystemStats::default(),
        }
    }
}

#[async_trait::async_trait]
impl System for MovementSystem {
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
    ) -> Result<Vec<ActionResult>, oats_framework::OatsError> {
        let mut results = Vec::new();
        let start_time = std::time::Instant::now();

        for object in objects {
            if object.has_trait("position") {
                // Random movement for demo
                let mut rng = StdRng::from_entropy();
                let new_x = rng.gen_range(-10.0..10.0);
                let new_y = rng.gen_range(-10.0..10.0);

                let movement_action = MovementAction::new(new_x, new_y);
                let mut context = ActionContext::new();
                context.add_object("target", object);

                match movement_action.execute(context).await {
                    Ok(result) => {
                        results.push(result);
                        self.stats.actions_executed += 1;
                    }
                    Err(e) => {
                        self.stats.errors += 1;
                        let error_result = ActionResult::failure(format!("Movement failed: {e}"));
                        results.push(error_result);
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
    println!("🎮 OATS Game Example");
    println!("====================\n");

    // Create game characters
    println!("1. Creating game characters...");

    let mut player = Object::new("hero", "player");
    let health_trait = Trait::new("health", TraitData::Number(100.0));
    let mut position_data = HashMap::new();
    position_data.insert("x".to_string(), serde_json::json!(0.0));
    position_data.insert("y".to_string(), serde_json::json!(0.0));
    let position_trait = Trait::new("position", TraitData::Object(position_data));
    player.add_trait(health_trait);
    player.add_trait(position_trait);

    let mut enemy1 = Object::new("goblin", "enemy");
    let enemy_health = Trait::new("health", TraitData::Number(30.0));
    let mut enemy_pos_data = HashMap::new();
    enemy_pos_data.insert("x".to_string(), serde_json::json!(2.0));
    enemy_pos_data.insert("y".to_string(), serde_json::json!(2.0));
    let enemy_position = Trait::new("position", TraitData::Object(enemy_pos_data));
    enemy1.add_trait(enemy_health);
    enemy1.add_trait(enemy_position);

    let mut enemy2 = Object::new("orc", "enemy");
    let orc_health = Trait::new("health", TraitData::Number(60.0));
    let mut orc_pos_data = HashMap::new();
    orc_pos_data.insert("x".to_string(), serde_json::json!(-1.0));
    orc_pos_data.insert("y".to_string(), serde_json::json!(1.0));
    let orc_position = Trait::new("position", TraitData::Object(orc_pos_data));
    enemy2.add_trait(orc_health);
    enemy2.add_trait(orc_position);

    println!("   Created hero: {} (health: 100)", player.name());
    println!("   Created goblin: {} (health: 30)", enemy1.name());
    println!("   Created orc: {} (health: 60)", enemy2.name());

    // Create game systems
    println!("\n2. Creating game systems...");

    let combat_system = CombatSystem::new();
    let movement_system = MovementSystem::new();

    println!("   Created combat system: {}", combat_system.name());
    println!("   Created movement system: {}", movement_system.name());

    // Set up system manager
    println!("\n3. Setting up game world...");

    let mut game_world = SystemManager::new();
    game_world.add_system(Box::new(combat_system));
    game_world.add_system(Box::new(movement_system));

    // Register characters
    game_world.register_object(player).await;
    game_world.register_object(enemy1).await;
    game_world.register_object(enemy2).await;

    println!("   Registered {} systems", game_world.systems().len());
    println!("   Registered {} characters", 3);

    // Simulate game rounds
    println!("\n4. Simulating game rounds...");

    for round in 1..=5 {
        println!("\n   --- Round {round} ---");

        let results = game_world.process_all(Priority::Normal).await?;

        for result in results {
            if result.is_success() {
                for message in &result.messages {
                    println!("     {message}");
                }
            } else {
                for message in &result.messages {
                    println!("     ❌ {message}");
                }
            }
        }

        // Check character status
        let all_objects = game_world.get_all_objects().await;
        for obj in all_objects {
            if let Some(health_trait) = obj.get_trait("health") {
                if let Some(health) = health_trait.data().as_number() {
                    println!("     {} health: {:.1}", obj.name(), health);
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Final statistics
    println!("\n5. Game statistics:");

    let stats = game_world.get_all_stats();
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

    println!("\n🎉 Game simulation completed!");
    Ok(())
}
