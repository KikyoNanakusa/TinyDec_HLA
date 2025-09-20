#include <stdio.h>

void main(int n, int k) {
    if (k) goto LOOP;         
    puts("pre");
LOOP:
    puts("body");
    if (--n > 0) goto LOOP;   
    puts("after");
}