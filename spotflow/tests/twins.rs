use std::{path::Path, sync::mpsc, time::Duration};

use spotflow::{DeviceClient, DeviceClientBuilder};

use anyhow::{Context, Result};
use serde_json::json;
use spotflow::{DesiredProperties, DesiredPropertiesUpdatedCallback};
use uuid::Uuid;

#[path = "../examples/common/mod.rs"]
mod common;

struct TestPropertiesUpdatedCallback {
    tx: mpsc::Sender<DesiredProperties>,
}

impl TestPropertiesUpdatedCallback {
    fn new() -> (Self, mpsc::Receiver<DesiredProperties>) {
        let (tx, rx) = mpsc::channel();

        (Self { tx }, rx)
    }
}

impl DesiredPropertiesUpdatedCallback for TestPropertiesUpdatedCallback {
    fn properties_updated(&self, properties: DesiredProperties) -> Result<()> {
        log::info!("Properties updated in callback: {:?}", properties);
        self.tx
            .send(properties)
            .context("Unable to send updated properties")
    }
}

#[allow(deprecated)] // We're keeping all the functions here until the original ones are stabilized or removed
#[test]
fn twins() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("sqlx=warn,ureq=warn,rumqtt=warn,info"),
    )
    .init();

    let env_ctx = common::EnvironmentContext::try_load()
        .expect("Unable to load settings from environment variables.");

    let platform_caller = common::PlatformCaller::try_new(&env_ctx)
        .expect("This test needs to call the Platform automatically and it's unable to do so.");

    let path = Path::new("./test.db");

    let device_id = format!("twin_test_{}", Uuid::new_v4());

    log::info!("Using device ID {}", device_id);

    log::info!("Initiating Device Twin tests");

    common::clear_db(path);

    log::info!("Creating Device Client");

    let (desired_updated_callback, desired_updated_rx) = TestPropertiesUpdatedCallback::new();

    let client =
        DeviceClientBuilder::new(Some(device_id.clone()), env_ctx.provisioning_token, path)
            .with_instance(env_ctx.instance_url.to_string())
            .with_display_provisioning_operation_callback(Box::new(
                common::ProvisioningOperationApprovalHandler::new(Some(platform_caller.clone())),
            ))
            .with_desired_properties_updated_callback(Box::new(desired_updated_callback))
            .build()
            .expect("Unable to build ingress connection");

    log::info!("Obtaining desired properties");
    let desired = client
        .wait_desired_properties_changed()
        .expect("Unable to get desired properties change");
    assert_eq!(String::from("{}"), desired.values);
    let desired = client
        .desired_properties()
        .expect("Unable to read Desired Properties");
    assert_eq!(String::from("{}"), desired.values);

    assert_eq!("{}", desired_updated_rx.recv().unwrap().values);

    // Arrays are currently not supported by DMS
    let new_desired = json!({
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

    log::info!("Updating desired properties");
    platform_caller
        .update_desired_properties(&device_id, &new_desired)
        .expect("Unable to update desired properties");

    log::info!("Awaiting desired properties change");
    let desired = client
        .wait_desired_properties_changed()
        .expect("Unable to get desired properties change");
    let desired_properties = serde_json::from_str::<serde_json::Value>(&desired.values)
        .expect("Unable to deserialize received desired properties");

    log::info!("Properties obtained:\n{:?}", desired);
    assert_eq!(new_desired, desired_properties);
    let spotflow_desired_properties = platform_caller
        .get_desired_properties(&device_id)
        .expect("Unable to get desired properties");
    assert_eq!(desired_properties, spotflow_desired_properties);

    let callback_received_desired_properties = desired_updated_rx.recv().unwrap().values;

    assert_eq!(
        new_desired,
        serde_json::from_str::<serde_json::Value>(&callback_received_desired_properties)
            .expect("Unable to deserialize received desired properties"),
    );

    log::info!("Updating reported properties");

    let reported_patch = r#"{"abc": "def", "lorem": 42, "system": "on"}"#;
    let reported = update_reported(&client, reported_patch, false);

    log::info!("Reported properties:\n{:?}", reported);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(reported_patch)
            .expect("Unable to serialize expected reported properties"),
        serde_json::from_str::<serde_json::Value>(&reported)
            .expect("Unable to serialize received reported properties"),
    );

    let reported_patch = r#"{"abc": "ijk", "foo": "bar", "system": null}"#;
    let expected = r#"{"abc": "ijk", "foo": "bar", "lorem": 42}"#;
    let reported = update_reported(&client, reported_patch, false);

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(expected)
            .expect("Unable to serialize expected reported properties"),
        serde_json::from_str::<serde_json::Value>(&reported)
            .expect("Unable to serialize received reported properties"),
    );

    let new_reported = r#"{"abc": "xyz", "foo": "bar", "xyz": "xxx"}"#;
    let reported = update_reported(&client, new_reported, true);
    let new_reported = serde_json::from_str::<serde_json::Value>(new_reported)
        .expect("Unable to serialize expected reported properties");
    let reported = serde_json::from_str::<serde_json::Value>(&reported)
        .expect("Unable to serialize received reported properties");
    assert_eq!(new_reported, reported);

    let mut spotflow_reported = platform_caller
        .get_reported_properties(&device_id)
        .expect("Unable to get reported properties");
    spotflow_reported
        .as_object_mut()
        .unwrap()
        .remove("$metadata");
    spotflow_reported
        .as_object_mut()
        .unwrap()
        .remove("$version");
    assert_eq!(reported, spotflow_reported);

    log::info!("Terminating connection");
    drop(client);

    log::info!("Deleting the Device");
    platform_caller
        .delete_device(&device_id)
        .expect("Unable to delete the device");
}

#[allow(deprecated)] // We're using all the functions here until they are stabilized or removed
fn update_reported(client: &DeviceClient, patch: &str, full: bool) -> String {
    if full {
        client
            .update_reported_properties(patch)
            .expect("Unable to update reported properties");
    } else {
        client
            .patch_reported_properties(patch)
            .expect("Unable to update reported properties");
    }

    loop {
        let pending = client
            .any_pending_reported_properties_updates()
            .expect("Unable to get the number of pending reported properties updates");
        if !pending {
            break;
        }
        log::info!("Pending reported properties updates: {}", pending);
        std::thread::sleep(Duration::from_secs(1));
    }

    client
        .reported_properties()
        .expect("Reported properties are missing")
}
