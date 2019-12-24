// Tests that generic structs work and are properly bounded

extern crate component_group;
extern crate specs;
extern crate specs_derive;

use component_group::ComponentGroup;
use specs::{Component, VecStorage, NullStorage};
use specs_derive::Component;

trait Foo {}

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Position {x: i32, y: i32}

impl Foo for Position {}

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Velocity {x: i32, y: i32}

// This struct should not have any errors
#[derive(ComponentGroup)]
struct PlayerComponents<T, U> where
    T: Foo + Send + Sync + Component + Clone,
    U: Send + Sync + Component + Clone
{
    position: Position,
    foo: T,
    bar: U,
}

// impls Component + Clone, but not Foo
#[derive(Debug, Default, Clone, Component)]
#[storage(NullStorage)]
struct NotImplFoo;

// impls Component, but not Foo + Clone
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
struct NotClone;

type NotImplFooPlayerComponents = PlayerComponents<NotImplFoo, Velocity>;

fn foo1(c: PlayerComponents<Position, Velocity>) {} // No error
fn foo2(c: PlayerComponents<Position, NotImplFoo>) {} // No error
fn foo3(c: PlayerComponents<NotImplFoo, Velocity>) {}
//~^ ERROR the trait bound `NotImplFoo: Foo` is not satisfied [E0277]
fn foo4(c: PlayerComponents<Position, NotClone>) {} // No error
//~^ ERROR the trait bound `NotClone: std::clone::Clone` is not satisfied [E0277]
fn foo5(c: PlayerComponents<NotClone, Velocity>) {} // No error
//~^ ERROR the trait bound `NotClone: std::clone::Clone` is not satisfied [E0277]
//~| ERROR the trait bound `NotClone: Foo` is not satisfied [E0277]

fn main() {}
