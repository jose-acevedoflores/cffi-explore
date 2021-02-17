#include <iostream>
#include <chrono>

#include "MyLibrary.h"

MyLibrary::~MyLibrary() {
    done = true;
    for( auto& thread : incrThreads ) {
        thread.join();
    }

    //Lock exclusive for 'erase'
    std::unique_lock guard(handlersGuard);
    for (auto const& e : handlers){

        std::cout << "found dangling handler for '" << e.first << "'" << std::endl;
        delete e.second;
    }

    std::cout << "lib destroyed" << std::endl;
}

void MyLibrary::startIncrThread(InternalHandler * h, const std::string& dest) {
    //Start thread
    incrThreads.emplace_back( std::thread( &MyLibrary::incr, this, h, std::string(dest) ) );
}

int MyLibrary::send(const std::string& dest, const char* arg, size_t argLen){
    std::cout << "C side send " << dest << std::endl;

    //Lock before accessing handlers map.
    std::shared_lock guard(handlersGuard);

    auto search = handlers.find(dest);
    if(search != handlers.end()){
        search->second->onSend(dest, arg, argLen);
        startIncrThread(search->second, dest);
        return 0;
    } else {
        std::cout << "handler for "<<dest << " was not found" << std::endl;
        return -1;
    }
}


int MyLibrary::handle(const std::string& dest, InternalHandler * internal_handler ){
    std::cout << "C side handle " << dest << std::endl;

    //Lock exclusive for 'emplace'
    std::unique_lock guard(handlersGuard);

    handlers.emplace(dest, internal_handler);
    return 0;
}

int MyLibrary::cancel(const std::string& dest, InternalHandler * internal_handler ){
    std::cout << "C side cancel " << dest << std::endl;

    //Lock exclusive for 'erase'
    std::unique_lock guard(handlersGuard);

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
                delete internal_handler;
                return 0;
            }
        }
    }

    return -1;
}

void MyLibrary::incr(InternalHandler * h, const std::string& dest) {
    int b = 0;
    auto rec = std::string("recurrent from c side: ");
    while( b < 20 ){
        b++;
        std::this_thread::sleep_for(std::chrono::seconds(1));
        if( done ){
            std::cout<< "ON THE C SIDE lib was shutdown. this ptr is invalid"<< std::endl;
            return;
        }
        auto i = std::to_string(b);
        rec.append(i);
        std::cout<< "ON THE C SIDE loop " << rec << " thread id: "<<std::this_thread::get_id()<< std::endl;

        //Lock before accessing handlers map.
        std::shared_lock guard(handlersGuard);

        if(handlers.find(dest) != handlers.end()){
            h->onSend(dest, rec.c_str(), rec.length());
        } else {
            std::cout<< "ON THE C SIDE loop '" << dest <<"' removed" << std::endl;
            return;
        }
    }
}

