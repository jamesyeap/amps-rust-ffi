/*
 * test_header.c – Verifies that amps_ffi.h is valid C and that all
 * declared types, enums, and function prototypes are accessible.
 *
 * Compile: cc -std=c11 -I../include -c test_header.c -o /dev/null
 */

#include "amps_ffi.h"
#include <assert.h>
#include <stddef.h>

int main(void) {
    /* Error enum values exist and have expected ordering */
    assert(AMPS_FFI_OK == 0);
    assert(AMPS_FFI_ERROR_CONNECTION > AMPS_FFI_OK);
    assert(AMPS_FFI_ERROR_UNKNOWN > AMPS_FFI_OK);
    assert(AMPS_FFI_ERROR_NULL_POINTER > AMPS_FFI_OK);
    assert(AMPS_FFI_ERROR_INVALID_ARGUMENT > AMPS_FFI_OK);

    /* Error info struct has expected layout */
    amps_ffi_error_info_t err;
    err.code = AMPS_FFI_OK;
    err.message[0] = '\0';
    assert(sizeof(err.message) == 1024);

    /* Opaque handles are pointer-sized */
    assert(sizeof(amps_client_t) == sizeof(void*));
    assert(sizeof(amps_message_t) == sizeof(void*));
    assert(sizeof(amps_command_t) == sizeof(void*));

    /* Callback typedefs are function pointers (can be set to NULL) */
    amps_message_handler_t msg_handler = NULL;
    amps_disconnect_handler_t dc_handler = NULL;
    (void)msg_handler;
    (void)dc_handler;

    /* Function pointers are resolvable (link-time – here we just take addresses).
     * We cast to void* to avoid unused-variable warnings. */
    (void)(amps_ffi_error_string);
    (void)(amps_ffi_version);

    return 0;
}
