#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#if defined __linux__ || defined __APPLE__
#include <unistd.h>
#define SLEEP(x) sleep((x))
#define UNUSED __attribute__ ((unused))
#endif
#ifdef _WIN32
#include <windows.h>
#define SLEEP(x) Sleep((x) * 1000)
#define UNUSED
#endif

#include "spotflow.h"

#define DB_PATH "c/c.db"
#define DEFAULT_DEVICE_ID "test-device-c"
#define STREAM_GROUP "device-sdk"
#define STREAM "c"
#define DATA_UNIT "unit"

#define APPROVE_OPERATION_PYTHON_CMD_FORMAT "python -c \"from python.spotflow_cloud import approve_registration; approve_registration('%s')\""

void print_error()
{
    size_t len = SPOTFLOW_ERROR_MAX_LENGTH;
    char* buf = malloc(len);
    spotflow_read_last_error_message(buf, len);
    printf("ERROR: %s\n", buf);
    free(buf);
}

void display_and_approve_provisioning_operation(const spotflow_provisioning_operation_t* operation, UNUSED void* ctx)
{
    printf("Operation ID: %s\n", operation->id);
    printf("Verification Code: %s\n", operation->verification_code);
    printf("Approving the operation...\n");

    // Approve the operation so that this code can run automatically (the device wouldn't have the right to do that in real settings).
    // Use external Python code so that we don't need to reimplement the approval in C.
    char* cmd = malloc(strlen(APPROVE_OPERATION_PYTHON_CMD_FORMAT) + strlen(operation->id) + 1);
    sprintf(cmd, APPROVE_OPERATION_PYTHON_CMD_FORMAT, operation->id);
    system(cmd);
    free(cmd);
}

void send_data(spotflow_client_t* client)
{
    // Initialize settings common for all sent messages
    spotflow_message_context_t* context;
    spotflow_message_context_create(&context, STREAM_GROUP, STREAM);
    spotflow_message_context_set_compression(context, SPOTFLOW_COMPRESSION_FASTEST);

    // Hyphenated string representation of uuid is 36 chars long, plus \0 plus round up
    char* batch_id = calloc(40, 1);
    char* message_id = calloc(40, 1);
    uint8_t* data = calloc(10, 1);
    int i, j, k;

    for (i = 0; i < 2; i++)
    {
        sprintf(batch_id, "%04d", i);
        for (j = 0; j < 10; j++)
        {
            for (k = 0; k < 10; k++)
            {
                data[k] = j;
            }

            sprintf(message_id, "%04d", j);
            // printf("Sending message %s %s\n", batch_id, message_id);
            if (spotflow_client_enqueue_message(client, context, batch_id, message_id, data, 10))
            {
                printf("Error during sending a message\n");
                print_error();
            }
        }
        printf("Completing batch %s\n", batch_id);
        if (spotflow_client_enqueue_batch_completion(client, context, batch_id))
        {
            printf("Error during sending a message\n");
            print_error();
        }
    }

    free(batch_id);
    free(message_id);
    free(data);

    spotflow_message_context_destroy(context);
}

void flush_data(spotflow_client_t* client)
{
    size_t i;
    while (1)
    {
        if (spotflow_client_get_pending_messages_count(client, &i))
        {
            printf("Failed getting the number of pending messages");
            print_error();
            exit(1);
        }
        else
        {
            if (i == 0)
                break;
        }
        printf("Waiting for %lu pending messages...\n", (unsigned long)i);
        SLEEP(5);
    }
}

void print_device_id(spotflow_client_t* client)
{
    size_t len = SPOTFLOW_DEVICE_ID_MAX_LENGTH;
    char* device_id = malloc(len);

    while (1)
    {
        spotflow_result_t result = spotflow_client_get_device_id(client, device_id, len);
        if (result == SPOTFLOW_OK)
        {
            break;
        }
        else if (result == SPOTFLOW_NOT_READY)
        {
            printf("Device ID is not ready.\n");
            SLEEP(1);
            continue;
        }
        else
        {
            printf("Unable to obtain device ID\n");
            exit(1);
        }
    }
    printf("Device ID: %s\n", device_id);
    free(device_id);
}

void print_desired_properties(spotflow_client_t* client)
{
    char* twin;
    uint64_t version;
    twin = malloc(1024);
    if (spotflow_client_get_desired_properties(client, twin, 1024, NULL, &version))
    {
        printf("Unable to retrieve desired properties\n");
        print_error();
        exit(1);
    }
    printf("Desired properties (version %lu):\n%s\n", (unsigned long)version, twin);
    free(twin);
}

void wait_reported_properties_updated(spotflow_client_t* client)
{
    bool any_pending;
    while (1)
    {
        printf("Waiting for the update of the Reported Properties");
        if (spotflow_client_get_any_pending_reported_properties_updates(client, &any_pending))
        {
            printf("Unable to retrieve if there are any pending reported properties updates\n");
            print_error();
            exit(1);
        }
        if (!any_pending)
        {
            break;
        }
        SLEEP(1);
    }
}

int main(int argc, char** argv)
{
    // The program accepts one optional parameter - Device ID
    // (needed to prevent collisions when running simultaneously on different platforms)
    char* device_id = (argc >= 2) ? device_id = argv[1] : DEFAULT_DEVICE_ID;

    spotflow_set_log_level(SPOTFLOW_LOG_DEBUG);
#ifdef _WIN32
    // Windows does not use line-based buffering.
    // Therefore without this line the printf lines are not printed until the end of the program
    setvbuf (stdout, NULL, _IONBF, 0);
#endif

    const char* instance = getenv("SPOTFLOW_DEVICE_SDK_TEST_INSTANCE");
    if (instance == NULL)
    {
        instance = "api.eu1.spotflow.io";
    }

    const char* provisioning_token = getenv("SPOTFLOW_DEVICE_SDK_TEST_PROVISIONING_TOKEN");
    if (provisioning_token == NULL)
    {
        printf("The SPOTFLOW_DEVICE_SDK_TEST_PROVISIONING_TOKEN environment variable must be set to run this example\n");
        return 1;
    }

    spotflow_client_options_t* options;
    spotflow_client_options_create(&options, device_id, provisioning_token, DB_PATH);
    spotflow_client_options_set_instance(options, instance);
    spotflow_client_options_set_display_provisioning_operation_callback(options, display_and_approve_provisioning_operation, NULL);

    spotflow_client_t * client;
    printf("Instantiating client client\n");
    if (spotflow_client_start(&client, options))
    {
        printf("Unable to start the client\n");
        print_error();
        return 1;
    }

    print_device_id(client);

    send_data(client);

    printf("Closing client\n");
    spotflow_client_destroy(client);

    printf("Reopening client\n");
    if (spotflow_client_start(&client, options))
    {
        printf("Unable to resume client\n");
        print_error();
        return 1;
    }

    send_data(client);

    flush_data(client);

    print_desired_properties(client);

    spotflow_client_update_reported_properties(client, "{\"a\": \"a\", \"b\": {\"c\": \"c\"}}");
    wait_reported_properties_updated(client);

    printf("Freeing client\n");
    spotflow_client_destroy(client);

    spotflow_client_options_destroy(options);

    return 0;
}
