#ifndef DUMMY_LIB_H
#define DUMMY_LIB_H


typedef void* FFIHandler;

typedef void (*destruct)(void*);
struct FFIBuf {
   const char* data_ptr;
   size_t data_len;
   destruct cb;
   void* c_vec;
};


/**

   Signature of the callback for the FFIWrapper struct.
   - arg1: arg1 will be the 'cbSelf' field of the FFIWrapper struct

 */
typedef void (*callback_t)(FFIHandler, const char* dest, const char*, size_t);
typedef FFIBuf* (*callback_wr_t)(FFIHandler, const char* dest, const char*, size_t);
struct FFIWrapper {
    callback_t cb;
    callback_wr_t callback_with_return;
    FFIHandler cbSelf;
};

// Ptr to the Context object.
// As long as this ptr is valid, the 'FFIWrapper.cb' and 'cbSelf' will be considered valid.
// The  FFICtx will be invalidated after a call to 'cancel'.
// After invalidation, the user of the library can free any resources held by the FFIWrapper that was registered
// via the 'handler' function.
typedef void* FFICtx;


#ifdef __cplusplus
extern "C"
{
#endif

    int send(const char* dest, const char* arg, size_t argLen);
    FFIBuf* send_inline(const char* dest, const char* arg, size_t argLen);
    void m_des(void* buf);

    /**
        This function will register a user provided FFIWrapper ptr.
        Returns an FFICtx ptr that will be tied to the life of the given FFIWrapper
     */
    FFICtx handler(const char* dest, FFIWrapper* ext_handler);

    /**
        Cancels the given FFICtx ptr which will signal that the corresponding FFIWrapper will no longer be valid.
        User can free the FFIWrapper resources associated with the given FFICtx
     */
    int cancel(const char* dest, FFICtx ctx);

    /*
        Shutdowns the library, after this call all library calls are invalid.
     */
    void shutdown();

#ifdef __cplusplus
} // extern "C"
#endif




#endif // DUMMY_LIB_H