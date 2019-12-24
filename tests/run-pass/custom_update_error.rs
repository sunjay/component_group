extern crate component_group;
extern crate specs;
extern crate specs_derive;

use component_group::ComponentGroup;
use specs::{World, WorldExt, Builder, Entity, Entities, Component, VecStorage, ReadStorage, WriteStorage, Join};
use specs::error::Error as SpecsError;
use specs_derive::Component;

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Position {x: i32, y: i32}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Velocity {x: i32, y: i32}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Health(u32);

pub struct PlayerComponents {
    position: Position,
    velocity: Velocity,
    health: Health,
}

#[derive(Debug)]
pub enum InvalidUpdate {
    OutOfBounds(Position),
    SpecsError(SpecsError),
}

impl From<SpecsError> for InvalidUpdate {
    fn from(e: SpecsError) -> Self {
        InvalidUpdate::SpecsError(e)
    }
}

impl ComponentGroup for PlayerComponents {
    type UpdateError = InvalidUpdate;

    fn first_from_world(world: &World) -> Option<(Entity, Self)> {
        // Needs to be updated every time the struct changes
        let (entities, positions, velocities, healths) = world.system_data::<(
            Entities,
            ReadStorage<Position>,
            ReadStorage<Velocity>,
            ReadStorage<Health>,
        )>();
        (&entities, &positions, &velocities, &healths).join().next()
            .map(|(entity, pos, vel, health)| (entity, Self {
                // No need to clone because we know and can access all the fields
                position: Position {x: pos.x, y: pos.y},
                velocity: Velocity {x: vel.x, y: vel.y},
                health: Health(health.0),
            }))
    }

    fn from_world(world: &World, entity: Entity) -> Self {
        // Needs to be updated every time the struct changes
        let (positions, velocities, healths) = world.system_data::<(
            ReadStorage<Position>,
            ReadStorage<Velocity>,
            ReadStorage<Health>,
        )>();
        Self {
            // If any of these fields were Clone, we could call Option::cloned on the result
            // of `get(entity)` and avoid some of this boilerplate
            position: positions.get(entity).map(|pos| Position {x: pos.x, y: pos.y})
                .expect("expected a Position component to be present"),
            velocity: velocities.get(entity).map(|vel| Velocity {x: vel.x, y: vel.y})
                .expect("expected a Velocity component to be present"),
            health: healths.get(entity).map(|health| Health(health.0))
                .expect("expected a Health component to be present"),
        }
    }

    fn create(self, world: &mut World) -> Entity {
        // It's possible to write this code so that the compiler will at the very least
        // warn you if you forget one of the things you need to change when the struct
        // changes.

        // Using pattern matching here forces a compiler error whenever the struct changes
        let Self {position, velocity, health} = self;

        // Forgetting to add a .with() call will cause an unused variable warning
        world.create_entity()
            .with(position)
            .with(velocity)
            .with(health)
            .build()
    }

    fn update(self, world: &mut World, entity: Entity) -> Result<(), Self::UpdateError> {
        // don't update if position is out of bounds
        let Position {x, y} = self.position;
        if x < -20 || y < -20 || x > 20 || y > 20 {
            return Err(InvalidUpdate::OutOfBounds(self.position));
        }

        // Needs to be updated every time the struct changes
        let (mut positions, mut velocities, mut healths) = world.system_data::<(
            WriteStorage<Position>,
            WriteStorage<Velocity>,
            WriteStorage<Health>,
        )>();

        positions.insert(entity, self.position)?;
        velocities.insert(entity, self.velocity)?;
        healths.insert(entity, self.health)?;
        Ok(())
    }

    fn remove(world: &mut World, entity: Entity) -> Self {
        // Needs to be updated every time the struct changes
        let (mut positions, mut velocities, mut healths) = world.system_data::<(
            WriteStorage<Position>,
            WriteStorage<Velocity>,
            WriteStorage<Health>,
        )>();
        Self {
            // If any of these fields were Clone, we could call Option::cloned on the result
            // of `get(entity)` and avoid some of this boilerplate
            position: positions.remove(entity).map(|pos| Position {x: pos.x, y: pos.y})
                .expect("expected a Position component to be present"),
            velocity: velocities.remove(entity).map(|vel| Velocity {x: vel.x, y: vel.y})
                .expect("expected a Velocity component to be present"),
            health: healths.remove(entity).map(|health| Health(health.0))
                .expect("expected a Health component to be present"),
        }
    }
}

fn main() {}
