.PHONY: help sdl-run sdl-run-cgb sdl-watch sdl-watch-cgb screenshot clean check test

help:
	@echo "RetroBoy Emulator Development Commands:"
	@echo ""
	@echo "  make help           - Show this help message"
	@echo "  make build          - Build the core emulator library"
	@echo "  make build-wasm     - Build WASM version for web frontend"
	@echo "  make sdl-run        - Run SDL frontend once (shows file dialog)"
	@echo "  make sdl-run-cgb    - Run SDL frontend once in CGB mode"
	@echo "  make sdl-watch      - Auto-rebuild and run SDL frontend on changes"
	@echo "  make sdl-watch-cgb  - Auto-rebuild and run SDL frontend in CGB mode"
	@echo "  make screenshot     - Run headless and save screenshot to test_screenshots/"
	@echo "  make check          - Run cargo check on all components"
	@echo "  make test           - Run all tests"
	@echo "  make clean          - Clean all build artifacts"
	@echo ""
	@echo "SDL Examples:"
	@echo "  make sdl-run                              # Run with file dialog to pick game"
	@echo "  make sdl-run ROM=path/to/rom.gb           # Run with specific ROM"
	@echo "  make sdl-run ROM=path/to/rom.gb CGB=1     # Run with ROM in CGB mode"
	@echo "  make sdl-run-cgb ROM=path/to/rom.gb       # Same as above CGB command"
	@echo "  make sdl-watch                            # Watch with file dialog to pick game"
	@echo "  make sdl-watch ROM=path/to/rom.gb         # Watch with specific ROM"
	@echo "  make sdl-watch ROM=path/to/rom.gb CGB=1   # Watch with ROM in CGB mode"
	@echo "  make sdl-watch-cgb ROM=path/to/rom.gb     # Same as above CGB command"
	@echo ""
	@echo "Screenshot Examples:"
	@echo "  make screenshot ROM=game.gb SECONDS=5        # Run and capture screenshot"
	@echo "  make screenshot ROM=game.gbc SECONDS=5 CGB=1 # CGB mode"

CORE_SOURCES := $(shell find src -name "*.rs" 2>/dev/null)

target/debug/libretroboy.rlib: $(CORE_SOURCES) Cargo.toml
	@echo "🔨 Building core emulator library..."
	cargo build

build: target/debug/libretroboy.rlib

frontends/web/src/core/retroboyCore_bg.wasm: $(CORE_SOURCES) Cargo.toml
	@echo "🌐 Building WASM version for web frontend..."
	@echo "📁 Output: ./frontends/web/src/core/"
	wasm-pack build --target web --out-dir ./frontends/web/src/core --out-name retroboyCore --release

build-wasm: frontends/web/src/core/retroboyCore_bg.wasm

sdl-run:
	@echo "🎮 Running SDL frontend..."
ifdef ROM
ifdef CGB
	@echo "📁 ROM: $(ROM) (CGB mode)"
	cd frontends/sdl && cargo run -- "$(ROM)" --cgb
else
	@echo "📁 ROM: $(ROM)"
	cd frontends/sdl && cargo run -- "$(ROM)"
endif
else
ifdef CGB
	@echo "🎨 CGB mode enabled (file dialog will show)"
	cd frontends/sdl && cargo run -- --cgb
else
	@echo "📁 File dialog will show for ROM selection"
	cd frontends/sdl && cargo run
endif
endif

sdl-watch:
	@echo "🔄 Starting SDL frontend with auto-reload..."
	@echo "📁 Watching for changes in:"
	@echo "   • src/ (emulator core)"
	@echo "   • frontends/sdl/src/ (SDL frontend)"
	@echo ""
ifdef ROM
ifdef CGB
	@echo "💡 ROM: $(ROM) (CGB mode)"
	@echo "⏹️  Press Ctrl+C to stop watching"
	@echo ""
	cd frontends/sdl && cargo watch \
		-w ../../src \
		-w src \
		--clear \
		-x 'run -- "$(ROM)" --cgb'
else
	@echo "💡 ROM: $(ROM)"
	@echo "⏹️  Press Ctrl+C to stop watching"
	@echo ""
	cd frontends/sdl && cargo watch \
		-w ../../src \
		-w src \
		--clear \
		-x 'run -- "$(ROM)"'
endif
else
ifdef CGB
	@echo "💡 CGB mode enabled (file dialog will show)"
	@echo "⏹️  Press Ctrl+C to stop watching"
	@echo ""
	cd frontends/sdl && cargo watch \
		-w ../../src \
		-w src \
		--clear \
		-x 'run -- --cgb'
else
	@echo "💡 File dialog will show for ROM selection"
	@echo "⏹️  Press Ctrl+C to stop watching"
	@echo ""
	cd frontends/sdl && cargo watch \
		-w ../../src \
		-w src \
		--clear \
		-x 'run'
endif
endif

sdl-run-cgb:
	$(MAKE) sdl-run CGB=1

sdl-watch-cgb:
	$(MAKE) sdl-watch CGB=1

screenshot:
ifndef ROM
	$(error ROM is required. Usage: make screenshot ROM=path/to/rom.gb SECONDS=5)
endif
ifndef SECONDS
	$(error SECONDS is required. Usage: make screenshot ROM=path/to/rom.gb SECONDS=5)
endif
	@mkdir -p test_screenshots
	@echo "📸 Running headless frontend and capturing screenshot..."
	@echo "📁 ROM: $(ROM)"
	@echo "⏱️  Duration: $(SECONDS) seconds"
ifdef CGB
	@echo "🎨 CGB mode enabled"
	cargo run --release -p retroboy-headless -- "$(ROM)" $(SECONDS) --cgb --screenshot --screenshot-path test_screenshots
else
	cargo run --release -p retroboy-headless -- "$(ROM)" $(SECONDS) --screenshot --screenshot-path test_screenshots
endif

check:
	@echo "🔍 Running cargo check..."
	cargo check
	@echo "🎮 Checking SDL frontend..."
	cd frontends/sdl && cargo check

test:
	@echo "🧪 Running core emulator tests..."
	cargo test
	@echo "🎮 Running SDL frontend tests..."
	cd frontends/sdl && cargo test

clean:
	@echo "🧹 Cleaning core emulator..."
	cargo clean
	@echo "🎮 Cleaning SDL frontend..."
	cd frontends/sdl && cargo clean
	@echo "🌐 Cleaning WASM artifacts..."
	rm -rf frontends/web/src/core/