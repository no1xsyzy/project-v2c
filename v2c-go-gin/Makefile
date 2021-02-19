.PHONY: all
all: v2c

v2c: v2c.go
	go build

.PHONY: run
run: v2c
	GIN_MODE=release ./v2c
