use async_trait::async_trait;
use rumqttc::Publish;

pub(super) mod c2d;
pub(super) mod direct_method;
pub(super) mod twins;

pub(super) trait Handler {
    fn prefix(&self) -> Vec<&str>;
    fn handle(&mut self, publish: &Publish);
}

#[async_trait]
pub(super) trait AsyncHandler {
    fn prefix(&self) -> Vec<&str>;
    async fn handle(&mut self, publish: &Publish);
}
