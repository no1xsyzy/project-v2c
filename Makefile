SUBDIRS := $(wildcard v2c-*/.)

.PHONY: all $(SUBDIRS)
all: $(SUBDIRS)

$(SUBDIRS): 
	$(MAKE) -C $@
