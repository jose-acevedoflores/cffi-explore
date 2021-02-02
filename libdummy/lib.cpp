
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
   void send(const std::string& dest, const char* arg, size_t argLen);
   void handle(const std::string& dest, InternalHandler * internal_handler);
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

void MyLibrary::send(const std::string& dest, const char* arg, size_t argLen){
    std::cout << "C side send " << dest << std::endl;
    auto search = handlers.find(dest);
    if(search != handlers.end()){
        search->second->onSend(dest, arg, argLen);
    } else {
        std::cout << "handler for "<<dest << " was not found" << std::endl;
    }
}


void MyLibrary::handle(const std::string& dest, InternalHandler * internal_handler ){
    std::cout << "C side handle " << dest << std::endl;
    handlers.emplace(dest, internal_handler);
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

void send(const char* dest, const char* arg, size_t argLen){
    auto s = std::string(dest);
    lib->send(s, arg, argLen);
}

void handler(const char* dest, Wrapper* p){
    auto s = std::string(dest);
    auto handler = new InternalHandler(p);
    p->selfCSide = handler;
    lib->handle(s, handler);

}

///////////////


/*

   //build .so
   g++ -shared -std=c++14 -o libdummy.so lib.cpp -lc


   https://nachtimwald.com/2017/08/18/wrapping-c-objects-in-c/

*/