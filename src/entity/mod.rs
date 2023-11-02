#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "fmt")]
pub mod fmt;
#[cfg(feature = "in_memory")]
pub mod repository;
pub mod service;

mod error;
pub use error::*;

use crate::id::Id;
use crate::name::Name;
use crate::tag::Tags;

/// EntityName determines an instance of [Name] belongs to an [Entity].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityName;

/// EntityId determines an instance of [Id] belongs to an [Entity].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct EntityId;

/// An Entity is anything which to interact with.
#[derive(Clone, Serialize, Deserialize)]
pub struct Entity {
    #[serde(flatten)]
    id: Id<EntityId>,
    #[serde(flatten)]
    name: Name<EntityName>,
    tags: Tags,
}

impl Eq for Entity {}
impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Entity {
    /// Creates a new entity with the given name and an autogenerated id.
    pub fn new(name: Name<EntityName>) -> Self {
        Self {
            id: Id::new(),
            name,
            tags: Default::default(),
        }
    }

    /// Creates a new entity with the given id and name.
    pub fn with_id(id: Id<EntityId>, name: Name<EntityName>) -> Self {
        Self {
            id,
            name,
            tags: Default::default(),
        }
    }

    /// Returns a reference to the name of self.
    pub fn name(&self) -> &Name<EntityName> {
        &self.name
    }
}
