#include <stdio.h>

void main(int x) {
    if (x) {
        puts("T1");
        goto L;          
    }
    puts("JOIN");        
L:
    puts("AFTER");
}