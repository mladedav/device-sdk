#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#include "spotflow.h"

#ifdef _WIN32
#include <windows.h>
#define gmtime_r(timer, buf) gmtime_s(buf, timer)
#else
#include <unistd.h>
#define Sleep(x) usleep((x)*1000)
#endif

void show_last_error()
{
    size_t error_size = SPOTFLOW_ERROR_MAX_LENGTH;
    char* error_buffer = malloc(error_size);
    spotflow_read_last_error_message(error_buffer, error_size);
    printf("Error: %s\n", error_buffer);
    free(error_buffer);
}

void send_data(spotflow_client_t* client)
{
    spotflow_message_context_t* ctx;
    spotflow_message_context_create(&ctx, "default-stream-group", "default-stream");

    const size_t max_size = 1024;
    char* msg = malloc(max_size);

    for (int i = 0; i < 60; i++)
    {
        time_t now_ts = time(NULL);
        struct tm now;
        gmtime_r(&now_ts, &now);

        double temperature = 21 + (i * 0.05);
        double humidity = 50 + (i * 0.1);

        snprintf(
            msg, max_size,
            "{\"timestamp\": \"%04d-%02d-%02dT%02d:%02d:%02dZ\", \"temperatureCelsius\": %g, \"humidityPercent\": %g}",
            now.tm_year + 1900, now.tm_mon + 1, now.tm_mday, now.tm_hour, now.tm_min, now.tm_sec, temperature, humidity);

        printf("%s\n", msg);

        if (spotflow_client_send_message(client, ctx, NULL, NULL, (const uint8_t*)msg, strlen(msg)) != SPOTFLOW_OK)
        {
            show_last_error();
            return;
        }

        Sleep(5000);
    }

    free(msg);

    spotflow_message_context_destroy(ctx);
}

int main()
{
    spotflow_client_options_t* options;
    spotflow_client_options_create(&options, "my-device", "<Your Provisioning Token>", "spotflow.db");

    spotflow_client_t* client;
    if (spotflow_client_start(&client, options) != SPOTFLOW_OK)
    {
        show_last_error();
        return 1;
    }

    send_data(client);

    spotflow_client_destroy(client);
    spotflow_client_options_destroy(options);
}
