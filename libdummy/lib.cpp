
#include "lib.h"
#include <iostream>
#include <chrono>
#include <map>
#include "internal.h"



////////// ASYNC CB test
class MyLibrary {
   std::thread t;
   int i;
   std::map<std::string, InternalHandler*> handlers;
   void incr();
 public:
   void start();
   int send(const std::string& dest, const char* arg, size_t argLen);
   int handle(const std::string& dest, InternalHandler * internal_handler);
   int cancel(const std::string& dest, InternalHandler * internal_handler);
};


//void MyLibrary::register_cb(Wrapper *p) {
//
//    this->p = p;
//    this->p->cb(this->p->self, 89);
//}


void MyLibrary::start() {
    //Start thread
    t = std::thread(&MyLibrary::incr, this);
    handlers = std::map<std::string, InternalHandler*> ();
}

int MyLibrary::send(const std::string& dest, const char* arg, size_t argLen){
    std::cout << "C side send " << dest << std::endl;
    auto search = handlers.find(dest);
    if(search != handlers.end()){
        search->second->onSend(dest, arg, argLen);
        return 0;
    } else {
        std::cout << "handler for "<<dest << " was not found" << std::endl;
        return -1;
    }
}


int MyLibrary::handle(const std::string& dest, InternalHandler * internal_handler ){
    std::cout << "C side handle " << dest << std::endl;
    handlers.emplace(dest, internal_handler);
    return 0;
}

int MyLibrary::cancel(const std::string& dest, InternalHandler * internal_handler ){
    std::cout << "C side cancel " << dest << std::endl;
    auto search = handlers.find(dest);
    if(search != handlers.end()){
        delete search->second;
        return 0;
    } else {
        std::cout << "handler for "<<dest << " was not found" << std::endl;
        return -1;
    }

}


void MyLibrary::incr() {
    i = 0;
    while(true){
        i++;
        std::this_thread::sleep_for(std::chrono::seconds(2));
//        this->p->cb(this->p->self, i);
        std::cout<< "ON THE C SIDE " << i << "\n\n\n";
    }
}

auto lib = new MyLibrary();

/////////// Externs

int send(const char* dest, const char* arg, size_t argLen){
    auto s = std::string(dest);
    return lib->send(s, arg, argLen);
}

void* handler(const char* dest, FFIWrapper* p){
    auto s = std::string(dest);
    auto handler = new InternalHandler(p);
    int res = lib->handle(s, handler);

    return res >=0 ?  handler : nullptr;
}

int cancel(const char * dest, void *p) {
    auto s = std::string(dest);
    auto h = (InternalHandler*) p;
    return lib->cancel(s, h);
}

///////////////


/*

   //build .so
   g++ -shared -std=c++14 -o libdummy.so lib.cpp -lc


   https://nachtimwald.com/2017/08/18/wrapping-c-objects-in-c/

*/