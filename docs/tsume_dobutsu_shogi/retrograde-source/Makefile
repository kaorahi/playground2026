SHELL := /bin/bash
.SHELLFLAGS := -e -o pipefail -c

RUSTC ?= rustc
PYTHON ?= python3
XZ ?= xz
.PHONY: all normal-all fairy-all verify clean
.DEFAULT_GOAL := all
.DELETE_ON_ERROR:

all: normal-11.bin.xz fairy-11.bin.xz

build/tsume-enum: src/tsume-enum.rs src/tsume-sample.rs
	@mkdir -p build
	$(RUSTC) --edition=2021 -O $< -o $@

build/tsume-analyze: src/tsume-analyze-checkpoint.rs src/tsume-reverse.rs src/tsume-sample.rs
	@mkdir -p build
	$(RUSTC) --edition=2021 -O $< -o $@

# An existing complete enumeration is reused on restart.
# Run `make clean` after modifying the rules or enumeration source.
data/reachable_1.txt.xz: | build/tsume-enum
	@mkdir -p data
	./build/tsume-enum | $(XZ) -T1 -1 > $@.pending
	$(XZ) -t $@.pending
	mv $@.pending $@

normal-%.bin.xz: data/reachable_1.txt.xz build/tsume-analyze run_analysis.py export_depth.py
	$(PYTHON) run_analysis.py normal '$*'

fairy-%.bin.xz: data/reachable_1.txt.xz build/tsume-analyze run_analysis.py export_depth.py
	$(PYTHON) run_analysis.py fairy '$*'

normal-all: data/reachable_1.txt.xz build/tsume-analyze run_analysis.py export_depth.py
	$(PYTHON) run_analysis.py normal all

fairy-all: data/reachable_1.txt.xz build/tsume-analyze run_analysis.py export_depth.py
	$(PYTHON) run_analysis.py fairy all

verify: all
	@for file in normal-*.bin.xz fairy-*.bin.xz; do $(XZ) -t "$$file"; done
	$(PYTHON) verify_outputs.py normal-*.bin.xz fairy-*.bin.xz

clean:
	rm -rf build data normal-*.bin.xz fairy-*.bin.xz
