#if defined _MSC_VER
#include <afunix.h>
#else
#include <sys/un.h>
#endif

int main() {
    sockaddr_un address;
    return 0;
}
