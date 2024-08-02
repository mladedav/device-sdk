use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use http::Uri;
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;

use super::api_core::{put, RequestError};
use super::dps::RegistrationToken;
use super::duration_wrapper::DurationWrapper;
use super::log_workspace_disabled_error;

#[derive(Debug, Error)]
pub enum RegistrationError {
    #[error("Registration Token is invalid")]
    InvalidRegistrationToken,
    #[error("Workspace is disabled")]
    WorkspaceDisabled,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub enum ConnectionStringType {
    SharedAccessKey,
    SharedAccessSignature,
    AuthorizationHeader,
}

/// Only the used parts are deserialized.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationResponse {
    pub(self) connection_string: String,
    pub iot_hub_host_name: String,
    pub connection_string_type: ConnectionStringType,
    pub connection_string_expiration: Option<DateTime<Utc>>,
    pub token_remaining_lifetime: Option<DurationWrapper>,
}

impl RegistrationResponse {
    pub fn iot_hub_device_id(&self) -> Result<&str> {
        if self.connection_string_type != ConnectionStringType::SharedAccessSignature {
            bail!("Cannot parse anything but Shared Access Signature.");
        }

        let device_id = self
            .connection_string
            .split(';')
            .find_map(|part| {
                if let Some((k, v)) = part.split_once('=') {
                    if k == "DeviceId" {
                        return Some(v);
                    }
                }
                None
            })
            .unwrap();

        Ok(device_id)
    }

    pub fn workspace_id(&self) -> Result<&str> {
        let iot_hub_device_id = self.iot_hub_device_id()?;

        let separator_pos = Self::get_iot_hub_device_id_separator_pos(iot_hub_device_id)?;

        Ok(&iot_hub_device_id[..separator_pos])
    }

    pub fn device_id(&self) -> Result<&str> {
        let iot_hub_device_id = self.iot_hub_device_id()?;

        let separator_pos = Self::get_iot_hub_device_id_separator_pos(iot_hub_device_id)?;

        Ok(&iot_hub_device_id[separator_pos + 1..])
    }

    fn get_iot_hub_device_id_separator_pos(
        iot_hub_device_id: &str,
    ) -> Result<usize, anyhow::Error> {
        iot_hub_device_id.find(':').ok_or_else(|| {
            anyhow::anyhow!("Unknown format of IoT Hub Device ID, it does not contain a colon: '{iot_hub_device_id}'.")
        })
    }

    pub fn sas(&self) -> Result<&str> {
        if self.connection_string_type != ConnectionStringType::SharedAccessSignature {
            bail!("Cannot parse anything but Shared Access Signature.");
        }

        let sas = self
            .connection_string
            .split(';')
            .find_map(|part| {
                if let Some((k, v)) = part.split_once('=') {
                    if k == "SharedAccessSignature" {
                        return Some(v);
                    }
                }
                None
            })
            .unwrap();

        Ok(sas)
    }
}

pub fn register(
    instance_url: &Uri,
    rt: &RegistrationToken,
) -> Result<RegistrationResponse, RegistrationError> {
    let relative_url = Uri::from_static("/devices/register");
    let data = json!({
        "connectionStringType": "SharedAccessSignature",
    });

    put(instance_url, &relative_url, &rt.token, data)
        .map_err(|e| match e {
            RequestError::Status(401, _) => RegistrationError::InvalidRegistrationToken,
            RequestError::Status(423, _) => {
                log_workspace_disabled_error();
                RegistrationError::WorkspaceDisabled
            }
            _ => RegistrationError::Other(e.into()),
        })?
        .into_json()
        .context("Failed deserializing response from JSON")
        .map_err(|e| e.into())
}
