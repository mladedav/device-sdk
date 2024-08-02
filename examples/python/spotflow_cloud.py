from azure.identity import DefaultAzureCredential
import os
import requests

TEST_DEVICE_ID = "test-device-python"

def get_test_instance():
    return os.getenv('SPOTFLOW_DEVICE_SDK_TEST_INSTANCE', 'api.eu1.spotflow.io').removeprefix("https://")

def get_test_provisioning_token():
    provisioning_token = os.getenv('SPOTFLOW_DEVICE_SDK_TEST_PROVISIONING_TOKEN')
    if provisioning_token is None:
        raise Exception("Environment variable SPOTFLOW_DEVICE_SDK_TEST_PROVISIONING_TOKEN not set")
    return provisioning_token

def _get_test_workspace_id():
    workspace_id = os.getenv('SPOTFLOW_DEVICE_SDK_TEST_WORKSPACE_ID')
    if workspace_id is None:
        raise Exception("Environment variable SPOTFLOW_DEVICE_SDK_TEST_WORKSPACE_ID not set")
    return workspace_id

# Custom callback for waiting for device approval ()
def display_and_approve_provisioning_operation(operation):
    print(f'Operation ID: {operation.id}')
    print(f'Verification Code: {operation.verification_code}')

    # Approve the operation so that this code can run automatically (the device wouldn't have the right to do that in real settings)
    print('Approving the operation...')
    try:
        approve_registration(operation.id)
    except Exception as e:
        print(f'Error approving the operation automatically: {e}')
        print('Please approve the operation manually to continue')

def _get_access_token(service):
    auth0_token = os.getenv('SPOTFLOW_DEVICE_SDK_TEST_API_TOKEN')
    if auth0_token:
        return auth0_token
    else:
        instance = get_test_instance()
        credential = DefaultAzureCredential()
        return credential.get_token(f'https://{service}.{instance}/.default').token

def send_c2d(device_id, msg):
    token = _get_access_token('device-management')
    headers = {'Authorization': f'Bearer {token}'}
    instance = get_test_instance()
    workspace_id = _get_test_workspace_id()
    requests.post(f'https://{instance}/workspaces/{workspace_id}/devices/{device_id}/c2d-messages', data=msg, headers=headers).raise_for_status()

def update_desired_properties(device_id, twin):
    token = _get_access_token('device-management')
    headers = {'Authorization': f'Bearer {token}'}
    instance = get_test_instance()
    workspace_id = _get_test_workspace_id()
    requests.patch(f'https://{instance}/workspaces/{workspace_id}/devices/{device_id}/desired-properties', json=twin, headers=headers).raise_for_status()

def approve_registration(operation_id):
    token = _get_access_token('device-provisioning')
    headers = {'Authorization': f'Bearer {token}'}
    instance = get_test_instance()
    workspace_id = _get_test_workspace_id()
    requests.put(f'https://{instance}/workspaces/{workspace_id}/provisioning-operations/approve', json={"provisioningOperationId": operation_id}, headers=headers).raise_for_status()
