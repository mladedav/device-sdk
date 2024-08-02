use spotflow::{DeviceClientBuilder, MessageContext};
use std::{
    fs::{self, File},
    path::Path,
};

use log::{debug, info, warn};
use uuid::Uuid;

mod common;

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or("sqlx=warn,ureq=warn,rumqtt=warn,spotflow=debug,info"),
    )
    .init();

    let env_ctx = common::EnvironmentContext::try_load()
        .expect("Unable to load settings from environment variables.");

    let platform_caller = common::PlatformCaller::try_new(&env_ctx)
        .map_err(|e| {
            warn!("Unable to call the Platform automatically, continuing manually. Error: {e}")
        })
        .ok();

    let device_id = Uuid::new_v4().to_string();
    let stream_group = "device-sdk";
    let stream = "rust";
    let site = "test-site";

    let batch_count = 10u32;
    let message_count = 100u32;

    let path = Path::new("./test.db");

    info!("Using device ID {}", device_id);

    info!("Initiating connection tests");

    if path.exists() {
        info!("Removing the old local database file");
        fs::remove_file(path).expect("Unable to delete the old local database file");
    }
    info!("Creating a new local database file");
    File::create(path).expect("Unable to create a new local database file");

    {
        info!("Creating ingress client");

        let client = DeviceClientBuilder::new(
            Some(device_id.clone()),
            env_ctx.provisioning_token.clone(),
            path,
        )
        .with_instance(env_ctx.instance_url.to_string())
        .with_site_id(site.to_owned())
        .with_display_provisioning_operation_callback(Box::new(
            common::ProvisioningOperationApprovalHandler::new(platform_caller.clone()),
        ))
        .build()
        .expect("Unable to build ingress connection");

        let message_context =
            MessageContext::new(Some(stream_group.to_owned()), Some(stream.to_owned()));

        for batch_id in 0..batch_count {
            let batch_id = format!("{batch_id:0>2}");

            for message_id in 0..message_count {
                let message_id = format!("{message_id:0>2}");

                debug!("Publishing message {message_id}");
                client
                    .enqueue_message(
                        &message_context,
                        Some(batch_id.clone()),
                        Some(message_id),
                        vec![b'a'; 1000],
                    )
                    .expect("Unable to send message");
            }

            info!("Completing batch {batch_id}");
            client
                .enqueue_batch_completion(&message_context, batch_id)
                .expect("Unable to complete batch");
        }

        info!("Dropping original ingress");
    }

    {
        info!("Starting new ingress client with configuration");

        let client = DeviceClientBuilder::new(
            Some(device_id.clone()),
            env_ctx.provisioning_token.clone(),
            path,
        )
        .with_instance(env_ctx.instance_url.to_string())
        .with_display_provisioning_operation_callback(Box::new(
            common::ProvisioningOperationApprovalHandler::new(platform_caller.clone()),
        ))
        .build()
        .expect("Unable to build ingress connection");

        loop {
            let pending = client
                .pending_messages_count()
                .expect("Unable to obtain number of pending messages");
            if pending < 200 {
                break;
            }
            warn!("Waiting for {} more messages to be sent.", pending);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        info!("Dropping ingress");
    }

    {
        info!("Starting new ingress client");

        let client =
            DeviceClientBuilder::new(Some(device_id.clone()), env_ctx.provisioning_token, path)
                .with_instance(env_ctx.instance_url.to_string())
                .with_display_provisioning_operation_callback(Box::new(
                    common::ProvisioningOperationApprovalHandler::new(platform_caller),
                ))
                .build()
                .expect("Unable to build ingress connection");

        loop {
            let pending = client
                .pending_messages_count()
                .expect("Unable to obtain number of pending messages");
            if pending == 0 {
                break;
            }
            warn!("Waiting for {} more messages to be sent.", pending);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        info!("Dropping ingress");
    }
}
