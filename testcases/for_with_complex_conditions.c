#include <stdio.h>

int main() {
    int i;
    int prime_count = 0;
    int fibonacci_count = 0;
    
    for (i = 1; i <= 20; i++) {
        if (i == 2 || i == 3 || i == 5 || i == 7 || i == 11 || i == 13 || i == 17 || i == 19) {
            printf("Prime number: %d\n", i);
            prime_count++;
        }
        
        if (i == 1 || i == 2 || i == 3 || i == 5 || i == 8 || i == 13) {
            printf("Fibonacci number: %d\n", i);
            fibonacci_count++;
        }
        
        if (i % 2 == 0 && i % 3 == 0) {
            printf("Divisible by both 2 and 3: %d\n", i);
        }
        
        if (i > 10 && i < 15) {
            printf("In range 11-14: %d\n", i);
        }
        
        if (i == 1 || i == 8 || i == 15) {
            printf("Special milestone: %d\n", i);
        }
    }
    
    printf("Found %d prime numbers\n", prime_count);
    printf("Found %d fibonacci numbers\n", fibonacci_count);
    return 0;
}
