// Tests that generic structs work and are properly bounded

extern crate component_group;
extern crate specs;
extern crate specs_derive;

use component_group::ComponentGroup;
use specs::{Component, VecStorage};
use specs_derive::Component;

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Position {x: i32, y: i32}

trait Foo {}

#[derive(ComponentGroup)]
//~^ ERROR the trait bound `T: specs::Component` is not satisfied [E0277]
//~| ERROR the trait bound `T: specs::Component` is not satisfied [E0277]
//~| ERROR the trait bound `T: std::clone::Clone` is not satisfied [E0277]
//~| ERROR the trait bound `U: specs::Component` is not satisfied [E0277]
//~| ERROR the trait bound `U: specs::Component` is not satisfied [E0277]
//~| ERROR the trait bound `U: std::clone::Clone` is not satisfied [E0277]
//~| ERROR `T` cannot be sent between threads safely [E0277]
//~| ERROR `T` cannot be shared between threads safely [E0277]
//~| ERROR `U` cannot be sent between threads safely [E0277]
//~| ERROR `U` cannot be shared between threads safely [E0277]
//~| ERROR no method named `join` found for type
//~| ERROR no method named `get` found for type
//~| ERROR no method named `get` found for type
//~| ERROR no method named `insert` found for type
//~| ERROR no method named `insert` found for type
struct MissingBounds<T: Foo, U> { // Missing Component + Clone bounds
    position: Position,
    foo: T,
    bar: U,
}
