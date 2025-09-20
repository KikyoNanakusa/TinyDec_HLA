#include <stdio.h>

int main() {
    int i;
    int sum = 0;
    int count = 0;
    
    for (i = 0; i < 20; i++) {
        if (i % 2 == 0) {
            printf("Even: %d\n", i);
        }
        
        if (i % 3 == 0) {
            sum += i;
        }
        
        if (i > 10) {
            count++;
        }
        
        if (i == 15) {
            printf("Halfway point reached!\n");
        }
    }
    
    printf("Sum of multiples of 3: %d\n", sum);
    printf("Count of numbers > 10: %d\n", count);
    return 0;
}
