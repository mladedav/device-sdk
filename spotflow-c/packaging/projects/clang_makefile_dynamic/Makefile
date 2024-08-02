.PHONY: build run clean

SHELL = /bin/sh

SPOTFLOW_ROOT := $(realpath ../..)

CC = clang
CFLAGS = -Wall -Wextra -Wpedantic
CINCLUDES = -I"$(SPOTFLOW_ROOT)/include"
CLIBS = -L"$(SPOTFLOW_ROOT)/bin" -lspotflow -Wl,-rpath,$(SPOTFLOW_ROOT)/bin

build: get_started 

get_started.o: ../get_started.c
	$(CC) $(CFLAGS) $(CINCLUDES) -c ../get_started.c

get_started: get_started.o
	$(CC) $(CFLAGS) get_started.o -o get_started $(CLIBS)

run: build
	./get_started

clean:
	rm -f get_started.o get_started
