#include "amps_ffi.h"
#include <ampsplusplus.hpp>
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
 *  More-specific exception types must be caught before their base classes.
 *  AlreadyConnectedException derives from ConnectionException, so it
 *  comes first in the catch chain.
 */
#define CATCH_AMPS_EXCEPTIONS(block) \
    try { \
        block; \
        return AMPS_FFI_OK; \
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
    } catch (const AMPS::ConnectionException& e) { \
        set_error(error, AMPS_FFI_ERROR_CONNECTION, e.what()); \
        return AMPS_FFI_ERROR_CONNECTION; \
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
    } catch (const AMPS::PublishStoreGapException& e) { \
        set_error(error, AMPS_FFI_ERROR_PUBLISH_STORE_GAP, e.what()); \
        return AMPS_FFI_ERROR_PUBLISH_STORE_GAP; \
    } catch (const AMPS::TimedOutException& e) { \
        set_error(error, AMPS_FFI_ERROR_TIMEOUT, e.what()); \
        return AMPS_FFI_ERROR_TIMEOUT; \
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
    amps_message_handler_t handler;
    void* user_data;
};

extern "C" {

/* ═══════════════════════════════════════════════════════════════════════
 *  Client lifecycle
 * ═══════════════════════════════════════════════════════════════════════ */

amps_client_t amps_ffi_client_create(const char* client_name, amps_ffi_error_info_t* error) {
    try {
        AMPS::Client* client = new AMPS::Client(client_name ? client_name : "");
        return reinterpret_cast<amps_client_t>(client);
    } catch (const std::exception& e) {
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, e.what());
        return nullptr;
    } catch (...) {
        set_error(error, AMPS_FFI_ERROR_UNKNOWN, "Unknown exception");
        return nullptr;
    }
}

void amps_ffi_client_destroy(amps_client_t client) {
    if (client) {
        delete reinterpret_cast<AMPS::Client*>(client);
    }
}

/* ═══════════════════════════════════════════════════════════════════════
 *  Connection
 * ═══════════════════════════════════════════════════════════════════════ */

int amps_ffi_client_connect(amps_client_t client, const char* uri, amps_ffi_error_info_t* error) {
    NULL_GUARD(client, uri);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    CATCH_AMPS_EXCEPTIONS(
        cpp_client->connect(uri)
    );
}

int amps_ffi_client_disconnect(amps_client_t client, amps_ffi_error_info_t* error) {
    NULL_GUARD(client);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    CATCH_AMPS_EXCEPTIONS(
        cpp_client->disconnect()
    );
}

int amps_ffi_client_logon(amps_client_t client, const char* options, int timeout_ms, amps_ffi_error_info_t* error) {
    NULL_GUARD(client);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    CATCH_AMPS_EXCEPTIONS(
        if (options) {
            cpp_client->logon(timeout_ms, AMPS::Authenticator(), options);
        } else {
            cpp_client->logon(timeout_ms);
        }
    );
}

/* ═══════════════════════════════════════════════════════════════════════
 *  Publishing
 * ═══════════════════════════════════════════════════════════════════════ */

uint64_t amps_ffi_client_publish(amps_client_t client,
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
        cpp_client->publish(topic, data, data_len, expiration);
        return 1; /* success – AMPS publish does not return a sequence number directly */
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

uint64_t amps_ffi_client_delta_publish(amps_client_t client,
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
        cpp_client->deltaPublish(topic, data, data_len);
        return 1;
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
        /* Cast away const – the FFI handle is opaque; the Rust side only
         * reads through amps_ffi_message_get_* which take const-equivalent
         * pointers internally. */
        amps_message_t handle = reinterpret_cast<amps_message_t>(
            const_cast<AMPS::Message*>(&msg));
        ctx->handler(handle, ctx->user_data);
    }
}

int amps_ffi_client_subscribe(amps_client_t client,
                               const char* topic,
                               const char* filter,
                               const char* options,
                               int timeout_ms,
                               amps_message_handler_t handler,
                               void* user_data,
                               amps_ffi_error_info_t* error) {
    NULL_GUARD(client, topic);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);

    /* Heap-allocate so the context outlives this call. */
    CallbackContext* ctx = new CallbackContext{ handler, user_data };

    CATCH_AMPS_EXCEPTIONS(
        AMPS::Command cmd("subscribe");
        cmd.setTopic(topic);
        if (filter) cmd.setFilter(filter);
        if (options) cmd.setOptions(options);
        if (timeout_ms > 0) cmd.setTimeout(timeout_ms);
        cpp_client->executeAsync(cmd, [ctx](const AMPS::Message& m) {
            subscription_trampoline(m, ctx);
        });
    );
}

int amps_ffi_client_unsubscribe(amps_client_t client, const char* sub_id, amps_ffi_error_info_t* error) {
    NULL_GUARD(client, sub_id);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    CATCH_AMPS_EXCEPTIONS(
        cpp_client->unsubscribe(sub_id)
    );
}

int amps_ffi_client_unsubscribe_all(amps_client_t client, amps_ffi_error_info_t* error) {
    NULL_GUARD(client);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    CATCH_AMPS_EXCEPTIONS(
        cpp_client->unsubscribe()
    );
}

/* ═══════════════════════════════════════════════════════════════════════
 *  SOW
 * ═══════════════════════════════════════════════════════════════════════ */

int amps_ffi_client_sow(amps_client_t client,
                         const char* topic,
                         const char* filter,
                         const char* order_by,
                         int batch_size,
                         int top_n,
                         int timeout_ms,
                         amps_message_handler_t handler,
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
        });
    );
}

int amps_ffi_client_sow_and_subscribe(amps_client_t client,
                                       const char* topic,
                                       const char* filter,
                                       const char* options,
                                       int timeout_ms,
                                       amps_message_handler_t handler,
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
        });
    );
}

/* ═══════════════════════════════════════════════════════════════════════
 *  Message access
 * ═══════════════════════════════════════════════════════════════════════ */

const char* amps_ffi_message_get_data(amps_message_t message, size_t* len) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    const char* data = msg->getData().data();
    if (len) *len = msg->getData().len();
    return data;
}

const char* amps_ffi_message_get_topic(amps_message_t message) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    return msg->getTopic().data();
}

const char* amps_ffi_message_get_command(amps_message_t message) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    return msg->getCommand().data();
}

const char* amps_ffi_message_get_sow_key(amps_message_t message) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    return msg->getSowKey().data();
}

const char* amps_ffi_message_get_bookmark(amps_message_t message) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    return msg->getBookmark().data();
}

const char* amps_ffi_message_get_sub_id(amps_message_t message) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    return msg->getSubId().data();
}

const char* amps_ffi_message_get_command_id(amps_message_t message) {
    if (!message) return nullptr;
    const AMPS::Message* msg = reinterpret_cast<const AMPS::Message*>(message);
    return msg->getCommandId().data();
}

/* ═══════════════════════════════════════════════════════════════════════
 *  Client configuration
 * ═══════════════════════════════════════════════════════════════════════ */

int amps_ffi_client_set_disconnect_handler(amps_client_t client,
                                            amps_disconnect_handler_t handler,
                                            void* user_data,
                                            amps_ffi_error_info_t* error) {
    NULL_GUARD(client);
    AMPS::Client* cpp_client = reinterpret_cast<AMPS::Client*>(client);
    CATCH_AMPS_EXCEPTIONS(
        cpp_client->setDisconnectHandler(
            [handler, user_data](AMPS::Client& c) {
                if (handler) {
                    handler(reinterpret_cast<amps_client_t>(&c), user_data);
                }
                return AMPS::DisconnectHandler::DisconnectAction::DoNotRetry;
            })
    );
}

int amps_ffi_client_set_heartbeat(amps_client_t client, unsigned heartbeat_time_sec, unsigned read_timeout_sec) {
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
        case AMPS_FFI_OK:                     return "OK";
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
