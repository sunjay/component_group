//! This crate exposes the [`ComponentGroup`] trait.
//! This trait is used to make managing a group of [`specs::Component`] instances easier.
//! This is especially important when reading, writing, or modifying them all as a group across
//! multiple [`specs::World`] instances.
//!
//! This crate also provides a custom derive (documented below) that you can use to automatically
//! implement the trait. This can greatly reduce the amount of boilerplate and make modifying your
//! group of components much easier.
//!
//! ```rust,no_run
//! // Don't forget to add the component_group crate to your Cargo.toml file!
//! use component_group::ComponentGroup;
//! # use specs::{World, Component, VecStorage, HashMapStorage};
//! # use specs::error::Error as SpecsError;
//! # use specs_derive::Component;
//!
//! // These components are just for demonstration purposes. You should swap them out for your own
//! // Note that components need to be Clone to use the automatic derive of ComponentGroup
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
//! // This is all of the code you need to write to define the group and all of its operations!
//! #[derive(ComponentGroup)]
//! struct PlayerComponents {
//!     position: Position,
//!     velocity: Velocity,
//!     health: Health,
//! }
//!
//! // Now, you can add all of these components to an entity, load them all from the world with
//! // one line of code, or even update them all at once!
//! ```
//!
//! See the documentation for [`ComponentGroup`] for the exact operations you can perform on the
//! `PlayerComponents` struct. The rest of the documentation below goes into the motivation behind
//! this crate and details about how to use it.
//!
//! # Table of Contents
//!
//! * [Motivation](#motivation)
//! * [Manually Implementing `ComponentGroup`](#manually-implementing-componentgroup)
//! * [Automatically Implementing `ComponentGroup`](#automatically-implementing-componentgroup)
//! * [Optional Fields](#optional-fields)
//! * [Fetching Multiple Component Group Instances](#fetching-multiple-component-group-instances)
//! * [Generic Component Groups](#generic-component-groups)
//!
//! # Motivation
//!
//! The [`ComponentGroup`] trait makes operating on many components at once much easier and less
//! error-prone. Trying to update all of your code every time you add a new component to an entity
//! is very difficult to manage. By grouping all of the components together in a single struct, you
//! can better manage all of them and make fewer mistakes when you edit your code. The following is
//! an example of what your code might look like *without* this trait.
//!
//! ```rust
//! // Rust 2018 edition
//! use specs::{World, Builder, Entity, Component, VecStorage, ReadStorage, WriteStorage, Join};
//! use specs::error::Error as SpecsError;
//! use specs_derive::Component;
//!
//! // Let's setup some components to add to our World
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
//! # use specs::Entities;
//! # fn find_player_entity(world: &World) -> Entity {
//! #     world.system_data::<Entities>().join().next().unwrap() // cheat since only one entity
//! # }
//! #
//! fn main() -> Result<(), SpecsError> {
//!     // Start the player on level 1
//!     let mut level1 = World::new();
//!     # level1.register::<Position>(); level1.register::<Velocity>(); level1.register::<Health>();
//!     // Add the player to the level
//!     level1.create_entity()
//!         // This player only has three components right now, but imagine what could happen
//!         // to this code as that number grows
//!         .with(Position {x: 12, y: 59})
//!         .with(Velocity {x: -1, y: 2})
//!         .with(Health(5))
//!         .build();
//!
//!     // ...
//!
//!     // Player needs to move on to the next level
//!     let mut level2 = World::new();
//!     # level2.register::<Position>(); level2.register::<Velocity>(); level2.register::<Health>();
//!     # { // Need to do this in its own scope (not relevant to the example)
//!     // Somehow find the player in the world it was just in
//!     let player_entity = find_player_entity(&level1);
//!     // Need to fetch all the components of the player. Be careful to keep this in sync with
//!     // the code above!
//!     let (positions, velocities, healths) = level1.system_data::<(
//!         ReadStorage<Position>,
//!         ReadStorage<Velocity>,
//!         ReadStorage<Health>,
//!     )>();
//!     // If any of these fields were Clone, we could call Option::cloned on the result
//!     // of `get(entity)` and avoid some of this boilerplate
//!     let position = positions.get(player_entity).map(|pos| Position {x: pos.x, y: pos.y})
//!         .expect("bug: expected a Position component to be present");
//!     let velocity = velocities.get(player_entity).map(|vel| Velocity {x: vel.x, y: vel.y})
//!         .expect("bug: expected a Velocity component to be present");
//!     let health = healths.get(player_entity).map(|health| Health(health.0))
//!         .expect("bug: expected a Health component to be present");
//!     // Now we can add everything to the new level we created
//!     level2.create_entity()
//!         .with(position)
//!         .with(velocity)
//!         .with(health)
//!         .build();
//!     # } // Need to do this in its own scope (not relevant to the example)
//!
//!     // ...
//!
//!     // Player needs to go back to previous level
//!     // Find the player in level **2**
//!     let player_entity = find_player_entity(&level2);
//!     // That means that we need to now duplicate everything from the above again!
//!     // This time we're fetching from level2, not level1
//!     let (positions, velocities, healths) = level2.system_data::<(
//!         ReadStorage<Position>,
//!         ReadStorage<Velocity>,
//!         ReadStorage<Health>,
//!     )>();
//!     let position = positions.get(player_entity).map(|pos| Position {x: pos.x, y: pos.y})
//!         .expect("bug: expected a Position component to be present");
//!     let velocity = velocities.get(player_entity).map(|vel| Velocity {x: vel.x, y: vel.y})
//!         .expect("bug: expected a Velocity component to be present");
//!     let health = healths.get(player_entity).map(|health| Health(health.0))
//!         .expect("bug: expected a Health component to be present");
//!     // Now that we have the components, we need to re-add them. However, we have to first
//!     // find the player in level **1**
//!     let player_entity = find_player_entity(&level1);
//!     // Now we need to fetch write storages for every component, essentially duplicating the
//!     // code again! Have to make sure we insert into level **1**
//!     let (mut positions, mut velocities, mut healths) = level1.system_data::<(
//!         WriteStorage<Position>,
//!         WriteStorage<Velocity>,
//!         WriteStorage<Health>,
//!     )>();
//!
//!     positions.insert(player_entity, position)?;
//!     velocities.insert(player_entity, velocity)?;
//!     healths.insert(player_entity, health)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Notice how much there is to keep track of in this code! The slightest typo can result in
//! completely incorrect behaviour and very hard to track down bugs. Modifying this code is even
//! worse because you have to search through everywhere you could have added or modified a
//! component and make sure that area gets updated. These changes can propogate throughout your
//! entire codebase and make it very difficult to modify as the number of components grows.
//!
//! This crate is designed to provide a more sustainable approach to this problem. Rather than
//! trying to manage ad-hoc groups of components that could change in any part of the code at any
//! given time, the trait provided by this crate groups all of the components together in a single
//! struct. The code that needs to be written for the operations on these components is unavoidable
//! given the API that specs provides, but at least you can make it easier by grouping everything
//! into a single place.
//!
//! We've tried to make it even easier than that too by providing a way to automatically derive the
//! trait as long as your components implement the `Clone` trait. That means that you can actually
//! get all of the benefits of this trait by just defining a single struct. Everything else is done
//! automatically!
//!
//! The next section of the documentation shows you how to manually implement the [`ComponentGroup`]
//! trait for a given group of components. This is still a lot of boilerplate, but it is all
//! grouped in one place. The section after that shows how to remove all the boilerplate by
//! automatically deriving the trait.
//!
//! # Manually Implementing `ComponentGroup`
//!
//! This example is meant to show what manually implementing this trait can be like. The basic idea
//! is to move all of the duplicated code from above into reusable methods on a struct that groups
//! all of the components that need to be modified together. Implementing it manually like this is
//! still quite cumbersome, so a way to automatically derive the trait is also provided. See below
//! for more details about that.
//!
//! ```rust
//! // Rust 2018 edition
//! // Don't forget to add component_group as a dependency to your Cargo.toml file!
//! use component_group::ComponentGroup;
//! use specs::{World, Builder, Entity, Entities, Component, VecStorage, ReadStorage, WriteStorage, Join};
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
//!     fn first_from_world(world: &World) -> Option<(Entity, Self)> {
//!         // Needs to be updated every time the struct changes
//!         let (entities, positions, velocities, healths) = world.system_data::<(
//!             Entities,
//!             ReadStorage<Position>,
//!             ReadStorage<Velocity>,
//!             ReadStorage<Health>,
//!         )>();
//!         (&entities, &positions, &velocities, &healths).join().next()
//!             .map(|(entity, pos, vel, health)| (entity, Self {
//!                 // No need to clone because we know and can access all the fields
//!                 position: Position {x: pos.x, y: pos.y},
//!                 velocity: Velocity {x: vel.x, y: vel.y},
//!                 health: Health(health.0),
//!             }))
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
//!     let (_, player) = PlayerComponents::first_from_world(&level2).unwrap();
//!     let player_entity = find_player_entity(&level1);
//!     // Move the player back
//!     player.update(player_entity, &mut level1)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Automatically Implementing `ComponentGroup`
//!
//! You can also automatically implement the [`ComponentGroup`] trait using `#[derive(ComponentGroup)]`.
//! This removes all the boilerplate you saw in the example above and automatically provides the
//! methods in [`ComponentGroup`]. All fields in the struct must implement `Clone` so that they can
//! be copied within the methods that get implemented.
//!
//! ```rust
//! // Rust 2018 edition
//! use component_group::ComponentGroup;
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
//! // This is all of the code you need to write to define the group and all of its operations!
//! #[derive(ComponentGroup)]
//! struct PlayerComponents {
//!     position: Position,
//!     velocity: Velocity,
//!     health: Health,
//! }
//!
//! # fn find_player_entity(world: &World) -> specs::Entity {
//! #     use specs::{Entities, Join};
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
//!     let (_, player) = PlayerComponents::first_from_world(&level2).unwrap();
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
//! `update` will remove that component for that entity from the component's storage.
//!
//! ```rust
//! # use component_group::ComponentGroup;
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
//!     // This component is allowed to not be present
//!     animation: Option<Animation>
//! }
//!
//! # fn find_player_entity(world: &World) -> specs::Entity {
//! #     use specs::{Entities, Join};
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
//!         // Since this field is None, the Animation component will not be added when the entity
//!         // is created
//!         animation: None, // Not animated to begin with
//!     };
//!     player.create(&mut level1);
//!
//!     // ...
//!
//!     // Player needs to move on to the next level
//!     let mut level2 = World::new();
//!     # level2.register::<Position>(); level2.register::<Velocity>(); level2.register::<Health>(); level2.register::<Animation>();
//!     // If an Animation component was added between the call to create() and this next call,
//!     // the field will be set to Some(animation_component) where animation_component is the
//!     // instance of the Animation component that was added. Otherwise, the field will be None.
//!     let player_entity = find_player_entity(&level1);
//!     let player = PlayerComponents::from_world(player_entity, &level1);
//!     player.create(&mut level2);
//!
//!     // ...
//!
//!     // Player needs to go back to previous level
//!     // The Animation component may have changed/added/removed, but we don't need to worry
//!     // about that here! The behaviour is the same as above.
//!     let (_, player) = PlayerComponents::first_from_world(&level2).unwrap();
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
//! # use specs::{World, Component, VecStorage, ReadStorage};
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
//! # Generic Component Groups
//!
//! It is possible to use the [`ComponentGroup`] trait and custom derive with generic structs. Just
//! make sure to add `Send + Sync + Component + Clone` trait bounds to the generic type parameters
//! or you will get a compile error. (The `Send + Sync` part is required by the `specs` crate.)
//!
//! ```rust,no_run
//! # use component_group::ComponentGroup;
//! # use specs::{World, Component, VecStorage, ReadStorage};
//! # use specs::error::Error as SpecsError;
//! # use specs_derive::Component;
//! #
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
//! pub struct AngularVelocity {deg: f64}
//!
//! // Don't forget the trait bounds!
//! #[derive(ComponentGroup)]
//! struct PlayerComponents<V: Send + Sync + Component + Clone> {
//!     position: Position,
//!     velocity: V,
//! }
//!
//! // Can use this to provide different component groups that share most of their structure
//! type RunningPlayer = PlayerComponents<Velocity>;
//! type SpinningPlayer = PlayerComponents<AngularVelocity>;
//! #
//! # fn main() {
//! #     // Need to use the type aliases for them to be checked
//! #     let runner: RunningPlayer = unimplemented!();
//! #     let spinner: SpinningPlayer = unimplemented!();
//! # }
//! ```
//!
//! [`ComponentGroup`]: trait.ComponentGroup.html
//! [`specs::Component`]: https://docs.rs/specs/*/specs/trait.Component.html
//! [`specs::World`]: https://docs.rs/specs/*/specs/specs/world/struct.World.html
//! [Generic Associated Types (GATs)]: https://github.com/rust-lang/rust/issues/44265

#![deny(unused_must_use)]

#[doc(hidden)] pub use component_group_derive::*;

use specs::{World, Entity};

/// Represents a group of [`specs::Component`] fields that can be added or extracted from
/// a [`specs::World`].
///
/// See the [top-level crate documentation](index.html) for more details.
///
/// [`specs::Component`]: https://docs.rs/specs/*/specs/trait.Component.html
/// [`specs::World`]: https://docs.rs/specs/*/specs/world/struct.World.html
pub trait ComponentGroup: Sized {
    /// The error type from the [`update` method](#tymethod.update)
    type UpdateError;

    /// Extracts the first instance of this component group from the world.
    ///
    /// This method is convenient if you know that there is exactly one instance of a this group in
    /// the world.
    ///
    /// Returns `None` if any of the required fields could not be populated. Fields with an
    /// `Option` type will be set to `None` if their component could not be populated.
    fn first_from_world(world: &World) -> Option<(Entity, Self)>;
    /// Extracts this group of components for the given entity from the given world.
    ///
    /// Panics if one of the component fields could not be populated. This can happen if the
    /// component does not exist for this entity. If the field is an `Option` type, its value will
    /// be set to `None` instead of panicking.
    fn from_world(entity: Entity, world: &World) -> Self;
    /// Creates a new entity in the world and adds all the components from this group to that entity.
    ///
    /// Any fields with a value of `None` will not be added to the created entity.
    fn create(self, world: &mut World) -> Entity;
    /// Update the components of a given entity with all of the components from this group.
    ///
    /// Any fields with a value of `None` will be explicitly removed from the given entity.
    ///
    /// Note: Any additional components that the entity has other than the ones covered by
    /// the fields of this group will be left untouched.
    fn update(self, entity: Entity, world: &mut World) -> Result<(), Self::UpdateError>;
}
