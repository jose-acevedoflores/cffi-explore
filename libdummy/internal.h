#include "lib.h"
#include <iostream>


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
};

