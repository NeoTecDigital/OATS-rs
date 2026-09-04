use async_trait::async_trait;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oats_framework::{
    Action, ActionContext, ActionResult, OatsError, Object, Priority, System, SystemManager, Trait,
    TraitData,
};
use std::collections::HashMap;
use tokio::runtime::Runtime;

// Benchmark increment action
#[derive(Clone)]
struct BenchmarkIncrementAction {
    trait_name: String,
    increment: f64,
}

impl BenchmarkIncrementAction {
    fn new(trait_name: impl Into<String>, increment: f64) -> Self {
        Self {
            trait_name: trait_name.into(),
            increment,
        }
    }
}

#[async_trait]
impl Action for BenchmarkIncrementAction {
    fn name(&self) -> &str {
        "benchmark_increment"
    }

    fn description(&self) -> &str {
        "Benchmark increment action"
    }

    async fn execute(&self, context: ActionContext) -> Result<ActionResult, OatsError> {
        let target = context.get_object("target").unwrap();
        let current_value = target
            .get_trait(&self.trait_name)
            .and_then(|t| t.data().as_number())
            .unwrap_or(0.0);

        let new_value = current_value + self.increment;
        let new_trait = Trait::new(&self.trait_name, TraitData::Number(new_value));

        let mut result = ActionResult::success();
        result.add_trait_update(new_trait);
        Ok(result)
    }
}

// Benchmark system
struct BenchmarkSystem {
    stats: oats_framework::systems::SystemStats,
}

impl BenchmarkSystem {
    fn new() -> Self {
        Self {
            stats: oats_framework::systems::SystemStats::default(),
        }
    }
}

#[async_trait]
impl System for BenchmarkSystem {
    fn name(&self) -> &str {
        "benchmark_system"
    }

    fn description(&self) -> &str {
        "Benchmark system"
    }

    async fn process(
        &mut self,
        objects: Vec<Object>,
        _priority: Priority,
    ) -> Result<Vec<ActionResult>, OatsError> {
        let mut results = Vec::with_capacity(objects.len());
        let start_time = std::time::Instant::now();

        for object in objects {
            let action = BenchmarkIncrementAction::new("health", 1.0);
            let mut context = ActionContext::new();
            context.add_object("target", object);

            match action.execute(context).await {
                Ok(result) => {
                    results.push(result);
                    self.stats.actions_executed += 1;
                }
                Err(_) => {
                    self.stats.errors += 1;
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

fn create_test_objects(count: usize) -> Vec<Object> {
    let mut objects = Vec::with_capacity(count);

    for i in 0..count {
        let mut obj = Object::new(format!("object_{}", i), "test_type");

        // Add some traits using bulk operation
        let health_trait = Trait::new("health", TraitData::Number(100.0));
        let mut position_data = HashMap::new();
        position_data.insert("x".to_string(), serde_json::json!(i as f64));
        position_data.insert("y".to_string(), serde_json::json!(i as f64));
        let position_trait = Trait::new("position", TraitData::Object(position_data));

        obj.add_traits_bulk(vec![health_trait, position_trait]);

        objects.push(obj);
    }

    objects
}

fn create_complex_objects(count: usize) -> Vec<Object> {
    let mut objects = Vec::with_capacity(count);

    for i in 0..count {
        let mut obj = Object::new(format!("complex_object_{}", i), "complex_type");

        // Add many traits to test batch operations
        let traits = vec![
            Trait::new("health", TraitData::Number(100.0 + i as f64)),
            Trait::new("mana", TraitData::Number(50.0 + i as f64)),
            Trait::new("stamina", TraitData::Number(75.0 + i as f64)),
            Trait::new("level", TraitData::Number(i as f64)),
            Trait::new("experience", TraitData::Number(i as f64 * 100.0)),
            Trait::new("gold", TraitData::Number(i as f64 * 10.0)),
            Trait::new("active", TraitData::Boolean(i % 2 == 0)),
            Trait::new("name", TraitData::String(format!("Player_{}", i))),
        ];

        obj.add_traits_bulk(traits);

        // Add metadata
        obj.set_metadata("created_by", "benchmark");
        obj.set_metadata("version", "1.0");

        objects.push(obj);
    }

    objects
}

fn benchmark_object_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Object Creation");

    group.bench_function("create_100_objects", |b| {
        b.iter(|| {
            black_box(create_test_objects(100));
        });
    });

    group.bench_function("create_1000_objects", |b| {
        b.iter(|| {
            black_box(create_test_objects(1000));
        });
    });

    group.bench_function("create_10000_objects", |b| {
        b.iter(|| {
            black_box(create_test_objects(10000));
        });
    });

    group.bench_function("create_100_complex_objects", |b| {
        b.iter(|| {
            black_box(create_complex_objects(100));
        });
    });

    group.finish();
}

fn benchmark_trait_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Trait Operations");

    group.bench_function("add_traits_individual", |b| {
        b.iter(|| {
            let mut obj = Object::new("test", "type");

            // Add traits individually
            for i in 0..10 {
                let trait_name = format!("trait_{}", i);
                let trait_data = TraitData::Number(i as f64);
                let trait_obj = Trait::new(&trait_name, trait_data);
                obj.add_trait(trait_obj);
            }

            black_box(obj);
        });
    });

    group.bench_function("add_traits_batch", |b| {
        b.iter(|| {
            let mut obj = Object::new("test", "type");

            // Add traits in batch
            let traits: Vec<Trait> = (0..10)
                .map(|i| {
                    let trait_name = format!("trait_{}", i);
                    let trait_data = TraitData::Number(i as f64);
                    Trait::new(&trait_name, trait_data)
                })
                .collect();

            obj.add_traits(traits);
            black_box(obj);
        });
    });

    group.bench_function("add_traits_bulk_optimized", |b| {
        b.iter(|| {
            let mut obj = Object::new("test", "type");

            // Add traits using optimized bulk operation
            let traits: Vec<Trait> = (0..10)
                .map(|i| {
                    let trait_name = format!("trait_{}", i);
                    let trait_data = TraitData::Number(i as f64);
                    Trait::new(&trait_name, trait_data)
                })
                .collect();

            obj.add_traits_bulk(traits);
            black_box(obj);
        });
    });

    group.bench_function("trait_access_zero_copy", |b| {
        let mut obj = Object::new("test", "type");
        let health_trait = Trait::new("health", TraitData::Number(100.0));
        let position_trait = Trait::new("position", TraitData::Object(HashMap::new()));
        obj.add_trait(health_trait);
        obj.add_trait(position_trait);

        b.iter(|| {
            // Test zero-copy access
            black_box(obj.get_trait_data("health"));
            black_box(obj.get_trait_data("position"));
        });
    });

    group.bench_function("batch_trait_validation", |b| {
        let obj = create_complex_objects(1)[0].clone();

        b.iter(|| {
            // Test batch trait checking
            let required_traits = vec!["health", "mana", "stamina", "level"];
            black_box(obj.has_traits(&required_traits));
        });
    });

    group.finish();
}

fn benchmark_action_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("Action Execution");
    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("simple_action", |b| {
        b.iter(|| {
            rt.block_on(async {
                let action = BenchmarkIncrementAction::new("health", 10.0);
                let mut obj = Object::new("test", "type");
                let health_trait = Trait::new("health", TraitData::Number(100.0));
                obj.add_trait(health_trait);

                let mut context = ActionContext::new();
                context.add_object("target", obj);

                black_box(action.execute(context).await.unwrap());
            });
        });
    });

    group.bench_function("increment_action", |b| {
        b.iter(|| {
            rt.block_on(async {
                let action = BenchmarkIncrementAction::new("health", 10.0);
                let mut obj = Object::new("test", "type");
                let health_trait = Trait::new("health", TraitData::Number(100.0));
                obj.add_trait(health_trait);

                let mut context = ActionContext::new();
                context.add_object("target", obj);

                black_box(action.execute(context).await.unwrap());
            });
        });
    });

    group.bench_function("action_with_capacity", |b| {
        b.iter(|| {
            rt.block_on(async {
                let action = BenchmarkIncrementAction::new("health", 10.0);
                let mut obj = Object::new("test", "type");
                let health_trait = Trait::new("health", TraitData::Number(100.0));
                obj.add_trait(health_trait);

                let mut context = ActionContext::with_capacity(2, 1);
                context.add_object("target", obj);

                black_box(action.execute(context).await.unwrap());
            });
        });
    });

    group.finish();
}

fn benchmark_system_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("System Processing");
    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("process_100_objects", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut system = BenchmarkSystem::new();
                let objects = create_test_objects(100);

                black_box(system.process(objects, Priority::Normal).await.unwrap());
            });
        });
    });

    group.bench_function("process_1000_objects", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut system = BenchmarkSystem::new();
                let objects = create_test_objects(1000);

                black_box(system.process(objects, Priority::Normal).await.unwrap());
            });
        });
    });

    group.bench_function("process_10000_objects", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut system = BenchmarkSystem::new();
                let objects = create_test_objects(10000);

                black_box(system.process(objects, Priority::Normal).await.unwrap());
            });
        });
    });

    group.finish();
}

fn benchmark_system_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("System Manager");
    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("manager_with_capacity", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = SystemManager::with_capacity(1000);
                let objects = create_test_objects(100);

                for obj in objects {
                    manager.register_object(obj).await;
                }

                manager.add_system(Box::new(BenchmarkSystem::new()));
                black_box(manager.process_all(Priority::Normal).await.unwrap());
            });
        });
    });

    group.bench_function("multiple_systems", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = SystemManager::new();
                let objects = create_test_objects(100);

                for obj in objects {
                    manager.register_object(obj).await;
                }

                // Add multiple systems
                manager.add_system(Box::new(BenchmarkSystem::new()));
                manager.add_system(Box::new(BenchmarkSystem::new()));
                manager.add_system(Box::new(BenchmarkSystem::new()));

                black_box(manager.process_all(Priority::Normal).await.unwrap());
            });
        });
    });

    group.finish();
}

fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("Serialization");

    group.bench_function("serialize_object", |b| {
        let obj = create_test_objects(1)[0].clone();

        b.iter(|| {
            black_box(serde_json::to_string(&obj).unwrap());
        });
    });

    group.bench_function("deserialize_object", |b| {
        let obj = create_test_objects(1)[0].clone();
        let json = serde_json::to_string(&obj).unwrap();

        b.iter(|| {
            black_box(serde_json::from_str::<Object>(&json).unwrap());
        });
    });

    group.bench_function("serialize_complex_object", |b| {
        let obj = create_complex_objects(1)[0].clone();

        b.iter(|| {
            black_box(serde_json::to_string(&obj).unwrap());
        });
    });

    group.finish();
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Concurrent Operations");
    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("concurrent_object_registration", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = SystemManager::with_capacity(1000);
                let objects = create_test_objects(100);

                // Register objects sequentially to avoid cloning issues
                for obj in objects {
                    manager.register_object(obj).await;
                }

                black_box(manager);
            });
        });
    });

    group.bench_function("concurrent_action_execution", |b| {
        b.iter(|| {
            rt.block_on(async {
                let action = BenchmarkIncrementAction::new("health", 1.0);
                let objects = create_test_objects(100);

                let mut handles = Vec::new();
                for obj in objects {
                    let action = action.clone();
                    handles.push(tokio::spawn(async move {
                        let mut context = ActionContext::new();
                        context.add_object("target", obj);
                        action.execute(context).await.unwrap()
                    }));
                }

                for handle in handles {
                    black_box(handle.await.unwrap());
                }
            });
        });
    });

    group.finish();
}

fn benchmark_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory Efficiency");

    group.bench_function("large_object_creation", |b| {
        b.iter(|| {
            let mut obj = Object::new("large_object", "type");

            // Add many traits to test memory efficiency
            let traits: Vec<Trait> = (0..100)
                .map(|i| {
                    let trait_name = format!("trait_{}", i);
                    let trait_data = TraitData::String(format!("value_{}", i));
                    Trait::new(&trait_name, trait_data)
                })
                .collect();

            obj.add_traits_bulk(traits);
            black_box(obj);
        });
    });

    group.bench_function("batch_trait_operations", |b| {
        b.iter(|| {
            let mut obj = Object::new("batch_test", "type");

            // Create traits in batch
            let traits: Vec<Trait> = (0..50)
                .map(|i| {
                    let trait_name = format!("batch_trait_{}", i);
                    let trait_data = TraitData::Number(i as f64);
                    Trait::new(&trait_name, trait_data)
                })
                .collect();

            obj.add_traits_bulk(traits);
            black_box(obj);
        });
    });

    group.finish();
}

fn benchmark_stress_tests(c: &mut Criterion) {
    let mut group = c.benchmark_group("Stress Tests");
    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("stress_100k_objects", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = SystemManager::with_capacity(100_000);
                let objects = create_test_objects(100_000);

                // Register objects in chunks to avoid memory issues
                for chunk in objects.chunks(1000) {
                    for obj in chunk {
                        manager.register_object(obj.clone()).await;
                    }
                }

                manager.add_system(Box::new(BenchmarkSystem::new()));
                let results = manager.process_all(Priority::Normal).await.unwrap();
                black_box(results.len());
            });
        });
    });

    group.bench_function("stress_many_systems", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = SystemManager::new();
                let objects = create_test_objects(1000);

                for obj in objects {
                    manager.register_object(obj).await;
                }

                // Add many systems to test system management
                for i in 0..50 {
                    let system = BenchmarkSystem::new();
                    manager.add_system(Box::new(system));
                }

                let results = manager.process_all(Priority::Normal).await.unwrap();
                black_box(results.len());
            });
        });
    });

    group.bench_function("stress_concurrent_actions", |b| {
        b.iter(|| {
            rt.block_on(async {
                let action = BenchmarkIncrementAction::new("health", 1.0);
                let objects = create_test_objects(1000);

                let mut handles = Vec::new();
                for obj in objects {
                    let action = action.clone();
                    handles.push(tokio::spawn(async move {
                        let mut context = ActionContext::new();
                        context.add_object("target", obj);
                        action.execute(context).await.unwrap()
                    }));
                }

                let results = futures::future::join_all(handles).await;
                black_box(results.len());
            });
        });
    });

    group.bench_function("stress_large_objects", |b| {
        b.iter(|| {
            let mut objects = Vec::new();

            // Create objects with many traits
            for i in 0..100 {
                let mut obj = Object::new(format!("stress_obj_{}", i), "stress_type");

                // Add many traits to each object
                for j in 0..100 {
                    let trait_name = format!("stress_trait_{}_{}", i, j);
                    let trait_data = TraitData::String(format!("stress_value_{}_{}", i, j));
                    let trait_obj = Trait::new(&trait_name, trait_data);
                    obj.add_trait(trait_obj);
                }

                objects.push(obj);
            }

            black_box(objects);
        });
    });

    group.finish();
}

fn benchmark_throughput_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("Throughput Analysis");
    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("throughput_1k_objects_per_second", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = SystemManager::with_capacity(1000);
                let objects = create_test_objects(1000);

                for obj in objects {
                    manager.register_object(obj).await;
                }

                manager.add_system(Box::new(BenchmarkSystem::new()));
                let start = std::time::Instant::now();
                let results = manager.process_all(Priority::Normal).await.unwrap();
                let duration = start.elapsed();

                let throughput = 1000.0 / duration.as_secs_f64();
                black_box((results.len(), throughput));
            });
        });
    });

    group.bench_function("throughput_10k_objects_per_second", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = SystemManager::with_capacity(10000);
                let objects = create_test_objects(10000);

                for obj in objects {
                    manager.register_object(obj).await;
                }

                manager.add_system(Box::new(BenchmarkSystem::new()));
                let start = std::time::Instant::now();
                let results = manager.process_all(Priority::Normal).await.unwrap();
                let duration = start.elapsed();

                let throughput = 10000.0 / duration.as_secs_f64();
                black_box((results.len(), throughput));
            });
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_object_creation,
    benchmark_trait_operations,
    benchmark_action_execution,
    benchmark_system_processing,
    benchmark_system_manager,
    benchmark_serialization,
    benchmark_concurrent_operations,
    benchmark_memory_efficiency,
    benchmark_stress_tests,
    benchmark_throughput_analysis,
);
criterion_main!(benches);
