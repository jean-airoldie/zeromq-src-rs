#include <winsock2.h>
#include <afunix.h>

int main() {
    // Use a type from winsock2.h: SOCKET
    SOCKET sock = INVALID_SOCKET;
    // Use a constant from afunix.h: AF_UNIX
    int family = AF_UNIX;
    return 0;
}
