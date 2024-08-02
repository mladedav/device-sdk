import time
import logging
import spotflow_cloud
from spotflow_device import DeviceClient, Compression

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

# Sending messages without batches
sender_default = client.create_stream_sender(stream_group = "device-sdk", stream = "default")

print("Messages without batches")
for i in range(10):
    print(f'Msg {i}')
    sender_default.enqueue_message(
        b'test',
    )

# Sending compressed messages
sender_compressed = client.create_stream_sender(stream_group = "device-sdk", stream = "default", compression = Compression.FASTEST)

print("Compressed messages without batches")
for i in range(10, 20):
    print(f'Msg {i}')
    sender_compressed.enqueue_message(
        b'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.',
    )

# Manual completion of batches (also manual message_id)
sender_manual_batch = client.create_stream_sender(stream_group = "device-sdk", stream = "manual-batch")

print("Manual completion of batches")
for i in range(4):
    print(f"Batch {i}")
    batch_id = str(i)
    for j in range(5):
        print(f'Msg {j}')
        sender_manual_batch.enqueue_message(
            b'test',
            batch_id = batch_id,
            message_id = str(j)
        )

sender_manual_batch.enqueue_batch_completion(batch_id)

# Manual completion of batches with slices
sender_manual_batch_slices = client.create_stream_sender(stream_group = "device-sdk", stream = "manual-batch-slices")

print("Messages with batch slices")
sender_manual_batch_slices.enqueue_message(b"Apple1..", batch_id="default_batch", message_id="1", batch_slice_id="apples")
sender_manual_batch_slices.enqueue_message(b"Orange1..", batch_id="default_batch", message_id="2", batch_slice_id="oranges")
sender_manual_batch_slices.enqueue_message(b"..Apple2", batch_id="default_batch", message_id="3", batch_slice_id="apples")
sender_manual_batch_slices.enqueue_message(b"..Orange2", batch_id="default_batch", message_id="4", batch_slice_id="oranges")
sender_manual_batch_slices.enqueue_batch_completion("default_batch")

# Message Chunking
sender_manual = client.create_stream_sender(stream_group = "device-sdk", stream = "manual")

print("Message Chunking")
for i in range(2):
    print(f"Batch {i}")
    batch_id = str(i)

    for j in range(2):
        print(f'Msg {j}')
        message_id = str(j)

        for k, payload in enumerate([b'I ', b' like', b' them', b' chunky']):
            print(f'Chunk {k}')
            chunk_id = str(k)

            sender_manual.enqueue_message(
                payload,
                batch_id = batch_id,
                message_id = message_id,
                chunk_id = chunk_id,
            )
        
        sender_manual.enqueue_message_completion(batch_id = batch_id, message_id = message_id)

# Completing of batches via autofill
sender_autofill = client.create_stream_sender(stream_group = "device-sdk", stream = "autofilled-batch")

print("Autofilled batch")
for i in range(10):
    print(f'Msg {i}')
    sender_autofill.enqueue_message(
        b'test',
    )

while True:
    pending = client.pending_messages_count
    if pending > 0:
        print(f'Pending: {pending}')
        time.sleep(1)
        continue
    print('All messages sent')
    break
