use async_trait::async_trait;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oats_framework::{
    Action, ActionContext, ActionResult, OatsError, Object, Priority, System, SystemManager, Trait,
    TraitData,
};
use std::collections::HashMap;
use tokio::runtime::Runtime;

// Simple benchmark action
struct SimpleBenchmarkAction;

#[async_trait]
impl Action for SimpleBenchmarkAction {
    fn name(&self) -> &str {
        "simple_benchmark"
    }

    fn description(&self) -> &str {
        "Simple benchmark action"
    }

    async fn execute(&self, _context: ActionContext) -> Result<ActionResult, OatsError> {
        let mut result = ActionResult::success();
        result.add_message("Benchmark action executed");
        Ok(result)
    }
}

// Simple benchmark system
struct SimpleBenchmarkSystem {
    stats: oats_framework::systems::SystemStats,
}

impl SimpleBenchmarkSystem {
    fn new() -> Self {
        Self {
            stats: oats_framework::systems::SystemStats::default(),
        }
    }
}

#[async_trait]
impl System for SimpleBenchmarkSystem {
    fn name(&self) -> &str {
        "simple_benchmark_system"
    }

    fn description(&self) -> &str {
        "Simple benchmark system"
    }

    async fn process(
        &mut self,
        objects: Vec<Object>,
        _priority: Priority,
    ) -> Result<Vec<ActionResult>, OatsError> {
        let mut results = Vec::with_capacity(objects.len());
        let start_time = std::time::Instant::now();

        for _object in objects {
            let action = SimpleBenchmarkAction;
            let context = ActionContext::new();

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

fn create_simple_objects(count: usize) -> Vec<Object> {
    let mut objects = Vec::with_capacity(count);

    for i in 0..count {
        let mut obj = Object::new(format!("object_{}", i), "test_type");
        let health_trait = Trait::new("health", TraitData::Number(100.0));
        obj.add_trait_internal(health_trait);
        objects.push(obj);
    }

    objects
}

fn simple_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Simple OATS Benchmarks");

    // Object creation benchmark
    group.bench_function("object_creation_100", |b| {
        b.iter(|| {
            black_box(create_simple_objects(100));
        });
    });

    // Trait operations benchmark
    group.bench_function("trait_operations", |b| {
        b.iter(|| {
            let mut obj = Object::new("test", "type");
            let traits = vec![
                Trait::new("health", TraitData::Number(100.0)),
                Trait::new("mana", TraitData::Number(50.0)),
                Trait::new("stamina", TraitData::Number(75.0)),
            ];
            obj.add_traits_bulk(traits);
            black_box(obj);
        });
    });

    // Action execution benchmark
    group.bench_function("action_execution", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                let action = SimpleBenchmarkAction;
                let context = ActionContext::new();
                black_box(action.execute(context).await.unwrap());
            });
        });
    });

    // System processing benchmark
    group.bench_function("system_processing_100", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                let mut system = SimpleBenchmarkSystem::new();
                let objects = create_simple_objects(100);
                black_box(system.process(objects, Priority::Normal).await.unwrap());
            });
        });
    });

    // System manager benchmark
    group.bench_function("system_manager_100", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                let mut manager = SystemManager::with_capacity(100);
                let objects = create_simple_objects(100);

                for obj in objects {
                    manager.register_object(obj).await;
                }

                manager.add_system(Box::new(SimpleBenchmarkSystem::new()));
                black_box(manager.process_all(Priority::Normal).await.unwrap());
            });
        });
    });

    group.finish();
}

criterion_group!(benches, simple_benchmarks);
criterion_main!(benches);
