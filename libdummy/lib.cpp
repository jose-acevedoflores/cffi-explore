//
// Created by bentlesf on 2/13/21.
//

#include <string>
#include <vector>

#include "MyLibrary.h"
#include "internal.h"
#include "lib.h"

auto lib = new MyLibrary();

int send_async(const char* dest, const char* arg, size_t argLen){
    auto s = std::string(dest);
    std::cout << "send " << s <<std::endl;
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

void m_des(FFIBuf buf) {
    std::cout << "destroy the buf" << std::endl;
    delete reinterpret_cast<std::vector<char>*>(buf.c_vec);

}

FFIBuf send_inline(const char* dest, const char* arg, size_t argLen){
    auto s = std::string(dest);

    auto vec = new std::vector<char>();
    int result = lib->send_inline(s, arg, argLen, *vec);

    auto buf = FFIBuf {
         vec->data(),
         vec->size(),
         m_des,
         reinterpret_cast<void*>(vec)
    };

    return buf;

}