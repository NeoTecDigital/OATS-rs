use crate::actions::{ActionResult, TraitUpdate};
use crate::{OatsError, Object, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// System identifier
pub type SystemId = uuid::Uuid;

/// Priority levels for system operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Normal
    }
}

/// A system represents orchestration that coordinates actions and manages resources
#[async_trait]
pub trait System: Send + Sync {
    /// Get the name of this system
    fn name(&self) -> &str;

    /// Get the description of this system
    fn description(&self) -> &str;

    /// Initialize the system
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Shutdown the system
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    /// Process objects with the given priority
    async fn process(
        &mut self,
        objects: Vec<Object>,
        priority: Priority,
    ) -> Result<Vec<ActionResult>>;

    /// Get the priority of this system
    fn priority(&self) -> Priority {
        Priority::Normal
    }

    /// Check if the system is ready to process
    fn is_ready(&self) -> bool {
        true
    }

    /// Get system statistics
    fn get_stats(&self) -> SystemStats {
        SystemStats::default()
    }
}

/// Statistics for a system
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemStats {
    /// Number of objects processed
    pub objects_processed: u64,
    /// Number of actions executed
    pub actions_executed: u64,
    /// Number of errors encountered
    pub errors: u64,
    /// Total processing time in milliseconds
    pub total_processing_time_ms: u64,
    /// Last processing timestamp
    pub last_processed: Option<chrono::DateTime<chrono::Utc>>,
    /// Average processing time per object in milliseconds
    pub avg_processing_time_ms: f64,
    /// Peak processing time in milliseconds
    pub peak_processing_time_ms: u64,
}

impl SystemStats {
    /// Update stats with new processing time
    pub fn update_processing_time(&mut self, processing_time_ms: u64) {
        self.total_processing_time_ms += processing_time_ms;
        self.peak_processing_time_ms = self.peak_processing_time_ms.max(processing_time_ms);

        if self.objects_processed > 0 {
            self.avg_processing_time_ms =
                self.total_processing_time_ms as f64 / self.objects_processed as f64;
        }
    }

    /// Get processing throughput (objects per second)
    pub fn throughput_objects_per_second(&self) -> f64 {
        if self.total_processing_time_ms > 0 {
            (self.objects_processed as f64 * 1000.0) / self.total_processing_time_ms as f64
        } else {
            0.0
        }
    }

    /// Reset all stats
    pub fn reset(&mut self) {
        self.objects_processed = 0;
        self.actions_executed = 0;
        self.errors = 0;
        self.total_processing_time_ms = 0;
        self.avg_processing_time_ms = 0.0;
        self.peak_processing_time_ms = 0;
        self.last_processed = None;
    }

    /// Get error rate as percentage
    pub fn error_rate(&self) -> f64 {
        let total = self.objects_processed + self.actions_executed;
        if total > 0 {
            (self.errors as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Outcome of a writeback pass.
///
/// Applying is deliberately a separate step from proposing: actions read traits
/// and return [`TraitUpdate`]s, and only an explicit call to
/// [`SystemManager::apply`] writes them to the registry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplyReport {
    /// Number of trait values written to the registry
    pub traits_applied: usize,
    /// Number of distinct objects that received at least one trait
    pub objects_updated: usize,
    /// Number of failed results whose updates were deliberately not applied
    pub results_skipped: usize,
}

impl ApplyReport {
    /// True when the pass wrote nothing
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.traits_applied == 0
    }
}

/// A system manager that coordinates multiple systems
pub struct SystemManager {
    systems: HashMap<String, Box<dyn System>>,
    object_registry: Arc<RwLock<HashMap<String, Object>>>,
}

impl SystemManager {
    /// Create a new system manager
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            object_registry: Arc::new(RwLock::new(HashMap::with_capacity(100))),
        }
    }

    /// Create a new system manager with expected capacity
    pub fn with_capacity(expected_objects: usize) -> Self {
        Self {
            systems: HashMap::new(),
            object_registry: Arc::new(RwLock::new(HashMap::with_capacity(expected_objects))),
        }
    }

    /// Add a system to the manager
    pub fn add_system(&mut self, system: Box<dyn System>) {
        let name = system.name().to_string();
        self.systems.insert(name, system);
    }

    /// Remove a system from the manager
    pub fn remove_system(&mut self, name: &str) -> Option<Box<dyn System>> {
        self.systems.remove(name)
    }

    /// Get a system by name
    pub fn get_system(&self, name: &str) -> Option<&dyn System> {
        self.systems.get(name).map(|system| system.as_ref())
    }

    /// Get all systems
    pub fn systems(&self) -> &HashMap<String, Box<dyn System>> {
        &self.systems
    }

    /// Get system count
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Register an object with the manager
    pub async fn register_object(&self, object: Object) {
        let mut registry = self.object_registry.write().await;
        registry.insert(object.id.to_string(), object);
    }

    /// Get an object by ID
    pub async fn get_object(&self, id: &str) -> Option<Object> {
        let registry = self.object_registry.read().await;
        registry.get(id).cloned()
    }

    /// Get all objects
    pub async fn get_all_objects(&self) -> Vec<Object> {
        let registry = self.object_registry.read().await;
        registry.values().cloned().collect()
    }

    /// Get object count
    pub async fn object_count(&self) -> usize {
        let registry = self.object_registry.read().await;
        registry.len()
    }

    /// Clear all objects
    pub async fn clear_objects(&self) {
        let mut registry = self.object_registry.write().await;
        registry.clear();
    }

    /// Reserve capacity for objects
    pub async fn reserve_objects(&self, additional: usize) {
        let mut registry = self.object_registry.write().await;
        registry.reserve(additional);
    }

    /// Apply the trait updates proposed by successful action results.
    ///
    /// This is the writeback half of the OATS loop. Actions never mutate; they
    /// return proposals, and the host decides here whether to accept them.
    /// Updates carried by failed results are never applied - they are counted in
    /// [`ApplyReport::results_skipped`] instead.
    ///
    /// The pass is all-or-nothing: if any update names an object the registry
    /// does not hold, nothing is written and
    /// [`OatsError::ObjectNotFound`] is returned.
    pub async fn apply(&self, results: &[ActionResult]) -> Result<ApplyReport> {
        let mut skipped = 0;
        let mut updates = Vec::new();

        for result in results {
            if result.is_success() {
                updates.extend(result.trait_updates.iter().cloned());
            } else {
                skipped += 1;
            }
        }

        let mut report = self.apply_updates(updates).await?;
        report.results_skipped = skipped;
        Ok(report)
    }

    /// Apply trait updates directly, without going through an [`ActionResult`].
    ///
    /// Takes ownership so a host that already holds the proposals can write them
    /// back without cloning. Same all-or-nothing guarantee as [`Self::apply`].
    pub async fn apply_updates(
        &self,
        updates: impl IntoIterator<Item = TraitUpdate>,
    ) -> Result<ApplyReport> {
        let updates: Vec<TraitUpdate> = updates.into_iter().collect();
        if updates.is_empty() {
            return Ok(ApplyReport::default());
        }

        let mut registry = self.object_registry.write().await;

        // Verify every target first so a bad update cannot half-write the batch.
        for update in &updates {
            let key = update.target.to_string();
            if !registry.contains_key(&key) {
                return Err(OatsError::object_not_found(key));
            }
        }

        let traits_applied = updates.len();
        let mut touched: HashSet<crate::objects::ObjectId> = HashSet::new();
        for update in updates {
            let key = update.target.to_string();
            let target = update.target;
            let object = registry
                .get_mut(&key)
                .expect("target presence verified above");
            object.apply_trait(update.into_trait());
            touched.insert(target);
        }

        Ok(ApplyReport {
            traits_applied,
            objects_updated: touched.len(),
            results_skipped: 0,
        })
    }

    /// Process all objects through all systems
    pub async fn process_all(&mut self, priority: Priority) -> Result<Vec<ActionResult>> {
        let objects = self.get_all_objects().await;
        let mut all_results = Vec::new();

        // Sort systems by priority (highest first)
        let mut system_names: Vec<_> = self.systems.keys().cloned().collect();
        system_names.sort_by(|a, b| {
            let a_priority = self
                .systems
                .get(a)
                .map(|s| s.priority())
                .unwrap_or(Priority::Normal);
            let b_priority = self
                .systems
                .get(b)
                .map(|s| s.priority())
                .unwrap_or(Priority::Normal);
            b_priority.cmp(&a_priority)
        });

        for system_name in system_names {
            if let Some(system) = self.systems.get_mut(&system_name) {
                if system.is_ready() {
                    match system.process(objects.clone(), priority).await {
                        Ok(results) => all_results.extend(results),
                        Err(e) => {
                            let error_result = ActionResult::failure(format!("System error: {e}"));
                            all_results.push(error_result);
                        }
                    }
                }
            }
        }

        Ok(all_results)
    }

    /// Process objects through a specific system
    pub async fn process_with_system(
        &mut self,
        system_name: &str,
        objects: Vec<Object>,
        priority: Priority,
    ) -> Result<Vec<ActionResult>> {
        let system = self
            .systems
            .get_mut(system_name)
            .ok_or_else(|| OatsError::system_error(format!("System '{system_name}' not found")))?;

        if !system.is_ready() {
            return Err(OatsError::system_error("System is not ready"));
        }

        system.process(objects, priority).await
    }

    /// Initialize all systems
    pub async fn initialize_all(&mut self) -> Result<()> {
        for (name, system) in &mut self.systems {
            if let Err(e) = system.initialize().await {
                return Err(OatsError::system_error(format!(
                    "Failed to initialize system '{name}': {e}"
                )));
            }
        }
        Ok(())
    }

    /// Shutdown all systems
    pub async fn shutdown_all(&mut self) -> Result<()> {
        for (name, system) in &mut self.systems {
            if let Err(e) = system.shutdown().await {
                return Err(OatsError::system_error(format!(
                    "Failed to shutdown system '{name}': {e}"
                )));
            }
        }
        Ok(())
    }

    /// Get statistics for all systems
    pub fn get_all_stats(&self) -> HashMap<String, SystemStats> {
        self.systems
            .iter()
            .map(|(name, system)| (name.clone(), system.get_stats()))
            .collect()
    }
}

impl Default for SystemManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }

    #[test]
    fn test_system_stats() {
        let stats = SystemStats::default();
        assert_eq!(stats.objects_processed, 0);
        assert_eq!(stats.actions_executed, 0);
        assert_eq!(stats.errors, 0);
    }
}
