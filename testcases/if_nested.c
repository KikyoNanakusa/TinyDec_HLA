#include <stdio.h>

int main() {
    int a = 1;
    if (a == 1) {
        printf("a is 1\n");
		if (a == 2) {
			printf("a is 2\n");
		} else {
			printf("a is not 2\n");
		}
	} else {
		printf("a is not 1\n");
    }
    return 0;
}