# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
.PHONY: develop test clean

ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))../..

develop:
	cd "$(CURDIR)" && maturin develop --release --manifest-path Cargo.toml

test:
	cd "$(CURDIR)" && python3 -m unittest discover -s tests -v

clean:
	rm -rf "$(CURDIR)/target" "$(CURDIR)/../target"
