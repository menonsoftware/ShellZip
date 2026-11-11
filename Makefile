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
PKG_SHELL := shellzip-shell
PKG_INTEGRATE := shellzip-integrate

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
	release-cli release-gui release-sfx release-shell \
	run run-release gui gui-release explore \
	test test-cli test-core test-sfx build-sfx \
	check clippy lint fmt fmt-check doc deny \
	clean ci install install-gui install-shell \
	gen-file-icons bench velopack-stage velopack-pack deb msix msix-uninstall help

# Velopack channel for this host (win / osx / linux).
ifeq ($(OS),Windows_NT)
VELOPACK_CHANNEL ?= win
else
UNAME_S := $(shell uname -s 2>/dev/null || echo Linux)
ifeq ($(UNAME_S),Darwin)
VELOPACK_CHANNEL ?= osx
else
VELOPACK_CHANNEL ?= linux
endif
endif

all: build

build:
	$(CARGO) build --workspace $(CARGO_FLAGS)

# CLI + desktop + size-optimized SFX stub (minisize, not --release).
release: release-cli release-gui release-sfx release-shell

release-shell:
ifeq ($(OS),Windows_NT)
	$(CARGO) build --release -p $(PKG_SHELL) $(CARGO_FLAGS)
else
	@echo skip shellzip-shell on non-Windows
endif

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
deny:
	$(CARGO) deny check

ci: fmt-check clippy test deny
	$(CARGO) clippy -p $(PKG_CORE) --no-default-features --features sevenz --all-targets -- -D warnings

install:
	$(CARGO) install --path crates/$(PKG_CLI) --locked --force

install-gui:
	$(CARGO) install --path crates/$(PKG_GUI) --locked --force

# Register file associations, icons, and shell menus for the current user.
install-shell: install-gui
ifeq ($(OS),Windows_NT)
	$(CARGO) build --release -p $(PKG_SHELL) $(CARGO_FLAGS)
	powershell -NoProfile -Command "$$bin = Join-Path $$env:USERPROFILE '.cargo\\bin'; New-Item -ItemType Directory -Force -Path $$bin | Out-Null; Copy-Item -Force 'target/release/shellzip_shell.dll' (Join-Path $$bin 'shellzip_shell.dll')"
endif
	shellzip --integrate

gen-file-icons:
	$(CARGO) run -p xtask -- gen-file-icons

# Release timings for ZIP threads, plus 7z/Rar when they are on PATH.
bench:
	$(CARGO) run --release -p xtask -- bench

# Stage GUI/CLI/SFX (+ shell DLL on Windows) and icons for `vpk pack`.
velopack-stage: release
	$(CARGO) run -p xtask -- velopack-stage $(VELOPACK_CHANNEL)

# Package a Velopack release (requires .NET 8 + `dotnet tool install -g vpk`).
# Windows: set CERTUM_EMAIL + ssign, or CERTUM_THUMBPRINT, to Authenticode-sign.
# Linux: set GPG_PRIVATE_KEY / GPG_KEY_ID to detach-sign after pack.
velopack-pack: velopack-stage
	$(CARGO) run -p xtask -- velopack-pack $(VELOPACK_CHANNEL)

# Linux-only: .deb with GUI, CLI, SFX, desktop file, icons, and menus.
deb: release
ifeq ($(OS),Windows_NT)
	@echo deb is Linux-only
else ifeq ($(UNAME_S),Darwin)
	@echo deb is Linux-only
else
	$(CARGO) run -p xtask -- deb-pack
endif

# Windows-only: release binaries, pack MSIX, self-sign with a local
# developer cert, and sideload with Add-AppxPackage. CI calls
# `xtask msix-pack` without --install so Partner Center can re-sign.
msix: release
ifeq ($(OS),Windows_NT)
	$(CARGO) run -p xtask -- msix-pack --install
else
	@echo msix is Windows-only
endif

# Windows-only: clear classic Explorer registration and remove the sideloaded
# ShellZip Appx package (Settings uninstall uses the cleanup scheduled task).
msix-uninstall:
ifeq ($(OS),Windows_NT)
	$(CARGO) run -p xtask -- msix-uninstall
else
	@echo msix-uninstall is Windows-only
endif

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
	@echo   make deny             cargo deny check
	@echo   make ci               fmt-check + clippy + test + deny
	@echo   make install          cargo install szip
	@echo   make install-gui      cargo install shellzip
	@echo   make install-shell    install-gui + register associations
	@echo   make gen-file-icons   Rasterize grx/shellzip_file_icons
	@echo   make release-shell    Windows Explorer shell-extension DLL
	@echo   make bench            Release ZIP timings into docs/BENCH.md
	@echo   make velopack-stage   Stage binaries + icons under dist/velopack/
	@echo   make velopack-pack    Stage then run vpk pack (GPG when env set)
	@echo   make deb              Linux: pack dist/deb/shellzip_<ver>_amd64.deb
	@echo   make msix             Windows: pack, self-sign, and sideload Store MSIX
	@echo   make msix-uninstall   Windows: clear classic menu keys and remove Appx
	@echo   make clean            cargo clean
