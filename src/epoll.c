#include <sys/epoll.h>

int main() {
    HANDLE ephnd = epoll_create(0);
    // TODO: remaining needed epoll methods..
    int result = epoll_close(ephnd);
    return 0;
}