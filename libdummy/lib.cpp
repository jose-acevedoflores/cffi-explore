//
// Created by bentlesf on 2/13/21.
//

#include <string>
#include <vector>

#include "MyLibrary.h"
#include "internal.h"
#include "lib.h"

auto lib = new MyLibrary();

int send(const char* dest, const char* arg, size_t argLen){
    auto s = std::string(dest);
    return lib->send(s, arg, argLen);
}

void* handler(const char* dest, FFIWrapper* ext_handler){
    auto s = std::string(dest);
    auto handler = new InternalHandler(ext_handler);
    int res = lib->handle(s, handler);

    return res >=0 ?  handler : nullptr;
}

int cancel(const char * dest, void *ctx) {
    auto s = std::string(dest);
    auto h = (InternalHandler*) ctx;
    return lib->cancel(s, h);
}

void shutdown(){
    std::cout << "Shutdown libdummy" << std::endl;
    delete lib;
    lib = nullptr;
}

FFIBuf send_inline(const char* dest, const char* arg, size_t argLen){
    auto s = std::string(dest);
    std::vector<char> vec;
    int result = lib->send_inline(s, arg, argLen, vec);
    return FFIBuf {
        .data_ptr = vec.data(),
        .data_len = vec.size(),
    };
}