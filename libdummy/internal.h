#ifndef DUMMY_INTERNAL_H
#define DUMMY_INTERNAL_H

#include <iostream>
#include "lib.h"

////////// ASYNC CB test
class InternalHandler {
    FFIWrapper *p;
 public:
    InternalHandler(FFIWrapper *p) {
        this->p = p;
    };

    ~InternalHandler(){
        std::cout << "~InternalHandler() destroyed" <<std::endl;
    }

    void onSend(const std::string& src, const char* arg, size_t argLen){
        std::cout << "C side onSend " << src << std::endl;
        p->cb(p->cbSelf, src.c_str(), arg, argLen);
    }


    std::string onSendInline(const std::string& src, const char* arg, size_t argLen){
        std::cout << "C side onSendInline " << src << std::endl;
        FFIBuf val = p->callback_with_return(p->cbSelf, src.c_str(), arg, argLen);

        auto s = std::string(val.data_ptr, val.data_len);

        val.cb(val);

        return s;
    }
};

#endif // DUMMY_INTERNAL_H
