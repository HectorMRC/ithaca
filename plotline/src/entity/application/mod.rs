//! The application service.

mod save;
pub use save::*;

mod filter;
pub use filter::*;

mod find;
pub use find::*;

mod remove;
pub use remove::*;

use super::{error::Result, Entity};
use crate::{id::Identifiable, transaction::Tx};
use std::sync::Arc;

pub trait EntityRepository {
    type Tx: Tx<Entity>;

    fn find(&self, id: <Entity as Identifiable>::Id) -> Result<Self::Tx>;
    fn filter(&self, filter: &EntityFilter) -> Result<Vec<Self::Tx>>;
    fn create(&self, entity: &Entity) -> Result<()>;
    fn delete(&self, id: <Entity as Identifiable>::Id) -> Result<()>;
}

pub struct EntityApplication<EntityRepo> {
    pub entity_repo: Arc<EntityRepo>,
}
