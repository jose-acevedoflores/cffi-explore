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

void m_des(FFIBuf buf) {
    std::cout << "destroy the buf" << std::endl;
    std::cout << "destroy the buf2" << std::endl;
//    auto ffibuf = (FFIBuf*) buf;
    delete reinterpret_cast<std::vector<char>*>(buf.c_vec);
//    delete ffibuf;

//    free(buf);

}

FFIBuf send_inline(const char* dest, const char* arg, size_t argLen){
    auto s = std::string(dest);

    auto vec = new std::vector<char>();
    int result = lib->send_inline(s, arg, argLen, *vec);

//    std::string sfvec(vec.begin(), vec.end());
//    std::cout << "c side vec size " << vec.size() << " and str was '"<< sfvec <<"'"<< std::endl;

//    auto ptr = new FFIBuf();


//    std::copy(vec.begin(), vec.end(), e);
//    ptr->data_ptr = vec->data();
//    ptr->data_len = vec->size();
//    ptr->c_vec = reinterpret_cast<void*>(vec);
//    ptr->cb = m_des;

    auto buf = FFIBuf {
        .data_ptr = vec->data(),
        .data_len = vec->size(),
        .cb = m_des,
        .c_vec = reinterpret_cast<void*>(vec)
    };

    return buf;

//    return ptr;
//    return FFIBuf {
//        .data_ptr = vec.data(),
//        .data_len = vec.size(),
//    };
}