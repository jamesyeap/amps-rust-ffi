#ifndef AMPS_FFI_H
#define AMPS_FFI_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>

/* Opaque handle types */
typedef struct amps_client_handle* amps_client_t;
typedef struct amps_message_handle* amps_message_t;
typedef struct amps_command_handle* amps_command_t;

/* Error codes */
typedef enum {
    AMPS_FFI_OK = 0,
    AMPS_FFI_ERROR_CONNECTION,
    AMPS_FFI_ERROR_ALREADY_CONNECTED,
    AMPS_FFI_ERROR_AUTHENTICATION,
    AMPS_FFI_ERROR_CONNECTION_REFUSED,
    AMPS_FFI_ERROR_DISCONNECTED,
    AMPS_FFI_ERROR_NAME_IN_USE,
    AMPS_FFI_ERROR_NOT_ENTITLED,
    AMPS_FFI_ERROR_BAD_FILTER,
    AMPS_FFI_ERROR_BAD_REGEX_TOPIC,
    AMPS_FFI_ERROR_BAD_SOW_KEY,
    AMPS_FFI_ERROR_INVALID_TOPIC,
    AMPS_FFI_ERROR_PUBLISH,
    AMPS_FFI_ERROR_SUBSCRIPTION_EXISTS,
    AMPS_FFI_ERROR_PUBLISH_STORE_GAP,
    AMPS_FFI_ERROR_TIMEOUT,
    AMPS_FFI_ERROR_UNKNOWN,
    AMPS_FFI_ERROR_NULL_POINTER,
    AMPS_FFI_ERROR_INVALID_ARGUMENT
} amps_ffi_error_t;

/* Error information structure */
typedef struct {
    amps_ffi_error_t code;
    char message[1024];
} amps_ffi_error_info_t;

/* Message handler callback type */
typedef void (*amps_message_handler_t)(amps_message_t message, void* user_data);

/* Disconnect handler callback type */
typedef void (*amps_disconnect_handler_t)(amps_client_t client, void* user_data);

/* ── Client lifecycle ── */
amps_client_t amps_ffi_client_create(const char* client_name, amps_ffi_error_info_t* error);
void amps_ffi_client_destroy(amps_client_t client);

/* ── Connection ── */
int amps_ffi_client_connect(amps_client_t client, const char* uri, amps_ffi_error_info_t* error);
int amps_ffi_client_disconnect(amps_client_t client, amps_ffi_error_info_t* error);
int amps_ffi_client_logon(amps_client_t client, const char* options, int timeout_ms, amps_ffi_error_info_t* error);

/* ── Publishing ── */
uint64_t amps_ffi_client_publish(amps_client_t client,
                                  const char* topic,
                                  const char* data,
                                  size_t data_len,
                                  unsigned long expiration,
                                  amps_ffi_error_info_t* error);

uint64_t amps_ffi_client_delta_publish(amps_client_t client,
                                        const char* topic,
                                        const char* data,
                                        size_t data_len,
                                        amps_ffi_error_info_t* error);

/* ── Subscription ── */
int amps_ffi_client_subscribe(amps_client_t client,
                               const char* topic,
                               const char* filter,
                               const char* options,
                               int timeout_ms,
                               amps_message_handler_t handler,
                               void* user_data,
                               amps_ffi_error_info_t* error);

int amps_ffi_client_unsubscribe(amps_client_t client, const char* sub_id, amps_ffi_error_info_t* error);
int amps_ffi_client_unsubscribe_all(amps_client_t client, amps_ffi_error_info_t* error);

/* ── SOW ── */
int amps_ffi_client_sow(amps_client_t client,
                         const char* topic,
                         const char* filter,
                         const char* order_by,
                         int batch_size,
                         int top_n,
                         int timeout_ms,
                         amps_message_handler_t handler,
                         void* user_data,
                         amps_ffi_error_info_t* error);

int amps_ffi_client_sow_and_subscribe(amps_client_t client,
                                       const char* topic,
                                       const char* filter,
                                       const char* options,
                                       int timeout_ms,
                                       amps_message_handler_t handler,
                                       void* user_data,
                                       amps_ffi_error_info_t* error);

/* ── Message access (from handler callbacks) ── */
const char* amps_ffi_message_get_data(amps_message_t message, size_t* len);
const char* amps_ffi_message_get_topic(amps_message_t message);
const char* amps_ffi_message_get_command(amps_message_t message);
const char* amps_ffi_message_get_sow_key(amps_message_t message);
const char* amps_ffi_message_get_bookmark(amps_message_t message);
const char* amps_ffi_message_get_sub_id(amps_message_t message);
const char* amps_ffi_message_get_command_id(amps_message_t message);

/* ── Client configuration ── */
int amps_ffi_client_set_disconnect_handler(amps_client_t client,
                                            amps_disconnect_handler_t handler,
                                            void* user_data,
                                            amps_ffi_error_info_t* error);

int amps_ffi_client_set_heartbeat(amps_client_t client, unsigned heartbeat_time_sec, unsigned read_timeout_sec);

/* ── Utility ── */
const char* amps_ffi_error_string(amps_ffi_error_t error_code);
const char* amps_ffi_version(void);

#ifdef __cplusplus
}
#endif

#endif /* AMPS_FFI_H */
