#pragma once

#include <thread>
#include <string>


typedef void (*callback_t)(void*, const char* dest, const char*, size_t);
struct Wrapper {
    callback_t cb;
    void * selfCSide;
    void * outerSelf;
};




#ifdef __cplusplus
extern "C"
{
#endif

    void send(const char* dest, const char* arg, size_t argLen);
    void handler(const char* dest, Wrapper* p);

#ifdef __cplusplus
} // extern "C"
#endif