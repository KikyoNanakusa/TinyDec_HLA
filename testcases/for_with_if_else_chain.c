#include <stdio.h>

int main() {
    int i;
    int score = 0;
    
    for (i = 0; i < 15; i++) {
        if (i < 5) {
            printf("Beginner level: %d\n", i);
            score += 10;
        } else if (i < 10) {
            printf("Intermediate level: %d\n", i);
            score += 20;
        } else if (i < 13) {
            printf("Advanced level: %d\n", i);
            score += 30;
        } else {
            printf("Expert level: %d\n", i);
            score += 50;
        }
        
        if (i % 4 == 0) {
            printf("Bonus round at %d!\n", i);
            score += 5;
        }
    }
    
    printf("Total score: %d\n", score);
    return 0;
}
