use std::{ops::Deref, sync::Arc};

use crate::persistence::sqlite_channel;
use tokio::{runtime::Handle, sync::Mutex};

pub use crate::persistence::CloudToDeviceMessage;

pub struct CloudToDeviceMessageGuard<'a> {
    msg: CloudToDeviceMessage,
    runtime: &'a Handle,
    consumer: Arc<Mutex<sqlite_channel::Receiver<CloudToDeviceMessage>>>,
}

impl<'a> CloudToDeviceMessageGuard<'a> {
    pub(super) fn new(
        msg: CloudToDeviceMessage,
        runtime: &'a Handle,
        consumer: Arc<Mutex<sqlite_channel::Receiver<CloudToDeviceMessage>>>,
    ) -> Self {
        CloudToDeviceMessageGuard {
            msg,
            runtime,
            consumer,
        }
    }
}

impl<'a> Deref for CloudToDeviceMessageGuard<'a> {
    type Target = CloudToDeviceMessage;

    fn deref(&self) -> &Self::Target {
        &self.msg
    }
}

impl<'a> Drop for CloudToDeviceMessageGuard<'a> {
    fn drop(&mut self) {
        let ack_result = self
            .runtime
            .block_on(async { self.consumer.lock().await.ack(&self.msg).await });

        if let Err(e) = ack_result {
            log::warn!(
                "Unable to remove message to prevent further processing: {:?}",
                e
            );
        }
    }
}
