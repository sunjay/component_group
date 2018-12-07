//! See the documentation for the [`ComponentGroup` trait](trait.ComponentGroup.html)

#![deny(unused_must_use)]

use specs::{World, Entity};

#[derive(Debug, Clone)]
pub enum FromWorldError {
    /// Did not find a group when a group was expected
    NoGroups,
    /// Found more groups than expected
    TooManyGroups,
}

/// Represents a group of [`specs::Component`] fields that can be added or extracted from
/// a [`specs::World`].
///
/// # Motivation
///
/// The purpose of this trait is to make moving a group of components between
/// worlds very easy and less error-prone. Instead of accidentally forgetting a component somewhere
/// in your code, you can group them all together and use the methods in this trait to move them
/// around.
///
/// ```rust
/// // Rust 2018 edition
/// use component_group::{ComponentGroup, FromWorldError};
/// use specs::{World, Entity, Component, VecStorage, ReadStorage, WriteStorage, Join};
/// use specs::error::Error as SpecsError;
/// use specs_derive::Component;
///
/// // If you manually implement the trait, you don't need to make the fields Clone.
/// // This adds a lot of boilerplate. See below for a boilerplate-free method that uses Clone.
/// #[derive(Debug, Component)]
/// #[storage(VecStorage)]
/// pub struct Position {x: i32, y: i32}
///
/// #[derive(Debug, Component)]
/// #[storage(VecStorage)]
/// pub struct Velocity {x: i32, y: i32}
///
/// #[derive(Debug, Component)]
/// #[storage(VecStorage)]
/// pub struct Health(u32);
///
/// pub struct PlayerComponents {
///     position: Position,
///     velocity: Velocity,
///     health: Health,
/// }
///
/// pub struct PlayerComponentsIter<'a> {
///     // Good luck constructing this type...
///     inner: specs::join::JoinIter<(&'a specs::storage::Storage<'a, Position, shred::Fetch<'a, specs::storage::MaskedStorage<Position>>>, &'a specs::storage::Storage<'a, Velocity, shred::Fetch<'a, specs::storage::MaskedStorage<Velocity>>>, &'a specs::storage::Storage<'a, Health, shred::Fetch<'a, specs::storage::MaskedStorage<Health>>>)>,
/// }
///
/// impl ComponentGroup for PlayerComponents {
///     type GroupIter = PlayerComponentsIter<'a>;
///     type UpdateError = SpecsError;
///
///     fn all_from_world(world: &World) -> Self::GroupIter {
///         // Needs to be updated every time the struct changes
///         let (position, velocity, health) = world.system_data::<(
///             ReadStorage<Position>,
///             ReadStorage<Velocity>,
///             ReadStorage<Health>,
///         )>();
///         let x: () = (&position, &velocity, &health).join()
///             .map(|position, velocity, health| Self {position, velocity, health});
///     }
///
///     fn create(self, world: &mut World) -> Entity {
///         // It's possible to write this code so that the compiler will at least warn you if
///         // you forget one of the things you need to change when the struct changes.
///
///         // Using pattern matching here forces a compiler error whenever the struct changes
///         let Self {position, velocity, health} = self;
///
///         // Forgetting to add a .with() call will cause an unused variable warning
///         world.create_entity()
///             .with(position)
///             .with(velocity)
///             .with(health)
///             .build()
///     }
///
///     fn update(self, entity: Entity, world: &mut World) -> Result<(), Self::UpdateError> {
///         // Needs to be updated every time the struct changes
///         let (position, velocity, health) = world.system_data::<(
///             WriteStorage<Position>,
///             WriteStorage<Velocity>,
///             WriteStorage<Health>,
///         )>();
///
///         position.insert(entity, self.position)?;
///         velocity.insert(entity, self.velocity)?;
///         health.insert(entity, self.health)?;
///         Ok(())
///     }
/// }
///
/// # use specs::Entities;
/// # fn find_player_entity(world: &World) -> Entity {
/// #     world.system_data::<Entities>().join().next().unwrap() // cheat since only one entity
/// # }
/// #
/// # #[derive(Debug)] struct AppError {s: String}
/// # impl From<component_group::FromWorldError> for AppError { fn from(x: component_group::FromWorldError) -> Self { Self {s: format!("{:?}", x) }} }
/// # impl From<specs::error::Error> for AppError { fn from(x: specs::error::Error) -> Self { Self {s: format!("{:?}", x) }} }
/// fn main() -> Result<(), AppError> {
///     let mut level1 = World::new();
///     // Having all the components together in a struct means that Rust will enforce that you
///     // never forget a field.
///     let player = PlayerComponents {
///         position: Position {x: 12, y: 59},
///         velocity: Velocity {x: -1, y: 2},
///         health: Health(5),
///     };
///     // Add the player to the level
///     player.create(&mut level1);
///
///     // ...
///
///     // Player needs to move on to the next level
///     let mut level2 = World::new();
///     // Extract the player from the world it was just in
///     // from_world is a default method on the ComponentGroup trait. You can use all_from_world
///     // to extract all instances
///     let player = PlayerComponents::from_world(&level1)?;
///     // Add it to the next world since it hasn't been added yet
///     player.create(level2);
///
///     // ...
///
///     // Player needs to go back to previous level
///     let player = PlayerComponents::from_world(&level2)?;
///     // Somehow find the player in the world it was just in
///     let player_entity = find_player_entity(&level1);
///     // Move the player back
///     player.update(player_entity, &mut level1)?;
///
///     Ok(())
/// }
/// ```
///
/// # Custom Derive
///
/// The `component_group_derive` crate makes this even easier by removing all of the boilerplate
/// involved in implementing this trait. That derive requires that all fields be Clone so that they
/// can be copied within the methods.
///
/// ```rust
/// // Rust 2018 edition
/// use component_group::{ComponentGroup, FromWorldError};
/// use component_group_derive::ComponentGroup;
/// use specs::{World, Component, VecStorage};
/// use specs_derive::Component;
///
/// // Note that components need to be Clone to use the automatic derive
/// #[derive(Debug, Clone, Component)]
/// #[storage(VecStorage)]
/// pub struct Position {x: i32, y: i32}
///
/// #[derive(Debug, Clone, Component)]
/// #[storage(VecStorage)]
/// pub struct Velocity {x: i32, y: i32}
///
/// #[derive(Debug, Clone, Component)]
/// #[storage(VecStorage)]
/// pub struct Health(u32);
///
/// #[derive(ComponentGroup)]
/// struct PlayerComponents {
///     position: Position,
///     velocity: Velocity,
///     health: Health,
/// }
///
/// # use specs::{Entity, Entities, Join};
/// # fn find_player_entity(world: &World) -> Entity {
/// #     world.system_data::<Entities>().join().next().unwrap() // cheat since only one entity
/// # }
/// #
/// # #[derive(Debug)] struct AppError {s: String}
/// # impl From<component_group::FromWorldError> for AppError { fn from(x: component_group::FromWorldError) -> Self { Self {s: format!("{:?}", x) }} }
/// # impl From<specs::error::Error> for AppError { fn from(x: specs::error::Error) -> Self { Self {s: format!("{:?}", x) }} }
/// fn main() -> Result<(), AppError> {
///     let mut level1 = World::new();
///     // Having all the components together in a struct means that Rust will enforce that you
///     // never forget a field.
///     let player = PlayerComponents {
///         position: Position {x: 12, y: 59},
///         velocity: Velocity {x: -1, y: 2},
///         health: Health(5),
///     };
///     // Add the player to the level
///     player.create(&mut level1);
///
///     // ...
///
///     // Player needs to move on to the next level
///     let mut level2 = World::new();
///     // Extract the player from the world it was just in
///     // from_world is a default method on the ComponentGroup trait. You can use all_from_world
///     // to extract all instances
///     let player = PlayerComponents::from_world(&level1)?;
///     // Add it to the next world since it hasn't been added yet
///     player.create(level2);
///
///     // ...
///
///     // Player needs to go back to previous level
///     let player = PlayerComponents::from_world(&level2)?;
///     // Somehow find the player in the world it was just in
///     let player_entity = find_player_entity(&level1);
///     // Move the player back
///     player.update(player_entity, &mut level1)?;
///
///     Ok(())
/// }
/// ```
///
/// # Optional Fields
///
/// You can also use `Option` to ignore part of the group if it isn't specified during creation or
/// if it isn't available in the `World` during extraction.
///
/// ```rust
/// # use component_group::{ComponentGroup, FromWorldError};
/// # use component_group_derive::ComponentGroup;
/// # use specs::{World, Component, VecStorage, HashMapStorage};
/// # use specs_derive::Component;
/// // Same components as before
/// # #[derive(Debug, Clone, Component)]
/// # #[storage(VecStorage)]
/// # pub struct Position {x: i32, y: i32}
/// #
/// # #[derive(Debug, Clone, Component)]
/// # #[storage(VecStorage)]
/// # pub struct Velocity {x: i32, y: i32}
/// #
/// # #[derive(Debug, Clone, Component)]
/// # #[storage(VecStorage)]
/// # pub struct Health(u32);
///
/// #[derive(Debug, Clone, Component)]
/// #[storage(HashMapStorage)]
/// pub struct Animation {frame: usize}
///
/// #[derive(ComponentGroup)]
/// struct PlayerComponents {
///     position: Position,
///     velocity: Velocity,
///     health: Health,
///     // Allowed to not be present
///     animation: Option<Animation>
/// }
///
/// # use specs::{Entity, Entities, Join};
/// # fn find_player_entity(world: &World) -> Entity {
/// #     world.system_data::<Entities>().join().next().unwrap() // cheat since only one entity
/// # }
/// #
/// # #[derive(Debug)] struct AppError {s: String}
/// # impl From<component_group::FromWorldError> for AppError { fn from(x: component_group::FromWorldError) -> Self { Self {s: format!("{:?}", x) }} }
/// # impl From<specs::error::Error> for AppError { fn from(x: specs::error::Error) -> Self { Self {s: format!("{:?}", x) }} }
/// fn main() -> Result<(), AppError> {
///     let level1 = World::new();
///     let player = PlayerComponents {
///         position: Position {x: 12, y: 59},
///         velocity: Velocity {x: -1, y: 2},
///         health: Health(5),
///         animation: None, // Not animated to begin with
///     };
///     player.create(&mut level1);
///
///     // ...
///
///     // Player needs to move on to the next level
///     let mut level2 = World::new();
///     // This code works regardless of whether an animation has been added or not.
///     // No need to check either!
///     let player = PlayerComponents::from_world(&level1)?;
///     player.create(level2);
///
///     // ...
///
///     // Player needs to go back to previous level
///     // The Animation component may have changed/added/removed, but we don't need to worry
///     // about that here!
///     let player = PlayerComponents::from_world(&level2)?;
///     let player_entity = find_player_entity(&level1);
///     player.update(player_entity, &mut level1)?;
///
///     Ok(())
/// }
/// ```
///
/// [`specs::Component`]: ../specs/trait.Component.html
/// [`specs::World`]: ../specs/world/struct.World.html
pub trait ComponentGroup: Sized {
    /// An iterator over instances of this group
    type GroupIter: Iterator<Item=Self>;
    type UpdateError;

    /// Extracts all instances of this group of components from the world.
    fn all_from_world(world: &World) -> Self::GroupIter;
    /// Create a new entity in the world and add all the components from this group.
    fn create(self, world: &mut World) -> Entity;
    /// Update the components of a given entity with all of the components from this group.
    ///
    /// Note: Any additional components that the entity has other than the ones covered by
    /// the fields of this group will be left untouched.
    fn update(self, entity: Entity, world: &mut World) -> Result<(), Self::UpdateError>;

    /// Extracts one instance of the component group from the world.
    fn from_world(world: &World) -> Result<Self, FromWorldError> {
        let mut group_iter = Self::all_from_world(world);
        let group = group_iter.next().ok_or(FromWorldError::NoGroups)?;
        match group_iter.next() {
            Some(_) => return Err(FromWorldError::TooManyGroups),
            None => {},
        }
        Ok(group)
    }
}
