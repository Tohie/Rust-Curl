RUSTC ?= rustc
RUSTFLAGS ?=
RUSTDOC ?= rustdoc

all: curlcrate

curlcrate:
	$(RUSTC) $(RUSTFLAGS) src/curl/lib.rs
	mv libcurl* lib/

examples: curlcrate
	$(RUSTC) -L lib $(RUSTFLAGS) src/examples/*

doc:
	$(RUSTDOC) src/curl/lib.rs

clean:
	rm lib/*
	rm -r doc
