//! This crate exposes the [`ComponentGroup`](trait.ComponentGroup.html) trait.
//! This trait is used to make managing a group of [`specs::Component`] fields easier, even across
//! multiple [`specs::World`] instances.
//!
//! The `component_group_derive` crate (documented below) provides a custom derive that you can use
//! to automatically implement the trait. This can save you from writing a lot of boilerplate.
//!
//! # Motivation
//!
//! The purpose of this trait is to make moving a group of components between worlds very easy and
//! less error-prone. Without grouping all the components to be moved in one place, it is very easy
//! to forget to update the different parts of your code that deal with creating, fetching, and
//! updating the entire group.
//!
//! This example is meant to show what manually implementing this trait can be like. It is quite
//! cumbersome, so a custom derive is provided to generate the implementation automatically. See
//! below for more details about using the custom derive.
//!
//! ```rust
//! // Rust 2018 edition
//! // Don't forget to add component_group as a dependency to your Cargo.toml file!
//! use component_group::ComponentGroup;
//! use specs::{World, Builder, Entity, Component, VecStorage, ReadStorage, WriteStorage, Join};
//! use specs::error::Error as SpecsError;
//! use specs_derive::Component;
//!
//! // The one benefit of implementing the trait manually is that you don't need to make the
//! // fields Clone like you do with the custom derive.
//! #[derive(Debug, Component)]
//! #[storage(VecStorage)]
//! pub struct Position {x: i32, y: i32}
//!
//! #[derive(Debug, Component)]
//! #[storage(VecStorage)]
//! pub struct Velocity {x: i32, y: i32}
//!
//! #[derive(Debug, Component)]
//! #[storage(VecStorage)]
//! pub struct Health(u32);
//!
//! pub struct PlayerComponents {
//!     position: Position,
//!     velocity: Velocity,
//!     health: Health,
//! }
//!
//! impl ComponentGroup for PlayerComponents {
//!     type UpdateError = SpecsError;
//!
//!     fn first_from_world(world: &World) -> Option<Self> {
//!         // Needs to be updated every time the struct changes
//!         let (positions, velocities, healths) = world.system_data::<(
//!             ReadStorage<Position>,
//!             ReadStorage<Velocity>,
//!             ReadStorage<Health>,
//!         )>();
//!         (&positions, &velocities, &healths).join().next().map(|(pos, vel, health)| Self {
//!             // No need to clone because we know and can access all the fields
//!             position: Position {x: pos.x, y: pos.y},
//!             velocity: Velocity {x: vel.x, y: vel.y},
//!             health: Health(health.0),
//!         })
//!     }
//!
//!     fn from_world(entity: Entity, world: &World) -> Self {
//!         // Needs to be updated every time the struct changes
//!         let (positions, velocities, healths) = world.system_data::<(
//!             ReadStorage<Position>,
//!             ReadStorage<Velocity>,
//!             ReadStorage<Health>,
//!         )>();
//!         Self {
//!             // If any of these fields were Clone, we could call Option::cloned on the result
//!             // of `get(entity)` and avoid some of this boilerplate
//!             position: positions.get(entity).map(|pos| Position {x: pos.x, y: pos.y})
//!                 .expect("bug: expected a Position component to be present"),
//!             velocity: velocities.get(entity).map(|vel| Velocity {x: vel.x, y: vel.y})
//!                 .expect("bug: expected a Velocity component to be present"),
//!             health: healths.get(entity).map(|health| Health(health.0))
//!                 .expect("bug: expected a Health component to be present"),
//!         }
//!     }
//!
//!     fn create(self, world: &mut World) -> Entity {
//!         // It's possible to write this code so that the compiler will at the very least
//!         // warn you if you forget one of the things you need to change when the struct
//!         // changes.
//!
//!         // Using pattern matching here forces a compiler error whenever the struct changes
//!         let Self {position, velocity, health} = self;
//!
//!         // Forgetting to add a .with() call will cause an unused variable warning
//!         world.create_entity()
//!             .with(position)
//!             .with(velocity)
//!             .with(health)
//!             .build()
//!     }
//!
//!     fn update(self, entity: Entity, world: &mut World) -> Result<(), Self::UpdateError> {
//!         // Needs to be updated every time the struct changes
//!         let (mut positions, mut velocities, mut healths) = world.system_data::<(
//!             WriteStorage<Position>,
//!             WriteStorage<Velocity>,
//!             WriteStorage<Health>,
//!         )>();
//!
//!         positions.insert(entity, self.position)?;
//!         velocities.insert(entity, self.velocity)?;
//!         healths.insert(entity, self.health)?;
//!         Ok(())
//!     }
//! }
//!
//! # use specs::Entities;
//! # fn find_player_entity(world: &World) -> Entity {
//! #     world.system_data::<Entities>().join().next().unwrap() // cheat since only one entity
//! # }
//! #
//! fn main() -> Result<(), SpecsError> {
//!     // Start the player on level 1
//!     let mut level1 = World::new();
//!     # level1.register::<Position>(); level1.register::<Velocity>(); level1.register::<Health>();
//!     // Having all the components together in a struct means that Rust will enforce that you
//!     // never forget a field. You can still forget to add a component to the group.
//!     // The custom derive below makes it so that adding that is just a one-line change.
//!     let player = PlayerComponents {
//!         position: Position {x: 12, y: 59},
//!         velocity: Velocity {x: -1, y: 2},
//!         health: Health(5),
//!     };
//!     // Add the player to the level
//!     player.create(&mut level1);
//!
//!     // ...
//!
//!     // Player needs to move on to the next level
//!     let mut level2 = World::new();
//!     # level2.register::<Position>(); level2.register::<Velocity>(); level2.register::<Health>();
//!     // Somehow find the player in the world it was just in
//!     let player_entity = find_player_entity(&level1);
//!     // Extract the player from the world it was just in
//!     let player = PlayerComponents::from_world(player_entity, &level1);
//!     // Add it to the next world since it hasn't been added yet
//!     player.create(&mut level2);
//!
//!     // ...
//!
//!     // Player needs to go back to previous level
//!     // Using first_from_world is safe when you know that there is only one entity with all
//!     // the components in the group
//!     let player = PlayerComponents::first_from_world(&level2).unwrap();
//!     let player_entity = find_player_entity(&level1);
//!     // Move the player back
//!     player.update(player_entity, &mut level1)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Custom Derive
//!
//! The `component_group_derive` crate makes this even easier by removing all of the boilerplate
//! involved in implementing this trait. That derive requires that all fields be Clone so that they
//! can be copied within the methods.
//!
//! ```rust
//! // Rust 2018 edition
//! use component_group::ComponentGroup;
//! // Note that you need to add component_group_derive as a dependency to your Cargo.toml file.
//! use component_group_derive::ComponentGroup;
//! use specs::{World, Component, VecStorage};
//! use specs::error::Error as SpecsError;
//! use specs_derive::Component;
//!
//! // Note that components need to be Clone to use the automatic derive
//! #[derive(Debug, Clone, Component)]
//! #[storage(VecStorage)]
//! pub struct Position {x: i32, y: i32}
//!
//! #[derive(Debug, Clone, Component)]
//! #[storage(VecStorage)]
//! pub struct Velocity {x: i32, y: i32}
//!
//! #[derive(Debug, Clone, Component)]
//! #[storage(VecStorage)]
//! pub struct Health(u32);
//!
//! #[derive(ComponentGroup)]
//! struct PlayerComponents {
//!     position: Position,
//!     velocity: Velocity,
//!     health: Health,
//! }
//!
//! # use specs::{Entity, Entities, Join};
//! # fn find_player_entity(world: &World) -> Entity {
//! #     world.system_data::<Entities>().join().next().unwrap() // cheat since only one entity
//! # }
//! #
//! fn main() -> Result<(), SpecsError> {
//!     // Start the player on level 1
//!     let mut level1 = World::new();
//!     # level1.register::<Position>(); level1.register::<Velocity>(); level1.register::<Health>();
//!     // Having all the components together in a struct means that Rust will enforce that you
//!     // never forget a field. You can still forget to add a component to the group, but at
//!     // least that is just a one-line change thanks to the custom derive.
//!     let player = PlayerComponents {
//!         position: Position {x: 12, y: 59},
//!         velocity: Velocity {x: -1, y: 2},
//!         health: Health(5),
//!     };
//!     // Add the player to the level
//!     player.create(&mut level1);
//!
//!     // ...
//!
//!     // Player needs to move on to the next level
//!     let mut level2 = World::new();
//!     # level2.register::<Position>(); level2.register::<Velocity>(); level2.register::<Health>();
//!     // Somehow find the player in the world it was just in
//!     let player_entity = find_player_entity(&level1);
//!     // Extract the player from the world it was just in
//!     let player = PlayerComponents::from_world(player_entity, &level1);
//!     // Add it to the next world since it hasn't been added yet
//!     player.create(&mut level2);
//!
//!     // ...
//!
//!     // Player needs to go back to previous level
//!     // Using first_from_world is safe when you know that there is only one entity with all
//!     // the components in the group
//!     let player = PlayerComponents::first_from_world(&level2).unwrap();
//!     let player_entity = find_player_entity(&level1);
//!     // Move the player back
//!     player.update(player_entity, &mut level1)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Optional Fields
//!
//! You can also use `Option` to ignore part of the group if it isn't specified during creation or
//! if it isn't available in the `World` during extraction. If the field is `None`, a call to
//! `update` will actually remove that component for that entity.
//!
//! ```rust
//! # use component_group::ComponentGroup;
//! # use component_group_derive::ComponentGroup;
//! # use specs::{World, Component, VecStorage, HashMapStorage};
//! # use specs::error::Error as SpecsError;
//! # use specs_derive::Component;
//! # #[derive(Debug, Clone, Component)]
//! # #[storage(VecStorage)]
//! # pub struct Position {x: i32, y: i32}
//! #
//! # #[derive(Debug, Clone, Component)]
//! # #[storage(VecStorage)]
//! # pub struct Velocity {x: i32, y: i32}
//! #
//! # #[derive(Debug, Clone, Component)]
//! # #[storage(VecStorage)]
//! # pub struct Health(u32);
//! // (same components as before)
//!
//! #[derive(Debug, Clone, Component)]
//! #[storage(HashMapStorage)]
//! pub struct Animation {frame: usize}
//!
//! #[derive(ComponentGroup)]
//! struct PlayerComponents {
//!     position: Position,
//!     velocity: Velocity,
//!     health: Health,
//!     // Allowed to not be present
//!     animation: Option<Animation>
//! }
//!
//! # use specs::{Entity, Entities, Join};
//! # fn find_player_entity(world: &World) -> Entity {
//! #     world.system_data::<Entities>().join().next().unwrap() // cheat since only one entity
//! # }
//! #
//! fn main() -> Result<(), SpecsError> {
//!     // Start the player on level 1
//!     let mut level1 = World::new();
//!     # level1.register::<Position>(); level1.register::<Velocity>(); level1.register::<Health>(); level1.register::<Animation>();
//!     let player = PlayerComponents {
//!         position: Position {x: 12, y: 59},
//!         velocity: Velocity {x: -1, y: 2},
//!         health: Health(5),
//!         // The Animation component will only be added to the created entity if this is not None
//!         animation: None, // Not animated to begin with
//!     };
//!     player.create(&mut level1);
//!
//!     // ...
//!
//!     // Player needs to move on to the next level
//!     let mut level2 = World::new();
//!     # level2.register::<Position>(); level2.register::<Velocity>(); level2.register::<Health>(); level2.register::<Animation>();
//!     // If an Animation component was added between the call to create() and this next call, the
//!     // field will be set to Some(...). Otherwise, it will be None.
//!     let player_entity = find_player_entity(&level1);
//!     let player = PlayerComponents::from_world(player_entity, &level1);
//!     player.create(&mut level2);
//!
//!     // ...
//!
//!     // Player needs to go back to previous level
//!     // The Animation component may have changed/added/removed, but we don't need to worry
//!     // about that here! The behaviour is the same as above.
//!     let player = PlayerComponents::first_from_world(&level2).unwrap();
//!     let player_entity = find_player_entity(&level1);
//!     // If the animation field is not None, we will call Storage::insert and add it to the
//!     // component's storage. Otherwise, we will call Storage::remove and get rid of it.
//!     player.update(player_entity, &mut level1)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! **Note:** The way we match for the `Option` type is very naive right now. Using
//! `Option<YourComponent>` as the type of your field will work, but using
//! `std::option::Option<YourComponent>` will not.
//!
//! # Fetching Multiple Component Group Instances
//!
//! In the future, when [Generic Associated Types (GATs)] are implemented, this trait may be
//! updated as follows:
//!
//! ```rust,ignore
//! pub trait ComponentGroup: Sized {
//!     type UpdateError;
//!     type GroupIter<'a>;
//!
//!     // Extracts all instances of this group of components from the world.
//!     fn all_from_world<'a>(world: &'a World) -> Self::GroupIter<'a>;
//!     // ...other methods...
//! }
//! ```
//!
//! It just isn't possible to express this as part of the trait right now. Adding this would be a
//! breaking change, so that update would not occur without a new major version being released.
//!
//! As a workaround, you can add the method yourself using the impl Trait feature:
//!
//! ```rust,no_run
//! # use component_group::ComponentGroup;
//! # use component_group_derive::ComponentGroup;
//! # use specs::{World, Component, VecStorage, ReadStorage, Join};
//! # use specs::error::Error as SpecsError;
//! # use specs_derive::Component;
//! #
//! # #[derive(Debug, Clone, Component)]
//! # #[storage(VecStorage)]
//! # pub struct Position {x: i32, y: i32}
//! #
//! # #[derive(Debug, Clone, Component)]
//! # #[storage(VecStorage)]
//! # pub struct Velocity {x: i32, y: i32}
//! #
//! # #[derive(Debug, Clone, Component)]
//! # #[storage(VecStorage)]
//! # pub struct Health(u32);
//! #
//! #[derive(ComponentGroup)]
//! struct PlayerComponents {
//!     position: Position,
//!     velocity: Velocity,
//!     health: Health,
//! }
//!
//! impl PlayerComponents {
//!     pub fn all_from_world<'a>(world: &'a World) -> impl Iterator<Item=Self> + 'a {
//!         // ...implement this...
//!         # (0..).map(|_| unimplemented!())
//!     }
//! }
//!
//! # use specs::{Entity, Entities};
//! # fn find_player_entity(world: &World) -> Entity {
//! #     world.system_data::<Entities>().join().next().unwrap() // cheat since only one entity
//! # }
//! #
//! fn main() {
//!     let mut level1 = World::new();
//!     // ...do stuff...
//!
//!     for group in PlayerComponents::all_from_world(&level1) {
//!         // ...do stuff with each group...
//!     }
//! }
//! ```
//!
//! [`specs::Component`]: ../specs/trait.Component.html
//! [`specs::World`]: ../specs/world/struct.World.html
//! [Generic Associated Types (GATs)]: https://github.com/rust-lang/rust/issues/44265

#![deny(unused_must_use)]

use specs::{World, Entity};

/// Represents a group of [`specs::Component`] fields that can be added or extracted from
/// a [`specs::World`].
///
/// See the [top-level crate documentation](index.html) for more details.
///
/// [`specs::Component`]: ../specs/trait.Component.html
/// [`specs::World`]: ../specs/world/struct.World.html
pub trait ComponentGroup: Sized {
    /// The error type from the [`update` method](#tymethod.update)
    type UpdateError;

    /// Extracts the first instance of the component group from the world.
    fn first_from_world(world: &World) -> Option<Self>;
    /// Extracts this group of components for the given entity from the given world
    ///
    /// Panics if one of the component fields could not be populated. This can happen if the
    /// component does not exist for this entity. Use Option in the field type to avoid this.
    fn from_world(entity: Entity, world: &World) -> Self;
    /// Create a new entity in the world and add all the components from this group.
    fn create(self, world: &mut World) -> Entity;
    /// Update the components of a given entity with all of the components from this group.
    ///
    /// Note: Any additional components that the entity has other than the ones covered by
    /// the fields of this group will be left untouched.
    fn update(self, entity: Entity, world: &mut World) -> Result<(), Self::UpdateError>;
}
