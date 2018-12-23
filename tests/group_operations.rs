use component_group::ComponentGroup;

use specs::{World, Entity, Component, VecStorage, HashMapStorage, NullStorage, ReadStorage, WriteStorage};
use specs::error::Error as SpecsError;
use specs_derive::Component;

#[derive(Debug, Clone, Component, PartialEq, Eq)]
#[storage(VecStorage)]
pub struct Position {x: i32, y: i32}

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
#[storage(VecStorage)]
pub struct Health(u32);

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
#[storage(HashMapStorage)]
pub struct Animation {frame: usize}

#[derive(Debug, Clone, Copy, Default, Component, PartialEq, Eq)]
#[storage(NullStorage)]
pub struct NotInGroup;

#[derive(ComponentGroup, Debug, Clone, PartialEq, Eq)]
struct PlayerComponents {
    position: Position,
    health: Health,
    // This component is allowed to not be present
    animation: Option<Animation>,
}

fn new_world() -> World {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Health>();
    world.register::<Animation>();
    world.register::<NotInGroup>();
    world
}

fn get<C: Component + Clone>(world: &World, entity: Entity) -> Option<C> {
    let storage = world.system_data::<ReadStorage<C>>();
    storage.get(entity).cloned()
}

fn insert<C: Component + Clone>(world: &mut World, entity: Entity, value: C) {
    let mut storage = world.system_data::<WriteStorage<C>>();
    storage.insert(entity, value).unwrap();
}

fn remove<C: Component + Clone>(world: &mut World, entity: Entity) {
    let mut storage = world.system_data::<WriteStorage<C>>();
    storage.remove(entity).unwrap();
}

#[test]
fn create_with_optional_component() {
    let mut world = new_world();
    let player = PlayerComponents {
        // required components - both must be added
        position: Position {x: 12, y: 59},
        health: Health(5),
        // not added since None
        animation: None,
    };
    let entity = player.create(&mut world);

    assert_eq!(get(&world, entity), Some(Position {x: 12, y: 59}));
    assert_eq!(get(&world, entity), Some(Health(5)));
    assert_eq!(get(&world, entity), None::<Animation>);

    // only components in the group are added
    assert_eq!(get(&world, entity), None::<NotInGroup>);

    let player = PlayerComponents {
        // required components - both must be added
        position: Position {x: 12, y: 59},
        health: Health(5),
        // added since Some(...)
        animation: Some(Animation {frame: 2}),
    };
    let entity = player.create(&mut world);

    assert_eq!(get(&world, entity), Some(Position {x: 12, y: 59}));
    assert_eq!(get(&world, entity), Some(Health(5)));
    assert_eq!(get(&world, entity), Some(Animation {frame: 2}));

    // normal insertion still works
    assert_eq!(get(&world, entity), None::<NotInGroup>);
    insert(&mut world, entity, NotInGroup);
    assert_eq!(get(&world, entity), Some(NotInGroup));
}

#[test]
fn load_change_after_modifying() {
    let mut world = new_world();
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: None,
    };
    let entity = player.create(&mut world);

    // should be able to observe created components with from_world and first_from_world
    let loaded_player = PlayerComponents::from_world(&world, entity);
    assert_eq!(loaded_player.position, Position {x: 12, y: 59});
    assert_eq!(loaded_player.health, Health(5));
    assert_eq!(loaded_player.animation, None);

    let (entity2, loaded_player) = PlayerComponents::first_from_world(&world)
        .expect("expected at least one group");
    assert_eq!(entity, entity2);
    assert_eq!(loaded_player.position, Position {x: 12, y: 59});
    assert_eq!(loaded_player.health, Health(5));
    assert_eq!(loaded_player.animation, None);

    // should be able to insert a new value and observe the change
    let new_value = Health(12);
    assert_ne!(loaded_player.health, new_value);
    insert(&mut world, entity, new_value);

    // should be the changed value with everything else untouched
    let loaded_player = PlayerComponents::from_world(&world, entity);
    assert_eq!(loaded_player.position, Position {x: 12, y: 59});
    assert_eq!(loaded_player.health, new_value);
    assert_eq!(loaded_player.animation, None);

    let (_, loaded_player) = PlayerComponents::first_from_world(&world)
        .expect("expected at least one group");
    assert_eq!(loaded_player.position, Position {x: 12, y: 59});
    assert_eq!(loaded_player.health, new_value);
    assert_eq!(loaded_player.animation, None);
}

#[test]
fn load_first_without_required_component() {
    let mut world = new_world();

    // should return None when there is nothing in the world just yet
    assert!(PlayerComponents::first_from_world(&world).is_none());

    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: None,
    };
    let entity = player.create(&mut world);

    // Starts by returning Some(...) since we added a complete instance of the group
    assert!(PlayerComponents::first_from_world(&world).is_some());

    // If a required component is removed, returns None
    remove::<Health>(&mut world, entity);
    assert!(PlayerComponents::first_from_world(&world).is_none());

    // This is despite the fact that other components in the group are still there
    assert_eq!(get(&world, entity), Some(Position {x: 12, y: 59}));
}

#[test]
fn load_first_without_optional_component() {
    let mut world = new_world();
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: Some(Animation {frame: 2}),
    };
    let entity = player.create(&mut world);

    // Starts by returning Some(...) since we added a complete instance of the group
    let (entity2, loaded_player) = PlayerComponents::first_from_world(&world)
        .expect("expected at least one group");
    assert_eq!(entity, entity2);

    // Removing the optional component
    assert_eq!(loaded_player.animation, Some(Animation {frame: 2}));
    remove::<Animation>(&mut world, entity);

    // should still succeed with Some(...), but that field should now be None
    let (entity2, loaded_player) = PlayerComponents::first_from_world(&world)
        .expect("expected at least one group");
    assert_eq!(entity, entity2);
    assert_eq!(loaded_player.animation, None);
}

#[test]
#[should_panic(expected = "expected a Health component to be present")]
fn load_without_required_component() {
    let mut world = new_world();

    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: None,
    };
    let entity = player.create(&mut world);

    // Starts by returning successfully since we added a complete instance of the group
    let loaded_player = PlayerComponents::from_world(&world, entity);
    assert_eq!(loaded_player.position, Position {x: 12, y: 59});
    assert_eq!(loaded_player.health, Health(5));
    assert_eq!(loaded_player.animation, None);

    // If a required component is removed, panics!
    remove::<Health>(&mut world, entity);
    PlayerComponents::from_world(&world, entity);
}

#[test]
fn load_without_optional_component() {
    let mut world = new_world();
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: Some(Animation {frame: 2}),
    };
    let entity = player.create(&mut world);

    // Starts by returning successfully since we added a complete instance of the group
    let loaded_player = PlayerComponents::from_world(&world, entity);

    // Make sure that optional component currently exists
    assert_eq!(loaded_player.animation, Some(Animation {frame: 2}));
    // Removing the optional component
    remove::<Animation>(&mut world, entity);

    // should still succeed, but that field should now be None
    let loaded_player = PlayerComponents::from_world(&world, entity);
    assert_eq!(loaded_player.animation, None);

    // Re-inserting the optional component with a different value
    let new_value = Animation {frame: 44};
    insert(&mut world, entity, new_value);

    // should still succeed, but that field should now be Some(...)
    let loaded_player = PlayerComponents::from_world(&world, entity);
    assert_eq!(loaded_player.animation, Some(new_value));
}

#[test]
fn load_multiple() {
    let mut world = new_world();

    // can create multiple entities with different values
    let player1 = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: Some(Animation {frame: 2}),
    };
    let player2 = PlayerComponents {
        position: Position {x: -10, y: 78},
        health: Health(230),
        animation: None,
    };
    // two different groups
    assert_ne!(player1, player2);

    let entity1 = player1.create(&mut world);
    let entity2 = player2.create(&mut world);
    // two different entities
    assert_ne!(entity1, entity2);

    // from_world should not return the same components for distinct entities
    // since both the entities and their components are different
    let loaded_player1 = PlayerComponents::from_world(&world, entity1);
    let loaded_player2 = PlayerComponents::from_world(&world, entity2);
    assert_ne!(loaded_player1, loaded_player2);

    // first_from_world should always return the same value
    // The order isn't guaranteed, so with multiple instances we can't know which will be returned,
    // but it should always be deterministic.
    let (first1, first1_player) = PlayerComponents::first_from_world(&world)
        .expect("expected at least one group");
    let (first2, first2_player) = PlayerComponents::first_from_world(&world)
        .expect("expected at least one group");
    assert_eq!(first1, first2);
    assert_eq!(first1_player, first2_player);
}

#[test]
fn update_with_non_group_component() -> Result<(), SpecsError> {
    let mut world = new_world();
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: Some(Animation {frame: 2}),
    };
    let entity = player.create(&mut world);
    let loaded_player = PlayerComponents::from_world(&world, entity);

    // Add a component that is not part of the group
    insert(&mut world, entity, NotInGroup);
    assert_eq!(get(&world, entity), Some(NotInGroup));

    // Update all values
    let player = PlayerComponents {
        position: Position {x: 32, y: -30},
        health: Health(8),
        animation: Some(Animation {frame: 4}),
    };
    player.update(&mut world, entity)?;

    // All values part of the group should be changed
    let updated_player = PlayerComponents::from_world(&world, entity);
    assert_ne!(loaded_player, updated_player);

    // update should not update components that aren't in the group
    assert_eq!(get(&world, entity), Some(NotInGroup));

    Ok(())
}

#[test]
fn update_optional_field() -> Result<(), SpecsError> {
    let mut world = new_world();
    let frame = 2;
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: Some(Animation {frame}),
    };
    let entity = player.create(&mut world);

    // Make sure that optional component currently exists
    assert_eq!(get(&world, entity), Some(Animation {frame}));

    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        // None - update should now explicitly remove the component
        animation: None,
    };
    player.update(&mut world, entity)?;

    // Optional component is now gone
    assert_eq!(get(&world, entity), None::<Animation>);

    let frame2 = 33;
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        // Some - update should now explicitly insert the component
        animation: Some(Animation {frame: frame2}),
    };
    player.update(&mut world, entity)?;

    // Optional component has been re-inserted
    assert_ne!(get(&world, entity), Some(Animation {frame}));
    assert_eq!(get(&world, entity), Some(Animation {frame: frame2}));

    let frame3 = 128;
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        // Some - update should now explicitly change the component value
        animation: Some(Animation {frame: frame3}),
    };
    player.update(&mut world, entity)?;

    // Optional component has been changed
    assert_ne!(get(&world, entity), Some(Animation {frame}));
    assert_ne!(get(&world, entity), Some(Animation {frame: frame2}));
    assert_eq!(get(&world, entity), Some(Animation {frame: frame3}));

    Ok(())
}

#[test]
fn update_should_overwrite() -> Result<(), SpecsError> {
    let mut world = new_world();
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: Some(Animation {frame: 2}),
    };
    let entity = player.create(&mut world);
    let loaded_player = PlayerComponents::from_world(&world, entity);

    // This value should get overwritten by update
    let overwritten_value = Health(100);
    assert_ne!(get(&world, entity), Some(overwritten_value));
    insert(&mut world, entity, overwritten_value);
    // Make sure the value to be overwritten was in fact present at some point
    assert_eq!(get(&world, entity), Some(overwritten_value));

    // Update to the original values
    loaded_player.update(&mut world, entity)?;

    // Overwritten value should be changed
    assert_ne!(get(&world, entity), Some(overwritten_value));

    Ok(())
}

#[test]
fn move_non_group_should_not_be_moved() {
    let mut world = new_world();
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: None,
    };
    let entity = player.create(&mut world);

    // Add a component that is not part of the group
    assert_eq!(get(&world, entity), None::<NotInGroup>);
    insert(&mut world, entity, NotInGroup);
    assert_eq!(get(&world, entity), Some(NotInGroup));

    assert_eq!(get(&world, entity), Some(Position {x: 12, y: 59}));
    assert_eq!(get(&world, entity), Some(Health(5)));
    assert_eq!(get(&world, entity), None::<Animation>);

    // Move group to another world
    let mut world2 = new_world();
    let (_, player) = PlayerComponents::first_from_world(&world).unwrap();
    player.create(&mut world2);

    // everything should have been added exactly as-is from the first world
    assert_eq!(get(&world2, entity), Some(Position {x: 12, y: 59}));
    assert_eq!(get(&world2, entity), Some(Health(5)));
    assert_eq!(get(&world2, entity), None::<Animation>);

    // However, the non-group component should not have been moved
    assert_eq!(get(&world2, entity), None::<NotInGroup>);
    // Still exists in first world
    assert_eq!(get(&world, entity), Some(NotInGroup));
}

#[test]
fn moved_components_modify_independently() {
    let mut world = new_world();
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: None,
    };
    let entity = player.create(&mut world);

    assert_eq!(get(&world, entity), Some(Position {x: 12, y: 59}));
    assert_eq!(get(&world, entity), Some(Health(5)));
    assert_eq!(get(&world, entity), None::<Animation>);

    // Move group to another world
    let mut world2 = new_world();
    let (_, player) = PlayerComponents::first_from_world(&world).unwrap();
    player.create(&mut world2);

    assert_eq!(get(&world2, entity), Some(Position {x: 12, y: 59}));
    assert_eq!(get(&world2, entity), Some(Health(5)));
    assert_eq!(get(&world2, entity), None::<Animation>);

    // modifying after move doesn't modify the components from the original world
    let new_value = Health(32);
    assert_ne!(get::<Health>(&world2, entity).unwrap(), new_value);
    insert(&mut world2, entity, new_value);
    assert_eq!(get(&world2, entity), Some(new_value));
    // first world is still the same
    assert_eq!(get(&world, entity), Some(Health(5)));
    assert_ne!(get::<Health>(&world, entity).unwrap(), new_value);
}

#[test]
fn remove_with_non_group_components() {
    let mut world = new_world();
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: Some(Animation {frame: 2}),
    };
    let entity = player.create(&mut world);

    assert_eq!(get(&world, entity), Some(Position {x: 12, y: 59}));
    assert_eq!(get(&world, entity), Some(Health(5)));
    assert_eq!(get(&world, entity), Some(Animation {frame: 2}));

    // Add a component that is not part of the group
    assert_eq!(get(&world, entity), None::<NotInGroup>);
    insert(&mut world, entity, NotInGroup);
    assert_eq!(get(&world, entity), Some(NotInGroup));

    // Remove the group from the world
    let removed_player = PlayerComponents::remove(&mut world, entity);
    assert_eq!(removed_player.position, Position {x: 12, y: 59});
    assert_eq!(removed_player.health, Health(5));
    assert_eq!(removed_player.animation, Some(Animation {frame: 2}));

    // all group components are removed
    assert_eq!(get(&world, entity), None::<Position>);
    assert_eq!(get(&world, entity), None::<Health>);
    assert_eq!(get(&world, entity), None::<Animation>);

    // non-group component still exists
    assert_eq!(get(&world, entity), Some(NotInGroup));
}

#[test]
#[should_panic(expected = "expected a Health component to be present")]
fn remove_required_component_not_present() {
    let mut world = new_world();
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: Some(Animation {frame: 2}),
    };
    let entity = player.create(&mut world);

    // If a required component is not present for removal, panics!
    remove::<Health>(&mut world, entity);
    PlayerComponents::remove(&mut world, entity);
}

#[test]
fn remove_optional_component_not_present() {
    let mut world = new_world();
    let player = PlayerComponents {
        position: Position {x: 12, y: 59},
        health: Health(5),
        animation: Some(Animation {frame: 2}),
    };
    let entity = player.create(&mut world);

    // Make sure that optional component currently exists
    assert_eq!(get(&world, entity), Some(Animation {frame: 2}));
    // Removing the optional component
    remove::<Animation>(&mut world, entity);

    // Remove succeeds without panicking, but sets that field to None
    let removed_player = PlayerComponents::remove(&mut world, entity);
    assert_eq!(removed_player.position, Position {x: 12, y: 59});
    assert_eq!(removed_player.health, Health(5));
    assert_eq!(removed_player.animation, None);
}
