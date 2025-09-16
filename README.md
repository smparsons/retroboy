<p align="center">
  <img src="images/logo.png" width="175px" height="175px">
  <br />
  Retro Boy is a simple Game Boy emulator written in Rust that can be played on the web. <a href="https://smparsons.github.io/retroboy">Try it here.</a>
</p>

## Introduction

Retro Boy is a cycle-accurate Game Boy emulator written in Rust. It uses `wasm-pack` to translate the Rust code into WebAssembly so it can be played on the web. The web frontend then uses Web Audio API and HTML Canvas for audio and graphics. It also leverages the browser's local storage to persist cartridge RAM data for battery-backed MBC cartridges.

## Features

- Accurate CPU that passes all [JSON CPU tests](https://github.com/adtennant/GameboyCPUTests)
- Accurate audio emulation
- Graphics emulation built using a scanline-based renderer
- MBC1, MBC3, MBC5, and HuC1 support
- RTC support for MBC3 cartridges
- Cartridge RAM that persists to browser local storage for battery-backed cartridges
- Support for GameShark or GameGenie cheats
- Basic support for save states
- A web frontend that supports:
  - Fullscreen mode
  - Pausing/resuming
  - Selectable monochrome or color modes
  - Customizable key map for game controls
  - Management and enabling/disabling of game cheat codes
  - A mobile-friendy responsive design

## How to Compile to WebAssembly

To compile the implementation to WebAssembly, you will first need to install wasm-pack with the command `cargo install wasm-pack` if you haven't done so already. Then, run `make build-wasm` to build the core project and generate the Javascript binding code in the web frontend directory.

## Web Frontend

The web frontend for this emulator is a React/TypeScript app designed with Material UI. It is located in the frontends/web folder. The UI provides the ability to load a ROM as well as play, pause, or reset the emulator. It also provides a fullscreen mode.

To run the web frontend:

1. Compile the Rust code to WebAssembly and generate the Javascript binding code as described in the "How to Compile to WebAssembly" section.
2. When the binding code is generated, it will be added to the frontends/web/src/core directory.
3. Run `yarn install` in the frontends/web directory to install all dependencies.
4. Run `yarn start` in the same directory to run the application locally.

## SDL Frontend (Test App)

I created an alternative frontend using the sdl2 library that allows using the emulator without having to go to the web
app. It's purpose is to make testing a little simpler. The Makefile has different commands to startup the test app:

```
# Run commands
$ make sdl-run                             # Run with file dialog to pick game
$ make sdl-run ROM=path/to/rom.gb          # Run with specific ROM
$ make sdl-run ROM=path/to/rom.gb CGB=1    # Run with ROM in CGB mode
$ make sdl-run-cgb ROM=path/to/rom.gb      # Same as above CGB command

# Watch commands
$ make sdl-watch                           # Watch with file dialog to pick game
$ make sdl-watch ROM=path/to/rom.gb        # Watch with specific ROM
$ make sdl-watch ROM=path/to/rom.gb CGB=1  # Watch in CGB mode
$ make sdl-watch-cgb ROM=path/to/rom.gb    # Same as above CGB command
```

You can pass your ROM filepath to the command if you'd like, otherwise the app will display a file dialog where you can pick the game you want to select.

The watch commands allow faster debugging, because when you change one of the Rust files, it will automatically re-build and re-run the test app. If you want to use a watach command, I'd recommend passing a specific ROM path, otherwise after every rebuild it will show a file dialog and ask you to pick a game.

Note that if you are using a Mac and you install the SDL library on your machine via `brew`, you may have problems with the linker when you try to build and run the SDL frontend. Exporting the following paths may help you to solve the problem (I put these in my .zshenv file in my home folder):

```
export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH"
export CPATH="/opt/homebrew/include:$CPATH"
```

I have not tried running the test app on a Windows machine or a Linux machine, so I am not sure what kinds of problems could happen there.

The control mappings for the test app are as follows:

| Keyboard Key | Game Boy Button |
| ------------ | --------------- |
| Arrow Keys   | D-Pad           |
| Z            | A               |
| X            | B               |
| Enter        | Start           |
| Space        | Select          |

## Screenshots

<p float="left">
  <img src="images/screenshots/pokemon-red.png" width="240" />
  <img src="images/screenshots/pokemon-crystal.png" width="240" />
  <img src="images/screenshots/yugioh-dds.png" width="240" />
  <img src="images/screenshots/marble-madness.png" width="240" />
  <img src="images/screenshots/pacman.png" width="240" />
  <img src="images/screenshots/super-mario-land.png" width="240" />
  <img src="images/screenshots/tetris.png" width="240" />
  <img src="images/screenshots/kirby.png" width="240" />
  <img src="images/screenshots/links-awakening.png" width="240">
</p>

## Test ROMs

This emulator passes the following test suites from Blargg's test ROM collection:

1. [CPU instruction tests](https://github.com/retrio/gb-test-roms/tree/master/cpu_instrs)
2. [CPU instruction timing tests](https://github.com/retrio/gb-test-roms/tree/master/instr_timing)
3. [Memory timing tests](https://github.com/retrio/gb-test-roms/tree/master/mem_timing)
4. [Memory timing tests 2](https://github.com/retrio/gb-test-roms/tree/master/mem_timing-2)
5. [Interrupt timing tests (CGB)](https://github.com/retrio/gb-test-roms/tree/master/interrupt_time)
6. [APU tests (DMG)](https://github.com/retrio/gb-test-roms/tree/master/dmg_sound)
7. [APU tests (CGB)](https://github.com/retrio/gb-test-roms/tree/master/cgb_sound)

Additionally, this emulator passes all [JSON CPU tests](https://github.com/adtennant/GameboyCPUTests), and
only some tests from the [Mooneye test ROM collection](https://github.com/Gekkio/mooneye-test-suite).

## Test Suite

This project holds a fairly extensive test suite, as the bulk of the logic was designed using a TDD approach. There are a lot of tests that exercise CPU opcodes, and basic tests that exercise the GPU. Run `cargo test` to run the test suite.

## Helpful Resources

For convenience, here is a list of the resources I used to build this emulator:

1. [Gameboy CPU Manual](http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf)
2. [Pan Docs](https://gbdev.io/pandocs/)
3. [Blargg's Test ROM Collection](https://github.com/retrio/gb-test-roms)
4. [Gameboy Doctor](https://github.com/robert/gameboy-doctor)
5. [Imran Nazar's Gameboy Emulator Tutorial](https://imrannazar.com/series/gameboy-emulation-in-javascript)
