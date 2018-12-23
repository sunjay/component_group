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

#[derive(ComponentGroup, Debug, PartialEq, Eq)]
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
    let loaded_player = PlayerComponents::from_world(entity, &world);
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
    let loaded_player = PlayerComponents::from_world(entity, &world);
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
    let loaded_player = PlayerComponents::from_world(entity, &world);
    assert_eq!(loaded_player.position, Position {x: 12, y: 59});
    assert_eq!(loaded_player.health, Health(5));
    assert_eq!(loaded_player.animation, None);

    // If a required component is removed, panics!
    remove::<Health>(&mut world, entity);
    PlayerComponents::from_world(entity, &world);
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
    let loaded_player = PlayerComponents::from_world(entity, &world);

    // Make sure that optional component currently exists
    assert_eq!(loaded_player.animation, Some(Animation {frame: 2}));
    // Removing the optional component
    remove::<Animation>(&mut world, entity);

    // should still succeed, but that field should now be None
    let loaded_player = PlayerComponents::from_world(entity, &world);
    assert_eq!(loaded_player.animation, None);

    // Re-inserting the optional component with a different value
    let new_value = Animation {frame: 44};
    insert(&mut world, entity, new_value);

    // should still succeed, but that field should now be Some(...)
    let loaded_player = PlayerComponents::from_world(entity, &world);
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
    let loaded_player1 = PlayerComponents::from_world(entity1, &world);
    let loaded_player2 = PlayerComponents::from_world(entity2, &world);
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
fn update_with_non_group_component() {
    // update should not update components that aren't in the group
    unimplemented!()
}

#[test]
fn update_optional_field() {
    // Updating to None - component is removed
    // Updating to Some - component is inserted
    // Updating to Some (again) - component is updated to new value
    unimplemented!()
}

#[test]
fn update_should_overwrite() {
    // create
    // loaded = first_from_world()
    // insert() - modifies a component
    // first_from_world - has the modified values
    // update(loaded)
    // first_from_world - has the values of loaded, not the modified values
    unimplemented!()
}

#[test]
fn move_non_group_should_not_be_moved() {
    // create in world 1
    // insert(non group)
    // assert everything exists as expected
    // first_from_world
    // create in world 2
    // assert everything exists as expected, except the non-group component
    // non-group component still exists in world 1
    unimplemented!()
}

#[test]
fn moved_components_modify_independently() {
    // modifying after move doesn't modify the components from the original world the components were moved from
    unimplemented!()
}

#[test]
fn remove_does_not_remove_non_group_components() {
    // non-group component still exists
    unimplemented!()
}

#[test]
#[should_panic(expected = "")]
fn remove_required_component_not_present() {
    // panics if a required component could not be removed
    unimplemented!()
}

#[test]
fn remove_optional_component_not_present() {
    // sets the field to None if it was optional and not present in the world during remove
    unimplemented!()
}
