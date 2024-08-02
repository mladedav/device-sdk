from spotflow_device import DeviceClient
import spotflow_cloud

import time
import logging

FORMAT = '%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s'
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.INFO)
logging.getLogger('azure').setLevel(logging.WARN)

# Connect to the Platform (starts Device Provisioning if the Device is not already registered)
client = DeviceClient.start(
    device_id="twins-device-python",
    provisioning_token=spotflow_cloud.get_test_provisioning_token(),
    db="python.db",
    instance=spotflow_cloud.get_test_instance(),
    display_provisioning_operation_callback=spotflow_cloud.display_and_approve_provisioning_operation)

desired = client.get_desired_properties()

print(f"Desired properties (version {desired.version}): {desired.values}")

properties = {
    'abc': "xyz",
    'lorem': [
        'lorem',
        'ipsum',
        'dolor',
        'sit',
        'amet',
    ],
    'question': {
        'answer': 42
    }
}
client.update_reported_properties(properties)
while True:
    if not client.any_pending_reported_properties_updates:
        break
    time.sleep(1)

for i in range(4):
    print("Changing desired properties")
    spotflow_cloud.update_desired_properties(client.device_id, {"i": str(i)})

    print("Waiting for desired properties change")
    while True:
        new_desired = client.get_desired_properties_if_newer(desired.version)
        if new_desired is not None:
            desired = new_desired
            break
        time.sleep(1)
    print("Desired properties changed")

    desired = client.get_desired_properties()
    print(f"Desired properties (version {desired.version}): {desired.values}")

    properties["i"] = desired.values["i"]
    client.update_reported_properties(properties)
