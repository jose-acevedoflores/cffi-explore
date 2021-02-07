
#include "lib.h"
#include <iostream>
#include <chrono>
#include <map>
#include "internal.h"
#include <vector>


////////// ASYNC CB test
class MyLibrary {
   std::map<std::string, InternalHandler*> handlers;
//   std::vector<std::thread> ths;
   void incr(InternalHandler * h, const std::string dest);
 public:
   ~MyLibrary();

   void start(InternalHandler * h, const std::string& dest);
   int send(const std::string& dest, const char* arg, size_t argLen);
   int handle(const std::string& dest, InternalHandler * internal_handler);
   int cancel(const std::string& dest, InternalHandler * internal_handler);
};

MyLibrary::~MyLibrary() {
    std::cout<< "lib destroyed" <<std::endl;
}

void MyLibrary::start(InternalHandler * h, const std::string& dest) {
    //Start thread
    auto copy = std::string(dest);
    //I think I'm doing this std::move right ?
    auto t = std::thread(&MyLibrary::incr, this, h, std::move(copy));
//  ths.push_back(std::move(t));
    t.detach();
}

int MyLibrary::send(const std::string& dest, const char* arg, size_t argLen){
    std::cout << "C side send " << dest << std::endl;
    auto search = handlers.find(dest);
    if(search != handlers.end()){
        search->second->onSend(dest, arg, argLen);
        start(search->second, dest);
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
        if(search->second == internal_handler){
            delete search->second;
            handlers.erase(dest);
            return 0;
        } else {
            std::cout << "wrong internal_handler " << std::endl;
        }
    } else {
        std::cout << "handler for "<<dest << " was not found" << std::endl;
    }

    //TODO it should probably destroy 'internal_handler' anyway otherwise there could be memory leak.
    return -1;
}


void MyLibrary::incr(InternalHandler * h, const std::string dest) {
    int b = 0;
    auto rec = std::string("recurrent from c side: ");
    while( b < 20){
        b++;
        std::this_thread::sleep_for(std::chrono::seconds(1));
        auto i = std::to_string(b);
        rec.append(i);
        std::cout<< "ON THE C SIDE loop " << rec << std::endl;
        if(this->handlers.find(dest) != this->handlers.end()){
            //NOTE: this is NOT SAFE since between checking the handlers and calling onSend the handler could be canceled
            //     But this is test code so we good.
            h->onSend(dest, rec.c_str(), rec.length());
        } else {
            std::cout<< "ON THE C SIDE loop '" << dest <<"' removed" << std::endl;
            return;
        }
    }
}

auto lib = new MyLibrary();

/////////// Externs

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
}

///////////////


/*

   //build .so (MAC)
   g++ -shared -std=c++14 -o libdummy.so lib.cpp -lc

   //build .so (linux)
   g++ -shared -std=c++14 -o libdummy.so lib.cpp -lc -fPIC

   https://nachtimwald.com/2017/08/18/wrapping-c-objects-in-c/

*/