use anyhow::Result;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::task::JoinHandle;

pub mod twins;

pub type JoinHandleVec = Vec<JoinHandle<()>>;

pub trait ConnectionImplementation: Send + Sync {
    // We are not using async_trait because we don't want the resulting future be dependant on the lifetime of &mut self.
    // This method returns a vector of tokio tasks that need to be run for the connection to work
    fn connect(&mut self) -> Pin<Box<dyn Future<Output = Result<JoinHandleVec>> + Send>>;
    fn error(&mut self) -> Option<Arc<dyn std::error::Error>>;
}
