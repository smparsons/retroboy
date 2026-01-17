# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RetroBoy is a cycle-accurate Game Boy (DMG) and Game Boy Color (CGB) emulator written in Rust. The core emulator compiles to WebAssembly for web deployment and also has a native SDL frontend for testing.

## Build Commands

```bash
# Core library
make build           # Build core emulator library
cargo test           # Run core emulator tests
make check           # Run cargo check on all components

# WASM build (requires wasm-pack: cargo install wasm-pack)
make build-wasm      # Build WASM and generate JS bindings in frontends/web/src/core/

# SDL frontend (for local testing)
make sdl-run                          # Run with file dialog
make sdl-run ROM=path/to/rom.gb       # Run specific ROM
make sdl-run ROM=path/to/rom.gb CGB=1 # Run in CGB mode
make sdl-watch ROM=path/to/rom.gb     # Auto-rebuild on changes

# Web frontend (after build-wasm)
cd frontends/web && yarn install
cd frontends/web && yarn start        # Dev server
cd frontends/web && yarn build        # Production build
```

## Architecture

### Core Emulator (src/)

The emulator follows a component-based architecture reflecting real Game Boy hardware:

- **Emulator** (`emulator.rs`): Top-level entry point that owns the CPU and provides the public API
- **CPU** (`cpu.rs`, `cpu/`): Z80-variant processor with registers, interrupts, and instruction execution
  - `opcodes.rs`: Main opcode dispatch and execution
  - `microops.rs`: Low-level CPU operations (memory access, timing)
  - `alu.rs`, `bitops.rs`, `loads.rs`, `jumps.rs`: Instruction implementations by category
  - `timers.rs`, `interrupts.rs`: Timer and interrupt handling
- **AddressBus** (`address_bus.rs`, `address_bus/`): Memory mapping and hardware register access
  - `cartridge.rs`: ROM loading and cartridge type detection
  - `mbc1.rs`, `mbc3.rs`, `mbc5.rs`, `huc1.rs`: Memory Bank Controller implementations
  - `dma.rs`, `hdma.rs`: DMA transfer implementations
  - `cheats.rs`: GameShark/GameGenie cheat code support
- **GPU** (`gpu.rs`, `gpu/`): Scanline-based graphics rendering
  - `background.rs`, `window.rs`, `sprites.rs`: Layer rendering
  - `palettes.rs`: DMG and CGB palette handling
  - `prioritization.rs`: Sprite/background priority resolution
- **APU** (`apu.rs`, `apu/`): Audio processing with four channels
  - `pulse.rs`: Two pulse wave channels
  - `wave.rs`: Programmable wave channel
  - `noise.rs`: Noise channel
  - `envelope.rs`, `sweep.rs`, `length.rs`: Audio parameter modules
- **Joypad** (`joypad.rs`): Input handling
- **Serial** (`serial.rs`): Serial port emulation
- **BIOS** (`bios.rs`): Boot ROM handling

### Serialization System

The project uses a custom `Serializable` trait with a derive macro (`tools/serializable_derive/`) for save states. Components implement `serialize`/`deserialize` methods for binary state persistence.

### Frontends

- **Web** (`frontends/web/`): React/TypeScript app using Material UI, Web Audio API, and HTML Canvas
- **SDL** (`frontends/sdl/`): Native test frontend using SDL2 for quick iteration
- **JSON Test Runner** (`frontends/json_test_runner/`): Runs CPU tests from JSON test suites

### WASM Bindings (src/wasm/)

- `api.rs`: Main WASM API exposed to JavaScript
- `emulator_settings.rs`, `rom_metadata.rs`, `save_state_result.rs`: Data transfer types

## Key Design Patterns

- **Cycle-accurate timing**: The CPU's `step()` function advances one instruction with proper timing propagated through all components
- **Audio sync**: `step_until_next_audio_buffer()` runs until the APU buffer is full, providing the main emulation loop timing
- **Callback-based rendering**: The GPU takes a renderer callback `fn(&[u8])` for frame output
- **CGB mode**: Toggled via `set_cgb_mode()`, affects memory banking, palettes, and speed switching

## Mac SDL Setup

If using Homebrew-installed SDL2, you may need these environment variables:
```bash
export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH"
export CPATH="/opt/homebrew/include:$CPATH"
```
