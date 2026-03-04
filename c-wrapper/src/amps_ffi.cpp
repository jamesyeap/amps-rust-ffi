#include "amps_ffi.h"
#include <amps/ampsplusplus.hpp>
#include <cstring>
#include <string>

#define AMPS_FFI_VERSION "0.1.0"

/* ── Helper: populate error info ── */
static void set_error(amps_ffi_error_info_t* error, amps_ffi_error_t code, const char* msg) {
    if (error) {
        error->code = code;
        strncpy(error->message, msg, sizeof(error->message) - 1);
        error->message[sizeof(error->message) - 1] = '\0';
    }
}

/* ── Helper macro to catch AMPS exceptions and populate error info ──
 *
 *  Catch order follows the actual AMPS hierarchy (util.hpp):
 *    AMPSException
 *    ├── ConnectionException
 *    │   ├── AlreadyConnectedException
 *    │   ├── AuthenticationException
 *    │   ├── ConnectionRefusedException
 *    │   ├── DisconnectedException
 *    │   ├── NameInUseException
 *    │   ├── NotEntitledException
 *    │   └── TimedOutException          ← derives from ConnectionException!
 *    ├── CommandException
 *    │   ├── BadFilterException
 *    │   ├── BadRegexTopicException
 *    │   ├── BadSowKeyException
 *    │   ├── InvalidTopicException
 *    │   ├── PublishException
 *    │   └── SubscriptionAlreadyExistsException
 *    └── StoreException
 *        └── PublishStoreGapException
 *
 *  Derived types MUST appear before their base class.
 */
#define CATCH_AMPS_EXCEPTIONS(block) \
    try { \
        block; \
        return AMPS_FFI_OK; \
    /* ── ConnectionException subtypes ── */ \
    } catch (const AMPS::AlreadyConnectedException& e) { \
        set_error(error, AMPS_FFI_ERROR_ALREADY_CONNECTED, e.what()); \
        return AMPS_FFI_ERROR_ALREADY_CONNECTED; \
    } catch (const AMPS::AuthenticationException& e) { \
        set_error(error, AMPS_FFI_ERROR_AUTHENTICATION, e.what()); \
        return AMPS_FFI_ERROR_AUTHENTICATION; \
    } catch (const AMPS::ConnectionRefusedException& e) { \
        set_error(error, AMPS_FFI_ERROR_CONNECTION_REFUSED, e.what()); \
        return AMPS_FFI_ERROR_CONNECTION_REFUSED; \
    } catch (const AMPS::DisconnectedException& e) { \
        set_error(error, AMPS_FFI_ERROR_DISCONNECTED, e.what()); \
        return AMPS_FFI_ERROR_DISCONNECTED; \
    } catch (const AMPS::NameInUseException& e) { \
        set_error(error, AMPS_FFI_ERROR_NAME_IN_USE, e.what()); \
        return AMPS_FFI_ERROR_NAME_IN_USE; \
    } catch (const AMPS::NotEntitledException& e) { \
        set_error(error, AMPS_FFI_ERROR_NOT_ENTITLED, e.what()); \
        return AMPS_FFI_ERROR_NOT_ENTITLED; \
    } catch (const AMPS::TimedOutException& e) { \
        set_error(error, AMPS_FFI_ERROR_TIMEOUT, e.what()); \
        return AMPS_FFI_ERROR_TIMEOUT; \
    /* ── ConnectionException base ── */ \
    } catch (const AMPS::ConnectionException& e) { \
        set_error(error, AMPS_FFI_ERROR_CONNECTION, e.what()); \
        return AMPS_FFI_ERROR_CONNECTION; \
    /* ── CommandException subtypes ── */ \
    } catch (const AMPS::BadFilterException& e) { \
        set_error(error, AMPS_FFI_ERROR_BAD_FILTER, e.what()); \
        return AMPS_FFI_ERROR_BAD_FILTER; \
    } catch (const AMPS::BadRegexTopicException& e) { \
        set_error(error, AMPS_FFI_ERROR_BAD_REGEX_TOPIC, e.what()); \
        return AMPS_FFI_ERROR_BAD_REGEX_TOPIC; \
    } catch (const AMPS::BadSowKeyException& e) { \
        set_error(error, AMPS_FFI_ERROR_BAD_SOW_KEY, e.what()); \
        return AMPS_FFI_ERROR_BAD_SOW_KEY; \
    } catch (const AMPS::InvalidTopicException& e) { \
        set_error(error, AMPS_FFI_ERROR_INVALID_TOPIC, e.what()); \
        return AMPS_FFI_ERROR_INVALID_TOPIC; \
    } catch (const AMPS::PublishException& e) { \
        set_error(error, AMPS_FFI_ERROR_PUBLISH, e.what()); \
        return AMPS_FFI_ERROR_PUBLISH; \
    } catch (const AMPS::SubscriptionAlreadyExistsException& e) { \
        set_error(error, AMPS_FFI_ERROR_SUBSCRIPTION_EXISTS, e.what()); \
        return AMPS_FFI_ERROR_SUBSCRIPTION_EXISTS; \
    /* ── CommandException base (not mapped to a specific code) ── */ \
    /* ── StoreException subtypes ── */ \
    } catch (const AMPS::PublishStoreGapException& e) { \
        set_error(error, AMPS_FFI_ERROR_PUBLISH_STORE_GAP, e.what()); \
        return AMPS_FFI_ERROR_PUBLISH_STORE_GAP; \
    /* ── AMPSException base ── */ \
    } catch (const AMPS::AMPSException& e) { \
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, e.what()); \
        return AMPS_FFI_ERROR_UNKNOWN; \
    } catch (const std::exception& e) { \
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, e.what()); \
        return AMPS_FFI_ERROR_UNKNOWN; \
    } catch (...) { \
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, "Unknown exception"); \
        return AMPS_FFI_ERROR_UNKNOWN; \
    }

/* ── Helper: NULL-pointer guard (returns error code) ── */
#define NULL_GUARD(...) \
    do { \
        const void* _ptrs[] = { __VA_ARGS__ }; \
        for (size_t _i = 0; _i < sizeof(_ptrs)/sizeof(_ptrs[0]); ++_i) { \
            if (!_ptrs[_i]) { \
                set_error(error, AMPS_FFI_ERROR_NULL_POINTER, "Null pointer argument"); \
                return AMPS_FFI_ERROR_NULL_POINTER; \
            } \
        } \
    } while (0)

/* ── Subscription callback context ── */
struct CallbackContext {
    amps_ffi_message_handler_t handler;
    void* user_data;
};

extern "C" {

/* ═══════════════════════════════════════════════════════════════════════
 *  Client lifecycle
 * ═══════════════════════════════════════════════════════════════════════ */

amps_ffi_client_t amps_ffi_client_create(const char* client_name, amps_ffi_error_info_t* error) {
    try {
        AMPS::Client* client = new AMPS::Client(client_name ? client_name : "");
        return reinterpret_cast<amps_ffi_client_t>(client);
    } catch (const std::exception& e) {
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, e.what());
        return nullptr;
    } catch (...) {
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, "Unknown exception");
        return nullptr;
    }
}

void amps_ffi_client_destroy(amps_ffi_client_t client) {
    if (client) {
        delete reinterpret_cast<AMPS::Client*>(client);
    }
}

/* ═══════════════════════════════════════════════════════════════════════
 *  Connection
 * ═══════════════════════════════════════════════════════════════════════ */

int amps_ffi_client_connect(amps_ffi_client_t client, const char* uri, amps_ffi_error_info_t* error) {
    NULL_GUARD(client, uri);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    CATCH_AMPS_EXCEPTIONS(
        cpp_client->connect(uri)
    );
}

int amps_ffi_client_disconnect(amps_ffi_client_t client, amps_ffi_error_info_t* error) {
    NULL_GUARD(client);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    CATCH_AMPS_EXCEPTIONS(
        cpp_client->disconnect()
    );
}

int amps_ffi_client_logon(amps_ffi_client_t client, const char* options, int timeout_ms, amps_ffi_error_info_t* error) {
    NULL_GUARD(client);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    CATCH_AMPS_EXCEPTIONS(
        if (options) {
            cpp_client->logon(options, timeout_ms);
        } else {
            cpp_client->logon(timeout_ms);
        }
    );
}

/* ═══════════════════════════════════════════════════════════════════════
 *  Publishing
 * ═══════════════════════════════════════════════════════════════════════ */

uint64_t amps_ffi_client_publish(amps_ffi_client_t client,
                                  const char* topic,
                                  const char* data,
                                  size_t data_len,
                                  unsigned long expiration,
                                  amps_ffi_error_info_t* error) {
    if (!client || !topic || !data) {
        set_error(error, AMPS_FFI_ERROR_NULL_POINTER, "Null pointer argument");
        return 0;
    }
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    try {
        if (expiration > 0) {
            return cpp_client->publish(topic, strlen(topic), data, data_len, expiration);
        } else {
            return cpp_client->publish(topic, strlen(topic), data, data_len);
        }
    } catch (const AMPS::AMPSException& e) {
        set_error(error, AMPS_FFI_ERROR_PUBLISH, e.what());
        return 0;
    } catch (const std::exception& e) {
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, e.what());
        return 0;
    } catch (...) {
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, "Unknown exception");
        return 0;
    }
}

uint64_t amps_ffi_client_delta_publish(amps_ffi_client_t client,
                                        const char* topic,
                                        const char* data,
                                        size_t data_len,
                                        amps_ffi_error_info_t* error) {
    if (!client || !topic || !data) {
        set_error(error, AMPS_FFI_ERROR_NULL_POINTER, "Null pointer argument");
        return 0;
    }
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    try {
        return cpp_client->deltaPublish(topic, data, data_len);
    } catch (const AMPS::AMPSException& e) {
        set_error(error, AMPS_FFI_ERROR_PUBLISH, e.what());
        return 0;
    } catch (const std::exception& e) {
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, e.what());
        return 0;
    } catch (...) {
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, "Unknown exception");
        return 0;
    }
}

/* ═══════════════════════════════════════════════════════════════════════
 *  Subscription
 * ═══════════════════════════════════════════════════════════════════════ */

static void subscription_trampoline(const AMPS::Message& msg, void* user_data) {
    CallbackContext* ctx = static_cast<CallbackContext*>(user_data);
    if (ctx && ctx->handler) {
        amps_ffi_message_t handle = reinterpret_cast<amps_ffi_message_t>(
            const_cast<AMPS::Message*>(&msg));
        ctx->handler(handle, ctx->user_data);
    }
}

int amps_ffi_client_subscribe(amps_ffi_client_t client,
                               const char* topic,
                               const char* filter,
                               const char* options,
                               int timeout_ms,
                               amps_ffi_message_handler_t handler,
                               void* user_data,
                               amps_ffi_error_info_t* error) {
    NULL_GUARD(client, topic);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);

    CallbackContext* ctx = new CallbackContext{ handler, user_data };

    CATCH_AMPS_EXCEPTIONS(
        AMPS::Command cmd("subscribe");
        cmd.setTopic(topic);
        if (filter) cmd.setFilter(filter);
        if (options) cmd.setOptions(options);
        if (timeout_ms > 0) cmd.setTimeout(timeout_ms);
        cpp_client->executeAsync(cmd, [ctx](const AMPS::Message& m) {
            subscription_trampoline(m, ctx);
        })
    );
}

int amps_ffi_client_unsubscribe(amps_ffi_client_t client, const char* sub_id, amps_ffi_error_info_t* error) {
    NULL_GUARD(client, sub_id);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    CATCH_AMPS_EXCEPTIONS(
        cpp_client->unsubscribe(sub_id)
    );
}

int amps_ffi_client_unsubscribe_all(amps_ffi_client_t client, amps_ffi_error_info_t* error) {
    NULL_GUARD(client);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    CATCH_AMPS_EXCEPTIONS(
        cpp_client->unsubscribe()
    );
}

/* ═══════════════════════════════════════════════════════════════════════
 *  SOW
 * ═══════════════════════════════════════════════════════════════════════ */

int amps_ffi_client_sow(amps_ffi_client_t client,
                         const char* topic,
                         const char* filter,
                         const char* order_by,
                         int batch_size,
                         int top_n,
                         int timeout_ms,
                         amps_ffi_message_handler_t handler,
                         void* user_data,
                         amps_ffi_error_info_t* error) {
    NULL_GUARD(client, topic);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);

    CallbackContext* ctx = new CallbackContext{ handler, user_data };

    CATCH_AMPS_EXCEPTIONS(
        AMPS::Command cmd("sow");
        cmd.setTopic(topic);
        if (filter) cmd.setFilter(filter);
        if (order_by) cmd.setOrderBy(order_by);
        if (batch_size > 0) cmd.setBatchSize(batch_size);
        if (top_n > 0) cmd.setTopN(top_n);
        if (timeout_ms > 0) cmd.setTimeout(timeout_ms);
        cpp_client->executeAsync(cmd, [ctx](const AMPS::Message& m) {
            subscription_trampoline(m, ctx);
        })
    );
}

int amps_ffi_client_sow_and_subscribe(amps_ffi_client_t client,
                                       const char* topic,
                                       const char* filter,
                                       const char* options,
                                       int timeout_ms,
                                       amps_ffi_message_handler_t handler,
                                       void* user_data,
                                       amps_ffi_error_info_t* error) {
    NULL_GUARD(client, topic);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);

    CallbackContext* ctx = new CallbackContext{ handler, user_data };

    CATCH_AMPS_EXCEPTIONS(
        AMPS::Command cmd("sow_and_subscribe");
        cmd.setTopic(topic);
        if (filter) cmd.setFilter(filter);
        if (options) cmd.setOptions(options);
        if (timeout_ms > 0) cmd.setTimeout(timeout_ms);
        cpp_client->executeAsync(cmd, [ctx](const AMPS::Message& m) {
            subscription_trampoline(m, ctx);
        })
    );
}

/* ═══════════════════════════════════════════════════════════════════════
 *  Message access
 * ═══════════════════════════════════════════════════════════════════════ */

/* Helper to copy Field data to thread-local buffer.
 * This ensures the returned pointer remains valid after the function returns.
 * The AMPS::Field class stores data as length + pointer, but the pointer may
 * point to internal message buffer that can be invalidated.
 */
static const char* copy_to_thread_local_buffer(const AMPS::Field& field, size_t* len) {
    // Thread-local buffer to store the copied data
    static thread_local std::string buffer;
    
    if (field.len() == 0 || field.data() == nullptr) {
        if (len) *len = 0;
        return "";
    }
    
    // Copy the data to the buffer
    buffer.assign(field.data(), field.len());
    if (len) *len = buffer.length();
    return buffer.c_str();
}

const char* amps_ffi_message_get_data(amps_ffi_message_t message, size_t* len) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    const AMPS::Field& f = msg->getData();
    return copy_to_thread_local_buffer(f, len);
}

const char* amps_ffi_message_get_topic(amps_ffi_message_t message, size_t* len) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    const AMPS::Field& f = msg->getTopic();
    return copy_to_thread_local_buffer(f, len);
}

const char* amps_ffi_message_get_command(amps_ffi_message_t message, size_t* len) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    const AMPS::Field& f = msg->getCommand();
    return copy_to_thread_local_buffer(f, len);
}

const char* amps_ffi_message_get_sow_key(amps_ffi_message_t message, size_t* len) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    const AMPS::Field& f = msg->getSowKey();
    return copy_to_thread_local_buffer(f, len);
}

const char* amps_ffi_message_get_bookmark(amps_ffi_message_t message, size_t* len) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    const AMPS::Field& f = msg->getBookmark();
    return copy_to_thread_local_buffer(f, len);
}

const char* amps_ffi_message_get_sub_id(amps_ffi_message_t message, size_t* len) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    const AMPS::Field& f = msg->getSubId();
    return copy_to_thread_local_buffer(f, len);
}

const char* amps_ffi_message_get_command_id(amps_ffi_message_t message, size_t* len) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    const AMPS::Field& f = msg->getCommandId();
    return copy_to_thread_local_buffer(f, len);
}

/* ═══════════════════════════════════════════════════════════════════════
 *  Client configuration
 * ═══════════════════════════════════════════════════════════════════════ */

/* Context for disconnect handler trampoline */
struct DisconnectContext {
    amps_ffi_disconnect_handler_t handler;
    void* user_data;
};

static void disconnect_trampoline(AMPS::Client& c, void* user_data) {
    DisconnectContext* ctx = static_cast<DisconnectContext*>(user_data);
    if (ctx && ctx->handler) {
        ctx->handler(reinterpret_cast<amps_ffi_client_t>(&c), ctx->user_data);
    }
}

int amps_ffi_client_set_disconnect_handler(amps_ffi_client_t client,
                                            amps_ffi_disconnect_handler_t handler,
                                            void* user_data,
                                            amps_ffi_error_info_t* error) {
    NULL_GUARD(client);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);

    /* Heap-allocate context so it outlives this call */
    DisconnectContext* ctx = new DisconnectContext{ handler, user_data };

    try {
#if defined(__clang__) || defined(__GNUC__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wdeprecated-declarations"
#endif
        cpp_client->setDisconnectHandler(
            AMPS::DisconnectHandler(disconnect_trampoline, ctx));
#if defined(__clang__) || defined(__GNUC__)
#pragma GCC diagnostic pop
#endif
        return AMPS_FFI_OK;
    } catch (const AMPS::AMPSException& e) {
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, e.what());
        return AMPS_FFI_ERROR_UNKNOWN;
    } catch (const std::exception& e) {
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, e.what());
        return AMPS_FFI_ERROR_UNKNOWN;
    } catch (...) {
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, "Unknown exception");
        return AMPS_FFI_ERROR_UNKNOWN;
    }
}

int amps_ffi_client_set_heartbeat(amps_ffi_client_t client, unsigned heartbeat_time_sec, unsigned read_timeout_sec) {
    if (!client) return AMPS_FFI_ERROR_NULL_POINTER;
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    try {
        cpp_client->setHeartbeat(heartbeat_time_sec, read_timeout_sec);
        return AMPS_FFI_OK;
    } catch (...) {
        return AMPS_FFI_ERROR_UNKNOWN;
    }
}

/* ═══════════════════════════════════════════════════════════════════════
 *  Utility
 * ═══════════════════════════════════════════════════════════════════════ */

const char* amps_ffi_error_string(amps_ffi_error_t error_code) {
    switch (error_code) {
        case AMPS_FFI_OK:                      return "OK";
        case AMPS_FFI_ERROR_CONNECTION:        return "Connection error";
        case AMPS_FFI_ERROR_ALREADY_CONNECTED: return "Already connected";
        case AMPS_FFI_ERROR_AUTHENTICATION:    return "Authentication failed";
        case AMPS_FFI_ERROR_CONNECTION_REFUSED: return "Connection refused";
        case AMPS_FFI_ERROR_DISCONNECTED:      return "Disconnected";
        case AMPS_FFI_ERROR_NAME_IN_USE:       return "Client name in use";
        case AMPS_FFI_ERROR_NOT_ENTITLED:      return "Not entitled";
        case AMPS_FFI_ERROR_BAD_FILTER:        return "Bad filter expression";
        case AMPS_FFI_ERROR_BAD_REGEX_TOPIC:   return "Bad regex topic";
        case AMPS_FFI_ERROR_BAD_SOW_KEY:       return "Bad SOW key";
        case AMPS_FFI_ERROR_INVALID_TOPIC:     return "Invalid topic";
        case AMPS_FFI_ERROR_PUBLISH:           return "Publish error";
        case AMPS_FFI_ERROR_SUBSCRIPTION_EXISTS: return "Subscription already exists";
        case AMPS_FFI_ERROR_PUBLISH_STORE_GAP: return "Publish store gap";
        case AMPS_FFI_ERROR_TIMEOUT:           return "Operation timed out";
        case AMPS_FFI_ERROR_UNKNOWN:           return "Unknown error";
        case AMPS_FFI_ERROR_NULL_POINTER:      return "Null pointer";
        case AMPS_FFI_ERROR_INVALID_ARGUMENT:  return "Invalid argument";
        default:                               return "Unrecognized error code";
    }
}

const char* amps_ffi_version(void) {
    return AMPS_FFI_VERSION;
}

} /* extern "C" */
