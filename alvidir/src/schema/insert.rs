//! Insertion transaction.

use std::cell::RefCell;

use crate::{
    chain::LiFoChain,
    command::{Command, NoopCommand},
    graph::Graph,
    id::Identify,
};

use super::{wrapper::Wrapper, Schema};

/// The context for the before-insertion triggers.
pub struct NodeToInsert<'a, T>
where
    T: Identify,
{
    /// The graph in which the node is being inserted.
    pub graph: &'a Graph<T>,
    /// The node being inserted into the schema.
    pub node: RefCell<T>,
}

/// The context of the after-insertion triggers.
pub struct InsertedNode<'a, T>
where
    T: Identify,
{
    /// The schema in which the node has been inserted.
    pub schema: &'a Schema<T>,
    /// The id of the inserted node.
    pub node: T::Id,
}

/// An insertion transaction for a node into a schema.
pub struct Insert<T, B, A>
where
    T: Identify,
{
    /// The node being inserted into the schema.
    pub node: T,
    /// The command to execute before inserting the node.
    ///
    /// If this command fails the whole transaction is aborted.
    pub before: B,
    /// The command to execute once the insertion has been performed.
    ///
    /// If this command fails the transaction IS NOT rollbacked. But the resulting error is retrived as the transaction's result.
    pub after: A,
}

impl<T, B, A, E> Command<Schema<T>> for Insert<T, B, A>
where
    T: 'static + Identify,
    T::Id: Ord + Clone,
    B: for<'b> Command<NodeToInsert<'b, T>, Err = E>,
    A: for<'a> Command<InsertedNode<'a, T>, Err = E>,
{
    type Err = E;

    fn execute(self, schema: &Schema<T>) -> Result<(), Self::Err> {
        let inserted_id = {
            let mut graph = match schema.graph.write() {
                Ok(graph) => graph,
                Err(poisoned) => {
                    tracing::error!("posioned graph has been recovered");
                    poisoned.into_inner()
                }
            };

            let final_node = {
                let payload = NodeToInsert {
                    graph: &graph,
                    node: RefCell::new(self.node),
                };

                self.before.execute(&payload)?;
                payload.node
            }
            .into_inner();

            let inserted_id = final_node.id().clone();
            graph.insert(final_node);

            inserted_id
        };

        let payload = InsertedNode {
            schema,
            node: inserted_id,
        };

        self.after.execute(&payload)
    }
}

impl<T> Insert<T, NoopCommand, NoopCommand>
where
    T: Identify,
{
    pub fn new(node: T) -> Self {
        Self {
            node,
            before: NoopCommand,
            after: NoopCommand,
        }
    }

    /// Configure triggers for this transaction.
    pub fn with_trigger(self) -> Wrapper<Self> {
        self.into()
    }
}

impl<T, B, A> Wrapper<Insert<T, B, A>>
where
    T: Identify,
{
    /// Configures the given command as a before insertion trigger.
    pub fn before<C>(self, command: C) -> Insert<T, LiFoChain<C, B>, A> {
        Insert {
            node: self.inner.node,
            before: LiFoChain {
                head: self.inner.before,
                value: command,
            },
            after: self.inner.after,
        }
    }
}

impl<T, B, A> Wrapper<Insert<T, B, A>>
where
    T: Identify,
{
    /// Configures the given command as an after insertion trigger.
    pub fn after<C>(self, command: C) -> Insert<T, B, LiFoChain<C, A>> {
        Insert {
            node: self.inner.node,
            before: self.inner.before,
            after: LiFoChain {
                head: self.inner.after,
                value: command,
            },
        }
    }
}
