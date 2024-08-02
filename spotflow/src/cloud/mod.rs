mod api_core;
pub mod dps;
pub mod drs;
mod duration_wrapper;

fn log_workspace_disabled_error() {
    log::error!(
        "Workspace is disabled, no data can be sent to the Platform now. \
        Enable the Workspace or connect again with a Provisioning Token for an enabled Workspace."
    );
}
