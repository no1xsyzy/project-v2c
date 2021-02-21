export CC=/usr/sbin/gcc
PONYC=corral run -- ponyc
OPENSSL_VERSION=1.1.x
FLAGS=-Dopenssl_$(OPENSSL_VERSION)

.PHONY: all
all: v2c-pony

v2c-pony: v2c.pony corral.json lock.json
	$(PONYC) $(FLAGS)

run: v2c-pony
	./v2c-pony
