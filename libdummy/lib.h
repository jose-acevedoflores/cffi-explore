#pragma once

#include <thread>
#include <string>


typedef void (*callback_t)(void*, const char* dest, const char*, size_t);
typedef void* FFICtx;
struct FFIWrapper {
    /**
     * first argument of this cb will be 'cbSelf'
     */
    callback_t cb;
    void * cbSelf;
};




#ifdef __cplusplus
extern "C"
{
#endif

    int send(const char* dest, const char* arg, size_t argLen);
    FFICtx handler(const char* dest, FFIWrapper* p);
    int cancel(const char* dest, FFICtx p);

#ifdef __cplusplus
} // extern "C"
#endif