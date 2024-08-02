#![allow(dead_code)]

use std::{fs, path::Path};

use anyhow::{anyhow, Result};
use http::Uri;
use log::{error, info};
pub use platform_caller::PlatformCaller;
use spotflow::{ProvisioningOperation, ProvisioningOperationDisplayHandler};

mod platform_caller;

const INSTANCE_ENV_VAR: &str = "SPOTFLOW_DEVICE_SDK_TEST_INSTANCE";
const PROVISIONING_TOKEN_ENV_VAR: &str = "SPOTFLOW_DEVICE_SDK_TEST_PROVISIONING_TOKEN";
const WORKSPACE_ID_ENV_VAR: &str = "SPOTFLOW_DEVICE_SDK_TEST_WORKSPACE_ID";
const API_TOKEN_ENV_VAR: &str = "SPOTFLOW_DEVICE_SDK_TEST_API_TOKEN";

const DEFAULT_INSTANCE: &str = "https://api.eu1.spotflow.io";

pub struct EnvironmentContext {
    pub instance_url: Uri,
    pub provisioning_token: String,
    pub workspace_id: Option<String>,
    pub api_token: Option<String>,
}

impl EnvironmentContext {
    pub fn try_load() -> Result<Self> {
        let instance = std::env::var(INSTANCE_ENV_VAR)
            .ok()
            .unwrap_or_else(|| String::from(DEFAULT_INSTANCE));

        let instance_url = if instance.starts_with("https://") {
            instance
        } else {
            format!("https://{instance}")
        };

        let instance_url = instance_url
            .parse()
            .map_err(|_| anyhow!("Cannot parse the instance URL '{instance_url}'."))?;

        let provisioning_token = std::env::var(PROVISIONING_TOKEN_ENV_VAR)
            .map_err(|_| anyhow!("Environment variable '{PROVISIONING_TOKEN_ENV_VAR}' is not set. Please set it to a valid Provisioning Token."))?;

        let workspace_id = std::env::var(WORKSPACE_ID_ENV_VAR).ok();

        let api_token = std::env::var(API_TOKEN_ENV_VAR).ok();

        Ok(Self {
            instance_url,
            provisioning_token,
            workspace_id,
            api_token,
        })
    }
}

pub struct ProvisioningOperationApprovalHandler {
    platform_caller: Option<PlatformCaller>,
}

impl ProvisioningOperationApprovalHandler {
    pub fn new(platform_caller: Option<PlatformCaller>) -> Self {
        Self { platform_caller }
    }
}

impl ProvisioningOperationDisplayHandler for ProvisioningOperationApprovalHandler {
    fn display(&self, provisioning_operation: &ProvisioningOperation) -> Result<(), anyhow::Error> {
        let approved = if let Some(platform_caller) = &self.platform_caller {
            info!(
                "Approving the Provisioning Operation {}",
                provisioning_operation.id
            );

            let approval_result =
                platform_caller.approve_provisioning_operation(&provisioning_operation.id);
            match approval_result {
                Ok(_) => {
                    info!(
                        "Provisioning Operation {} approved successfully",
                        provisioning_operation.id
                    );
                    true
                }
                Err(e) => {
                    error!("Error while trying to approve the Provisioning Operation automatically: {e}");
                    false
                }
            }
        } else {
            false
        };

        if !approved {
            println!("Approve this Provisioning Operation to allow the program to continue:");
            println!("Operation ID: {}", provisioning_operation.id);
            println!(
                "Verification Code: {}",
                provisioning_operation.verification_code
            );
        }

        Ok(())
    }
}

pub fn clear_db(path: &Path) {
    if path.exists() {
        log::info!("Removing old local database file");
        fs::remove_file(path).expect("Unable to delete old local database file");
    }
}
