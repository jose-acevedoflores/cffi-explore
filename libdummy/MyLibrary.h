#ifndef DUMMY_MYLIBRARY_H
#define DUMMY_MYLIBRARY_H

#include <shared_mutex>
#include <map>
#include <deque>
#include <atomic>
#include <thread>
#include <vector>

#include "internal.h"

class MyLibrary {

public:
    MyLibrary() = default;
    virtual ~MyLibrary();

    void startIncrThread(InternalHandler * h, const std::string& dest);
    int send(const std::string& dest, const char* arg, size_t argLen);
    int send_inline(const std::string& dest, const char* arg, size_t argLen, std::vector<char> &vec);
    int handle(const std::string& dest, InternalHandler * internal_handler);
    int cancel(const std::string& dest, InternalHandler * internal_handler);
    void shutdown();

private:
    std::map<std::string, InternalHandler*> handlers;
    std::shared_mutex handlersGuard;
    std::deque<std::thread> incrThreads;
    std::atomic<bool> done = false;

    void incr(InternalHandler* h, const std::string& dest, bool isInlineSend);
};

#endif //DUMMY_MYLIBRARY_H
