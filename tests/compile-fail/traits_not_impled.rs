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
//~| ERROR no method named `cloned` found for enum `Option<&NotClone>` in the current scope [E0599]
//~| ERROR the trait bound `Neither: specs::Component` is not satisfied [E0277]
//~| ERROR the trait bound `Neither: specs::Component` is not satisfied [E0277]
//~| ERROR no method named `get` found for
//~| ERROR no method named `get` found for
//~| ERROR no method named `insert` found for
//~| ERROR no method named `insert` found for
//~| ERROR no method named `remove` found for
//~| ERROR no method named `remove` found for
//~| ERROR no method named `join` found for
struct PlayerComponents2 {
    position: Option<Position>,
    velocity: Velocity,
    a: NotImplComponent,
    b: NotClone,
    //~^ ERROR the trait bound `NotClone: Clone` is not satisfied [E0277]
    c: Neither,
    //~^ ERROR the trait bound `Neither: Clone` is not satisfied [E0277]
}

#[derive(ComponentGroup)]
//~^ ERROR the trait bound `NotImplComponent: specs::Component` is not satisfied [E0277]
//~| ERROR no method named `cloned` found for enum `Option<&NotClone>` in the current scope [E0599]
//~| ERROR the trait bound `Neither: specs::Component` is not satisfied [E0277]
//~| ERROR no method named `maybe` found for
//~| ERROR no method named `maybe` found for
//~| ERROR no method named `get` found for
//~| ERROR no method named `get` found for
//~| ERROR no method named `insert` found for
//~| ERROR no method named `remove` found for
//~| ERROR no method named `insert` found for
//~| ERROR no method named `remove` found for
struct PlayerComponents3 {
    position: Option<Position>,
    velocity: Velocity,
    a: Option<NotImplComponent>,
    //~^ ERROR the trait bound `NotImplComponent: specs::Component` is not satisfied [E0277]
    b: Option<NotClone>,
    c: Option<Neither>,
    //~^ ERROR the trait bound `Neither: specs::Component` is not satisfied [E0277]
}

fn main() {}
