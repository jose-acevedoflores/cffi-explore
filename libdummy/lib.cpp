
#include "lib.h"
#include <iostream>
#include <chrono>
#include <map>
#include "internal.h"
#include <vector>
#include <shared_mutex>
#include <mutex>

std::mutex g_display_mutex;

////////// ASYNC CB test
class MyLibrary {
   std::map<std::string, InternalHandler*> handlers;
   std::mutex handlers_mutex;
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

    //Lock before accessing handlers map. NOTE this could be a shared mutex (this would be a shared case)
    std::lock_guard<std::mutex> guard(handlers_mutex);
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

    //NOTE this could be a shared mutex (this would be an exclusive case for 'emplace')
    std::lock_guard<std::mutex> guard(handlers_mutex);
    handlers.emplace(dest, internal_handler);
    return 0;
}

int MyLibrary::cancel(const std::string& dest, InternalHandler * internal_handler ){
    std::cout << "C side cancel " << dest << std::endl;

    //Lock before accessing handlers map. NOTE this could be a shared mutex (this would be an exclusive case for 'erase')
    std::lock_guard<std::mutex> guard(handlers_mutex);
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
        std::cout << "handler for "<<dest << " was not found, hard search" << std::endl;
        for (auto const& e : handlers){
            if(e.second == internal_handler){
                std::cout << "hard search: found handler for "<< e.first << std::endl;
                handlers.erase(e.first);
                return 0;
            }
        }
    }

    return -1;
}


std::shared_timed_mutex lib_mutex;
auto lib = new MyLibrary();

void MyLibrary::incr(InternalHandler * h, const std::string dest) {
    int b = 0;
    auto rec = std::string("recurrent from c side: ");
    while( b < 20){
        b++;
        std::this_thread::sleep_for(std::chrono::seconds(1));
        lib_mutex.lock_shared();
        if(lib == nullptr){
            lib_mutex.unlock_shared();
            std::cout<< "ON THE C SIDE lib was shutdown. this ptr is invalid"<< std::endl;
            return;
        }
        auto i = std::to_string(b);
        rec.append(i);
        std::thread::id this_id = std::this_thread::get_id();
        g_display_mutex.lock();
        std::cout<< "ON THE C SIDE loop " << rec << " thread id: "<<this_id<< std::endl;
        g_display_mutex.unlock();
        //Lock before accessing handlers map. NOTE this could be a shared mutex (this would be a shared case)
        std::lock_guard<std::mutex> guard(handlers_mutex);
        if(this->handlers.find(dest) != this->handlers.end()){
            h->onSend(dest, rec.c_str(), rec.length());
            lib_mutex.unlock_shared();
        } else {
            std::cout<< "ON THE C SIDE loop '" << dest <<"' removed" << std::endl;
            lib_mutex.unlock_shared();
            return;
        }
    }
}


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
    lib_mutex.lock();
    delete lib;
    lib = nullptr;
    lib_mutex.unlock();
}

///////////////


/*

   //build .so (MAC)
   g++ -shared -std=c++14 -o libdummy.so lib.cpp -lc

   //build .so (linux)
   g++ -shared -std=c++14 -o libdummy.so lib.cpp -lc -fPIC

   https://nachtimwald.com/2017/08/18/wrapping-c-objects-in-c/

*/