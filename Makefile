SUBDIRS := $(wildcard v2c-*/.)

.PHONY: all $(SUBDIRS)
all: $(SUBDIRS)

$(SUBDIRS): 
	$(MAKE) -C $@

.PHONY: serve
serve:
	$(MAKE) -C checker serve

.PHONY: check
check:
	$(MAKE) -C checker check
