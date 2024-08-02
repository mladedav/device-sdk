use std::path::Path;

use log::warn;
use spotflow::DeviceClientBuilder;

use serde_json::json;
use uuid::Uuid;

mod common;

#[allow(deprecated)] // We're using all the functions here until they are stabilized or removed
fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or("sqlx=warn,ureq=warn,rumqtt=warn,ingress=info,spotflow=trace,info"),
    )
    .init();

    let env_ctx = common::EnvironmentContext::try_load()
        .expect("Unable to load settings from environment variables.");

    let platform_caller = common::PlatformCaller::try_new(&env_ctx)
        .map_err(|e| {
            warn!("Unable to call the Platform automatically, continuing manually. Error: {e}")
        })
        .ok();

    let path = Path::new("./test.db");

    let device_id = format!("twin_example_{}", Uuid::new_v4());

    log::info!("Using device ID {}", device_id);

    log::info!("Initiating Device Twin example");

    log::info!("Creating Device Client");

    let client =
        DeviceClientBuilder::new(Some(device_id.clone()), env_ctx.provisioning_token, path)
            .with_instance(env_ctx.instance_url.to_string())
            .with_display_provisioning_operation_callback(Box::new(
                common::ProvisioningOperationApprovalHandler::new(platform_caller.clone()),
            ))
            .build()
            .expect("Unable to build ingress connection");

    log::info!("Obtaining desired properties");
    let desired = client.desired_properties();
    log::info!("Properties obtained:\n{:?}", desired);

    if let Some(platform_caller) = &platform_caller {
        // Arrays are not supported
        let data = json!({
            "a": r"\",
            "b": "\"",
            "stuff": "here",
            "more-stuff": {
                "number": 4u8,
                "numberer": 1.2f32,
                "obj": {
                    "1": 1u8, "2": 2u8, "2_5": 2.5f32, "3": "three"
                }
            }
        });

        log::info!("Updating Desired Properties");
        platform_caller
            .update_desired_properties(&device_id, &data)
            .expect("Unable to update Desired Properties");
    } else {
        println!("Update the Desired Properties of the Device '{device_id}' manually to continue.");
    }

    log::info!("Awaiting desired properties change");
    let desired = client.wait_desired_properties_changed();
    log::info!("Properties obtained:\n{:?}", desired);

    log::info!("Updating reported properties");

    client
        .patch_reported_properties(r#"{"abc": "def", "lorem": 42}"#)
        .expect("Unable to update reported properties");

    let reported = client.reported_properties();
    log::info!("Reported properties:\n{:?}", reported);

    std::thread::sleep(std::time::Duration::from_secs(10));

    log::info!("Terminating connection");
    drop(client);

    if let Some(platform_caller) = &platform_caller {
        log::info!("Deleting the Device '{device_id}'");
        platform_caller
            .delete_device(&device_id)
            .expect("Unable to delete the device");
    }
}
