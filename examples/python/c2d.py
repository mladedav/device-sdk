from spotflow_device import DeviceClient
import spotflow_cloud

import logging
import threading

FORMAT = '%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s'
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)
logging.getLogger('azure').setLevel(logging.WARN)

device_id = spotflow_cloud.TEST_DEVICE_ID

# Connect to the Platform (starts Device Provisioning if the Device is not already registered)
client = DeviceClient.start(
    device_id=device_id,
    provisioning_token=spotflow_cloud.get_test_provisioning_token(),
    db="python.db",
    instance=spotflow_cloud.get_test_instance(),
    display_provisioning_operation_callback=spotflow_cloud.display_and_approve_provisioning_operation)

def consume_c2d(client):
    consumed_any = False
    while True:
        try:
            msg = client.read_c2d_message(1)
            consumed_any = True
            print(msg.content)
            print(msg.properties)
            print()
            continue
        except Exception as e:
            if consumed_any:
                print(f"Stopped iterating because of {e}")
                break
            else:
                print("No message has been received yet")

thread = threading.Thread(name='c2d', target=consume_c2d, args=(client,), daemon=True)
thread.start()

for i in range(5):
    spotflow_cloud.send_c2d(device_id, str(i))

thread.join()
