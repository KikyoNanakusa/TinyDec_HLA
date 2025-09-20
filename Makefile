.PHONY: all clean

CC := gcc
CFLAGS := -O0 -m64 -march=x86-64 -mtune=generic -fPIC

SRCS := $(wildcard testcases/*.c)
BINS := $(patsubst testcases/%.c,testcases/%.out,$(SRCS))

all: $(BINS)

testcases/%.out: testcases/%.c
	$(CC) $(CFLAGS) $< -o $@

clean:
	rm -f $(BINS)
