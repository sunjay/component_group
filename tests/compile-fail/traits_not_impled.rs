// Tests for using the derive with types (Optional or not) that don't implement the right traits

extern crate component_group;
extern crate specs;
extern crate specs_derive;

use component_group::ComponentGroup;
use specs::{World, Component, VecStorage, NullStorage};
use specs_derive::Component;

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Position {x: i32, y: i32}

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Velocity {x: i32, y: i32}

#[derive(ComponentGroup)]
struct PlayerComponents { // This should not have any errors
    position: Option<Position>,
    velocity: Velocity,
}

// impls Clone, but not Component
#[derive(Debug, Default, Clone)]
struct NotImplComponent;

// impls Component, but not Clone
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
struct NotClone;

// impls neither Component nor Clone
#[derive(Debug)]
struct Neither;

#[derive(ComponentGroup)]
//~^ ERROR the trait bound `NotImplComponent: specs::Component` is not satisfied [E0277]
//~| ERROR the trait bound `NotImplComponent: specs::Component` is not satisfied [E0277]
//~| ERROR the trait bound `NotClone: std::clone::Clone` is not satisfied [E0277]
//~| ERROR no method named `cloned` found for type `std::option::Option<&NotClone>` in the current scope [E0599]
//~| ERROR the trait bound `Neither: specs::Component` is not satisfied [E0277]
//~| ERROR the trait bound `Neither: specs::Component` is not satisfied [E0277]
//~| ERROR the trait bound `Neither: std::clone::Clone` is not satisfied [E0277]
//~| ERROR no method named `get` found for type `specs::Storage<'_, NotImplComponent, specs::shred::Fetch<'_, specs::storage::MaskedStorage<NotImplComponent>>>` in the current scope [E0599]
//~| ERROR no method named `get` found for type `specs::Storage<'_, Neither, specs::shred::Fetch<'_, specs::storage::MaskedStorage<Neither>>>` in the current scope [E0599]
//~| ERROR no method named `insert` found for type `specs::Storage<'_, NotImplComponent, specs::shred::FetchMut<'_, specs::storage::MaskedStorage<NotImplComponent>>>` in the current scope [E0599]
//~| ERROR no method named `insert` found for type `specs::Storage<'_, Neither, specs::shred::FetchMut<'_, specs::storage::MaskedStorage<Neither>>>` in the current scope [E0599]
//~| ERROR no method named `join` found for type
struct PlayerComponents2 {
    position: Option<Position>,
    velocity: Velocity,
    a: NotImplComponent,
    b: NotClone,
    c: Neither,
}

#[derive(ComponentGroup)]
//~^ ERROR the trait bound `NotImplComponent: specs::Component` is not satisfied [E0277]
//~| ERROR the trait bound `NotImplComponent: specs::Component` is not satisfied [E0277]
//~| ERROR no method named `cloned` found for type `std::option::Option<&NotClone>` in the current scope [E0599]
//~| ERROR the trait bound `Neither: specs::Component` is not satisfied [E0277]
//~| ERROR the trait bound `Neither: specs::Component` is not satisfied [E0277]
//~| ERROR no method named `maybe` found for type `specs::Storage<'_, NotImplComponent, specs::shred::Fetch<'_, specs::storage::MaskedStorage<NotImplComponent>>>` in the current scope [E0599]
//~| ERROR no method named `maybe` found for type `specs::Storage<'_, Neither, specs::shred::Fetch<'_, specs::storage::MaskedStorage<Neither>>>` in the current scope [E0599]
//~| ERROR no method named `get` found for type `specs::Storage<'_, NotImplComponent, specs::shred::Fetch<'_, specs::storage::MaskedStorage<NotImplComponent>>>` in the current scope [E0599]
//~| ERROR no method named `get` found for type `specs::Storage<'_, Neither, specs::shred::Fetch<'_, specs::storage::MaskedStorage<Neither>>>` in the current scope [E0599]
//~| ERROR no method named `insert` found for type `specs::Storage<'_, NotImplComponent, specs::shred::FetchMut<'_, specs::storage::MaskedStorage<NotImplComponent>>>` in the current scope [E0599]
//~| ERROR no method named `remove` found for type `specs::Storage<'_, NotImplComponent, specs::shred::FetchMut<'_, specs::storage::MaskedStorage<NotImplComponent>>>` in the current scope [E0599]
//~| ERROR no method named `insert` found for type `specs::Storage<'_, Neither, specs::shred::FetchMut<'_, specs::storage::MaskedStorage<Neither>>>` in the current scope [E0599]
//~| ERROR no method named `remove` found for type `specs::Storage<'_, Neither, specs::shred::FetchMut<'_, specs::storage::MaskedStorage<Neither>>>` in the current scope [E0599]
struct PlayerComponents3 {
    position: Option<Position>,
    velocity: Velocity,
    a: Option<NotImplComponent>,
    b: Option<NotClone>,
    c: Option<Neither>,
}
