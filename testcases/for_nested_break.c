#include <stdio.h>

int main() {
    int i, j;
    
    for (i = 0; i < 5; i++) {
        printf("外側ループ: i = %d\n", i);
        
        for (j = 0; j < 10; j++) {
            if (j == 3) {
                printf("  j = %d でbreak\n", j);
                break;  
            }
            printf("  内側ループ: j = %d\n", j);
        }
        
        if (i == 2) {
            printf("i = %d で外側ループからbreak\n", i);
            break;  
        }
    }
    
    printf("ループ終了\n");
    return 0;
}