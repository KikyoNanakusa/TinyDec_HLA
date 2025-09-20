#include <stdio.h>

int main() {
    int i, j;
    int result = 0;
    
    for (i = 0; i < 10; i++) {
        if (i % 2 == 0) {
            printf("Processing even number: %d\n", i);
            
            if (i > 5) {
                printf("Large even number: %d\n", i);
                
                if (i == 8) {
                    printf("Special case: %d\n", i);
                    result += 100;
                } else {
                    result += i * 2;
                }
            } else {
                result += i;
            }
        } else {
            printf("Processing odd number: %d\n", i);
            
            if (i < 3) {
                result += i * 3;
            } else {
                result += i;
            }
        }
    }
    
    printf("Final result: %d\n", result);
    return 0;
}
