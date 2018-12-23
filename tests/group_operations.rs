use component_group::ComponentGroup;

use specs::{World, Entity, Component, VecStorage, HashMapStorage, NullStorage, ReadStorage, WriteStorage};
use specs::error::Error as SpecsError;
use specs_derive::Component;

#[derive(Debug, Clone, Component, PartialEq, Eq)]
#[storage(VecStorage)]
pub struct Position {x: i32, y: i32}

#[derive(Debug, Clone, Component, PartialEq, Eq)]
#[storage(VecStorage)]
pub struct Velocity {x: i32, y: i32}

#[derive(Debug, Clone, Component, PartialEq, Eq)]
#[storage(VecStorage)]
pub struct Health(u32);

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
#[storage(HashMapStorage)]
pub struct Animation {frame: usize}

#[derive(Debug, Clone, Default, Component, PartialEq, Eq)]
#[storage(NullStorage)]
pub struct NotInGroup;

#[derive(ComponentGroup)]
struct PlayerComponents {
    position: Position,
    velocity: Velocity,
    health: Health,
    // This component is allowed to not be present
    animation: Option<Animation>,
}

fn get<C: Component + Clone>(world: &World, entity: Entity) -> Option<C> {
    let storage = world.system_data::<ReadStorage<C>>();
    storage.get(entity).cloned()
}

fn insert<C: Component + Clone>(world: &mut World, entity: Entity, value: C) {
    let mut storage = world.system_data::<WriteStorage<C>>();
    storage.insert(entity, value).unwrap();
}

#[test]
fn move_between_levels() -> Result<(), SpecsError> {
    let mut level1 = World::new();
    level1.register::<Position>(); level1.register::<Velocity>(); level1.register::<Health>(); level1.register::<Animation>(); level1.register::<NotInGroup>();
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        velocity: Velocity {x: -1, y: 2},
        health: Health(5),
        animation: None, // Not animated to begin with
    };
    let player_entity = player.create(&mut level1);

    // Insert an entity that isn't in the group
    insert(&mut level1, player_entity, NotInGroup);

    assert_eq!(get(&level1, player_entity), Some(Position {x: 12, y: 59}));
    assert_eq!(get(&level1, player_entity), Some(Velocity {x: -1, y: 2}));
    assert_eq!(get(&level1, player_entity), Some(Health(5)));
    assert_eq!(get::<Animation>(&level1, player_entity), None);
    // Should exist
    assert_eq!(get(&level1, player_entity), Some(NotInGroup));

    // Modify entity
    let new_health = 100;
    // unwrap() to make sure that this is still Some but the inner value is different
    assert_ne!(get::<Health>(&level1, player_entity).unwrap(), Health(new_health));
    insert(&mut level1, player_entity, Health(new_health));
    // Should have changed
    assert_eq!(get(&level1, player_entity), Some(Health(new_health)));

    /////// Player needs to move on to the next level ///////

    let mut level2 = World::new();
    level2.register::<Position>(); level2.register::<Velocity>(); level2.register::<Health>(); level2.register::<Animation>(); level2.register::<NotInGroup>();
    let player = PlayerComponents::from_world(player_entity, &level1);
    // Should be the changed value
    assert_eq!(player.health, Health(new_health));
    let player_entity2 = player.create(&mut level2);

    // All values should be present
    assert_eq!(get(&level2, player_entity2), Some(Position {x: 12, y: 59}));
    assert_eq!(get(&level2, player_entity2), Some(Velocity {x: -1, y: 2}));
    assert_eq!(get(&level2, player_entity2), Some(Health(new_health)));
    assert_eq!(get::<Animation>(&level2, player_entity2), None);

    // Should not have been added to level 2
    assert_eq!(get::<NotInGroup>(&level2, player_entity2), None);

    // Modify animation
    let animation = Animation {frame: 11};
    insert(&mut level2, player_entity2, animation);
    assert_eq!(get(&level2, player_entity2), Some(animation));

    // Modifications to level 2 should not impact level 1
    assert_eq!(get(&level1, player_entity), Some(Position {x: 12, y: 59}));
    assert_eq!(get(&level1, player_entity), Some(Velocity {x: -1, y: 2}));
    assert_eq!(get(&level1, player_entity), Some(Health(new_health)));
    assert_eq!(get::<Animation>(&level1, player_entity), None);
    // Should exist
    assert_eq!(get(&level1, player_entity), Some(NotInGroup));

    // Modifications to level 1 should be overwritten during update
    let overwritten_health = 1234;
    // unwrap() to make sure that this is still Some but the inner value is different
    assert_ne!(get::<Health>(&level1, player_entity).unwrap(), Health(overwritten_health));
    insert(&mut level1, player_entity, Health(overwritten_health));
    assert_eq!(get(&level1, player_entity), Some(Health(overwritten_health)));

    /////// Player needs to go back to previous level ///////

    let (_, player) = PlayerComponents::first_from_world(&level2).unwrap();
    // Should still be animated
    assert_eq!(player.animation, Some(animation));
    player.update(player_entity, &mut level1)?;

    // Overwrite of health succeeded
    assert_ne!(get(&level1, player_entity), Some(Health(overwritten_health)));

    // All values should be overwritten
    assert_eq!(get(&level1, player_entity), Some(Position {x: 12, y: 59}));
    assert_eq!(get(&level1, player_entity), Some(Velocity {x: -1, y: 2}));
    assert_eq!(get(&level1, player_entity), Some(Health(new_health)));
    assert_eq!(get(&level1, player_entity), Some(animation));

    // Should not get overwritten since not in group
    assert_eq!(get(&level1, player_entity), Some(NotInGroup));
    assert_eq!(get::<NotInGroup>(&level2, player_entity2), None);

    // Remove player from first level
    PlayerComponents::remove(player_entity, &mut level1);

    // All components *in the group* should be gone
    assert_eq!(get::<Position>(&level1, player_entity), None);
    assert_eq!(get::<Velocity>(&level1, player_entity), None);
    assert_eq!(get::<Health>(&level1, player_entity), None);
    assert_eq!(get::<Animation>(&level1, player_entity), None);

    // Components *not in the group* should still exist (not removed)
    assert_eq!(get(&level1, player_entity), Some(NotInGroup));

    // Removing all the components in the group from one world should not affect the other
    assert_eq!(get(&level2, player_entity2), Some(Position {x: 12, y: 59}));
    assert_eq!(get(&level2, player_entity2), Some(Velocity {x: -1, y: 2}));
    assert_eq!(get(&level2, player_entity2), Some(Health(new_health)));
    assert_eq!(get::<Animation>(&level2, player_entity2), Some(animation));
    assert_eq!(get::<NotInGroup>(&level2, player_entity2), None);

    Ok(())
}
