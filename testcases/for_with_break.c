#include <stdio.h>

int main() {
    int i;
    for (i = 0; i < 5; i++) {
        printf("外側ループ: i = %d\n", i);

		if (i == 2) {
			printf("i = %d で外側ループからbreak\n", i);
			break;  
		}
	}
    return 0;
}