use super::{EntityFilter, EntityRepository, EntityService};
use crate::entity::{error::Result, Entity};
use std::sync::Arc;

pub struct RemoveEntity<R> {
    entity_repo: Arc<R>,
    filter: EntityFilter,
}

impl<R> RemoveEntity<R>
where
    R: EntityRepository,
{
    pub fn execute(self) -> Result<Arc<Entity>> {
        let entity = self.entity_repo.find(&self.filter)?;
        self.entity_repo.remove(entity.as_ref()).map(|_| entity)
    }
}

impl<R> RemoveEntity<R> {
    pub fn with_filter(mut self, filter: EntityFilter) -> Self {
        self.filter = filter;
        self
    }
}

impl<R> EntityService<R> {
    pub fn remove(&self) -> RemoveEntity<R> {
        RemoveEntity {
            entity_repo: self.entity_repo.clone(),
            filter: Default::default(),
        }
    }
}
