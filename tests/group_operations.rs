use component_group::ComponentGroup;

use specs::{World, Entity, Component, VecStorage, HashMapStorage, NullStorage, ReadStorage, WriteStorage};
use specs::error::Error as SpecsError;
use specs_derive::Component;

#[derive(Debug, Clone, Component, PartialEq, Eq)]
#[storage(VecStorage)]
pub struct Position {x: i32, y: i32}

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
    // required - added
    // optional, None - not added
    // optional, Some() - added
    unimplemented!()
}

#[test]
fn load_change_after_modifying() {
    // create
    // get -> should have old value
    // from_world -> should have old_value
    // insert(new_value)
    // get -> should have new value
    // from_world -> should have new_value
    unimplemented!()
}

#[test]
fn load_first_without_required_component() {
    // first_from_world - returns None before group is ever inserted

    // first_from_world - load group that is only partially in the world
    // should return None
    unimplemented!()
}

#[test]
fn load_first_without_optional_component() {
    // first_from_world - returns None before group is ever inserted

    // first_from_world - load group that is only partially in the world
    // should return Some, but have None for that component
    unimplemented!()
}

#[test]
#[should_panic(expected = "")]
fn load_without_required_component() {
    // create
    // from_world - succeeds and has the value for that field
    // remove(required component)
    // from_world - panics if a required component can't be loaded
    unimplemented!()
}

#[test]
fn load_without_optional_component() {
    // create
    // from_world - succeeds and has the value for that field
    // remove(optional component)
    // from_world - does not panic and just returns None for that field
    // insert(optional component)
    // from_world - succeeds and has the value for that field
    unimplemented!()
}

#[test]
fn load_multiple() {
    // create x2
    // from_world x2
    // values should be different (i.e. same entity is not always loaded)
    // first_from_world should always be the first created
    unimplemented!()
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
