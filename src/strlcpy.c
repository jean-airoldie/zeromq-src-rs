#include <string.h>

int main() {
    char buf[1];
    (void)strlcpy(buf, "a", 1);
    return 0;
}
