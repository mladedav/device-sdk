import threading
import time
from spotflow_device import DesiredProperties, DeviceClient
import spotflow_cloud

DEFAULT_SPEED = 10

properties = {
    "speed": DEFAULT_SPEED
}

# Immediately print all Desired Properties changes
def desired_properties_updated_callback(desired_properties: DesiredProperties):
    print(f"Desired Properties changed (version {desired_properties.version}): {desired_properties.values}")

# Connect to the Platform (starts Device Provisioning if the Device is not already registered)
client = DeviceClient.start(
    device_id="config-device-python",
    provisioning_token=spotflow_cloud.get_test_provisioning_token(),
    db="spotflow_config_python.db",
    instance=spotflow_cloud.get_test_instance(),
    display_provisioning_operation_callback=spotflow_cloud.display_and_approve_provisioning_operation,
    desired_properties_updated_callback=desired_properties_updated_callback)

# Emulate sending Desired Properties by the user

def send_desired_properties():
    time.sleep(3)
    spotflow_cloud.update_desired_properties(client.device_id, {"speed": 20})
    time.sleep(3)
    spotflow_cloud.update_desired_properties(client.device_id, {"speed": 30})
    time.sleep(3)
    spotflow_cloud.update_desired_properties(client.device_id, {"speed": 40})

thread = threading.Thread(name='send_desired_properties', target=send_desired_properties, daemon=True)
thread.start()

# Keep configuration in sync

desired_properties_version = None

for _ in range(15):
    desired_properties = client.get_desired_properties_if_newer(desired_properties_version)
    if desired_properties is not None:
        if 'speed' in desired_properties.values:
            properties['speed'] = desired_properties.values['speed']

        desired_properties_version = desired_properties.version
    
        client.update_reported_properties(properties)

    print("Current speed: {}".format(properties['speed']))

    time.sleep(1)
