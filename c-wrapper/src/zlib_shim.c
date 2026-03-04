/*
 * Zlib shim for AMPS FFI
 * 
 * AMPS uses dynamic loading for zlib functions (via amps_zlib_init), but
 * this doesn't work well with static linking. This shim provides static
 * implementations that forward to the system zlib.
 */

#include <zlib.h>

/* Function pointer types matching AMPS */
typedef const char* (*amps_zlibVersion_t)(void);
typedef int (*amps_deflateInit2_t)(z_streamp strm, int level, int method,
                                    int windowBits, int memLevel,
                                    int strategy);
typedef int (*amps_deflate_t)(z_streamp strm, int flush);
typedef int (*amps_deflate_end_t)(z_streamp strm);
typedef int (*amps_inflateInit2_t)(z_streamp strm, int windowBits);
typedef int (*amps_inflate_t)(z_streamp strm, int flush);
typedef int (*amps_inflate_end_t)(z_streamp strm);

/* Global function pointers that AMPS expects - initialized to NULL so they go in data section */
amps_zlibVersion_t amps_zlibVersion = NULL;
amps_deflateInit2_t amps_deflateInit2_ = NULL;
amps_deflate_t amps_deflate = NULL;
amps_deflate_end_t amps_deflateEnd = NULL;
amps_inflateInit2_t amps_inflateInit2_ = NULL;
amps_inflate_t amps_inflate = NULL;
amps_inflate_end_t amps_inflateEnd = NULL;

/* Static initialization flag */
static int zlib_initialized = 0;

/* Shim functions that call system zlib */
static const char* shim_zlibVersion(void) {
    return zlibVersion();
}

static int shim_deflateInit2_(z_streamp strm, int level, int method,
                               int windowBits, int memLevel, int strategy) {
    return deflateInit2(strm, level, method, windowBits, memLevel, strategy);
}

static int shim_deflate(z_streamp strm, int flush) {
    return deflate(strm, flush);
}

static int shim_deflateEnd(z_streamp strm) {
    return deflateEnd(strm);
}

static int shim_inflateInit2_(z_streamp strm, int windowBits) {
    return inflateInit2(strm, windowBits);
}

static int shim_inflate(z_streamp strm, int flush) {
    return inflate(strm, flush);
}

static int shim_inflateEnd(z_streamp strm) {
    return inflateEnd(strm);
}

/* Initialize the function pointers to point to our shims
 * This is called automatically when the library is loaded
 */
static void __attribute__((constructor)) init_zlib_shim(void) {
    if (!zlib_initialized) {
        amps_zlibVersion = shim_zlibVersion;
        amps_deflateInit2_ = shim_deflateInit2_;
        amps_deflate = shim_deflate;
        amps_deflateEnd = shim_deflateEnd;
        amps_inflateInit2_ = shim_inflateInit2_;
        amps_inflate = shim_inflate;
        amps_inflateEnd = shim_inflateEnd;
        zlib_initialized = 1;
    }
}

/* Dummy init function that always succeeds */
int amps_zlib_init(const char* libraryName) {
    (void)libraryName;
    init_zlib_shim();
    return 0;
}

/* Error string function */
const char* amps_zlib_last_error(void) {
    return "";
}
