import logging
import spotflow_cloud
from spotflow_device import DeviceClient

FORMAT = '%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s'
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)
logging.getLogger('azure').setLevel(logging.WARN)

# Connect to the Platform (starts Device Provisioning if the Device is not already registered)
client = DeviceClient.start(
    device_id=spotflow_cloud.TEST_DEVICE_ID,
    provisioning_token=spotflow_cloud.get_test_provisioning_token(),
    db="python.db",
    instance=spotflow_cloud.get_test_instance(),
    display_provisioning_operation_callback=spotflow_cloud.display_and_approve_provisioning_operation)
