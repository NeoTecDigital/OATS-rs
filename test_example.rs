use oats_framework::{
    Action, ActionContext, ActionResult, Object, Priority, System, SystemManager, Trait, TraitData,
};

// Custom test actions
struct TestIncrementAction {
    trait_name: String,
    increment: f64,
}

impl TestIncrementAction {
    fn new(trait_name: impl Into<String>, increment: f64) -> Self {
        Self {
            trait_name: trait_name.into(),
            increment,
        }
    }
}

#[async_trait::async_trait]
impl Action for TestIncrementAction {
    fn name(&self) -> &str {
        "test_increment"
    }

    fn description(&self) -> &str {
        "Test action that increments a trait"
    }

    async fn execute(
        &self,
        context: ActionContext,
    ) -> Result<ActionResult, oats_framework::OatsError> {
        let target_object = context
            .get_object("target")
            .ok_or_else(|| oats_framework::OatsError::action_failed("Target object not found"))?;

        let current_trait = target_object
            .get_trait(&self.trait_name)
            .ok_or_else(|| oats_framework::OatsError::trait_not_found(&self.trait_name))?;

        let current_value = current_trait
            .data()
            .as_number()
            .ok_or_else(|| oats_framework::OatsError::action_failed("Trait is not numeric"))?;

        let new_value = current_value + self.increment;
        let new_trait = Trait::new(&self.trait_name, TraitData::Number(new_value));

        let mut result = ActionResult::success();
        result.add_trait_update(new_trait);
        result.add_message(format!(
            "Incremented {} from {} to {}",
            self.trait_name, current_value, new_value
        ));

        Ok(result)
    }
}

// Custom test system
struct TestHealthSystem {
    name: String,
    description: String,
    stats: oats_framework::systems::SystemStats,
}

impl TestHealthSystem {
    fn new() -> Self {
        Self {
            name: "test_health_system".to_string(),
            description: "Test health system".to_string(),
            stats: oats_framework::systems::SystemStats::default(),
        }
    }
}

#[async_trait::async_trait]
impl System for TestHealthSystem {
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
            if object.has_trait("health") {
                let health_action = TestIncrementAction::new("health", -1.0);
                let mut context = ActionContext::new();
                context.add_object("target", object);

                match health_action.execute(context).await {
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
    println!("🧪 Testing OATS Implementation");
    println!("==============================\n");

    // Test 1: Create objects and traits
    println!("1. Testing object and trait creation...");
    let mut player = Object::new("test_player", "character");
    let health_trait = Trait::new("health", TraitData::Number(100.0));
    player.add_trait(health_trait);

    assert_eq!(player.name(), "test_player");
    assert_eq!(player.trait_count(), 1);
    assert!(player.has_trait("health"));
    println!("   ✅ Object and trait creation works");

    // Test 2: Test action execution
    println!("2. Testing action execution...");
    let heal_action = TestIncrementAction::new("health", 25.0);
    let mut context = ActionContext::new();
    context.add_object("target", player.clone());

    let result = heal_action.execute(context).await?;
    assert!(result.is_success());
    assert_eq!(result.trait_updates.len(), 1);
    println!("   ✅ Action execution works");

    // Test 3: Test system processing
    println!("3. Testing system processing...");
    let mut health_system = TestHealthSystem::new();
    let objects = vec![player];
    let results = health_system.process(objects, Priority::Normal).await?;
    assert!(!results.is_empty());
    println!("   ✅ System processing works");

    // Test 4: Test system manager
    println!("4. Testing system manager...");
    let mut manager = SystemManager::new();
    manager.add_system(Box::new(TestHealthSystem::new()));

    let test_object = Object::with_traits(
        "test_obj",
        "test_type",
        vec![Trait::new("health", TraitData::Number(50.0))],
    );
    manager.register_object(test_object).await;

    let results = manager.process_all(Priority::Normal).await?;
    assert!(!results.is_empty());
    println!("   ✅ System manager works");

    // Test 5: Test serialization
    println!("5. Testing serialization...");
    let obj = Object::new("serializable", "type");
    let json = serde_json::to_string(&obj)?;
    let deserialized: Object = serde_json::from_str(&json)?;
    assert_eq!(obj.name(), deserialized.name());
    println!("   ✅ Serialization works");

    println!("\n🎉 All tests passed! OATS implementation is working correctly.");
    Ok(())
}
