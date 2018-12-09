// Tests for using the derive on unsupported types (enums, union, unit structs, empty structs)

extern crate component_group;
extern crate component_group_derive;
extern crate specs;
extern crate specs_derive;

use component_group::ComponentGroup;
use component_group_derive::ComponentGroup;
use specs::{World, Component, VecStorage};
use specs_derive::Component;

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Position {x: i32, y: i32}

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Velocity {x: i32, y: i32}

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Health(u32);

#[derive(ComponentGroup)]
struct PlayerComponents { // This should not have any errors
    position: Position,
    velocity: Velocity,
    health: Health,
}

#[derive(ComponentGroup)]
enum PlayerComponents2 { //~ ERROR Only structs with named fields are supported
    Position(Position),
    Velocity(Velocity),
}

#[derive(ComponentGroup)]
enum PlayerComponents3 { //~ ERROR Only structs with named fields are supported
    // empty
}

#[derive(ComponentGroup)]
struct PlayerComponents4; //~ ERROR Only structs with named fields are supported

#[derive(ComponentGroup)]
struct PlayerComponents5 { //~ ERROR struct must have at least one field to derive ComponentGroup
    // empty
}
