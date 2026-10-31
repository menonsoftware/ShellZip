# ShellZip — cargo wrappers for the workspace.
#   make              debug build
#   make release      CLI + GUI + size-optimized SFX
#   make run ARGS="l archive.zip"
#   make gui
#   make ci
#
# Extra cargo flags:  make build CARGO_FLAGS="--offline"

CARGO       ?= cargo
CARGO_FLAGS ?=
ARGS        ?=

BIN_CLI := szip
BIN_GUI := shellzip
PKG_CLI := shellzip-cli
PKG_GUI := shellzip-gui
PKG_CORE := shellzip-core
PKG_SFX := shellzip-sfx
BIN_SFX := shellzip-sfx

# `make release` leaves szip, shellzip, and the size-optimized stub together.
ifeq ($(OS),Windows_NT)
COPY_RELEASE_SFX = powershell -NoProfile -Command "New-Item -ItemType Directory -Force -Path 'target/release' | Out-Null; Copy-Item -Force 'target/minisize/shellzip-sfx.exe' 'target/release/shellzip-sfx.exe'"
else
COPY_RELEASE_SFX = mkdir -p target/release && cp -f target/minisize/shellzip-sfx target/release/shellzip-sfx
endif

ifeq ($(OS),Windows_NT)
# Relative paths make Explorer open the user profile (often Documents); pass a full path from $(CURDIR).
OPEN_RELEASE = powershell -NoProfile -Command "$$p = Join-Path '$(CURDIR)' 'target/release'; $$dir = (New-Item -ItemType Directory -Force -Path $$p).FullName; explorer $$dir"
else
OPEN_RELEASE = mkdir -p "$(CURDIR)/target/release" && (xdg-open "$(CURDIR)/target/release" 2>/dev/null || open "$(CURDIR)/target/release")
endif

.PHONY: all build release build-cli build-gui build-core \
	release-cli release-gui release-sfx \
	run run-release gui gui-release explore \
	test test-cli test-core test-sfx build-sfx \
	check clippy lint fmt fmt-check doc \
	clean ci install install-gui bench help

all: build

build:
	$(CARGO) build --workspace $(CARGO_FLAGS)

# CLI + desktop + size-optimized SFX stub (minisize, not --release).
release: release-cli release-gui release-sfx

release-cli:
	$(CARGO) build --release -p $(PKG_CLI) $(CARGO_FLAGS)

release-gui:
	$(CARGO) build --release -p $(PKG_GUI) $(CARGO_FLAGS)

release-sfx: build-sfx
	$(COPY_RELEASE_SFX)

build-cli:
	$(CARGO) build -p $(PKG_CLI) $(CARGO_FLAGS)

build-gui:
	$(CARGO) build -p $(PKG_GUI) $(CARGO_FLAGS)

build-core:
	$(CARGO) build -p $(PKG_CORE) $(CARGO_FLAGS)

build-sfx:
	$(CARGO) build -p $(PKG_SFX) --profile minisize $(CARGO_FLAGS)

# CLI: make run ARGS="a out.zip src"   (defaults to --help)
run:
	$(CARGO) run --bin $(BIN_CLI) -- $(or $(ARGS),--help)

run-release:
	$(CARGO) run --release --bin $(BIN_CLI) -- $(or $(ARGS),--help)

# Desktop window
gui:
	$(CARGO) run --bin $(BIN_GUI) -- $(ARGS)

gui-release:
	$(CARGO) run --release --bin $(BIN_GUI) -- $(ARGS)

# Open target/release in the file manager (creates the folder if missing).
explore:
	$(OPEN_RELEASE)

test:
	$(CARGO) test --workspace $(CARGO_FLAGS)
	$(CARGO) test -p $(PKG_CORE) --features rar,sevenz-codecs,sevenz-parallel $(CARGO_FLAGS)

test-cli:
	$(CARGO) test -p $(PKG_CLI) $(CARGO_FLAGS)

test-core:
	$(CARGO) test -p $(PKG_CORE) $(CARGO_FLAGS)

test-sfx:
	$(CARGO) test -p $(PKG_SFX) $(CARGO_FLAGS)

check:
	$(CARGO) check --workspace --all-targets $(CARGO_FLAGS)

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

lint: fmt-check clippy

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

doc:
	$(CARGO) doc --workspace --no-deps --open

clean:
	$(CARGO) clean

# Same steps as .github/workflows/ci.yml
ci: fmt-check clippy test
	$(CARGO) clippy -p $(PKG_CORE) --no-default-features --features sevenz --all-targets -- -D warnings

install:
	$(CARGO) install --path crates/$(PKG_CLI) --locked --force

install-gui:
	$(CARGO) install --path crates/$(PKG_GUI) --locked --force

# Release timings for ZIP threads, plus 7z/Rar when they are on PATH.
bench:
	$(CARGO) run --release -p xtask -- bench

help:
	@echo ShellZip Makefile
	@echo   make / make build     Debug workspace build
	@echo   make release          Release CLI, GUI, and size-optimized SFX
	@echo   make release-cli      Release build of szip
	@echo   make release-gui      Release build of the desktop app
	@echo   make release-sfx      Size-optimized SFX stub, copied into target/release
	@echo   make build-cli        Build szip only
	@echo   make build-gui        Build the desktop app
	@echo   make build-sfx        Build the size-optimized SFX stub
	@echo   make test-sfx         Test the SFX stub
	@echo   make run              Run szip --help
	@echo   make run ARGS="..."   Run szip with arguments
	@echo   make gui              Open the ShellZip window
	@echo   make explore          Open target/release in the file manager
	@echo   make test             cargo test --workspace
	@echo   make check            Type-check all targets
	@echo   make clippy           Clippy with -D warnings
	@echo   make lint             fmt-check + clippy
	@echo   make fmt              rustfmt
	@echo   make doc              rustdoc and open it
	@echo   make ci               fmt-check + clippy + test
	@echo   make install          cargo install szip
	@echo   make install-gui      cargo install shellzip
	@echo   make bench            Release ZIP timings into docs/BENCH.md
	@echo   make clean            cargo clean
