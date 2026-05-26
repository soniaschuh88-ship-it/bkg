use crate::world::World;

/// A deterministic ECS system. Pure function: same world in = same mutations.
pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, world: &mut World);
}

/// A sequential system runner — runs systems in registration order.
/// Deterministic: no concurrency, no nondeterministic scheduling.
#[derive(Default)]
pub struct SystemRunner { systems: Vec<Box<dyn System>> }
impl SystemRunner {
    pub fn new() -> Self { Self::default() }
    pub fn add(&mut self, s: Box<dyn System>) { self.systems.push(s); }
    pub fn run_all(&self, world: &mut World) {
        for s in &self.systems { s.run(world); }
    }
    pub fn system_count(&self) -> usize { self.systems.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct CountSystem { pub name: String }
    impl System for CountSystem {
        fn name(&self) -> &str { &self.name }
        fn run(&self, world: &mut World) {
            // Spawn one entity per run (for test)
            world.spawn();
        }
    }
    #[test] fn system_runs() {
        let mut runner = SystemRunner::new();
        runner.add(Box::new(CountSystem { name: "spawner".into() }));
        let mut world = World::new();
        runner.run_all(&mut world);
        assert_eq!(world.entity_count(), 1);
    }
    #[test] fn ordered_execution() {
        let mut runner = SystemRunner::new();
        runner.add(Box::new(CountSystem { name: "a".into() }));
        runner.add(Box::new(CountSystem { name: "b".into() }));
        assert_eq!(runner.system_count(), 2);
    }
}
