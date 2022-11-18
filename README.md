# Pillow® Water Quality Aggregator

Created for ECE 198 and to learn new things! 

This repository is licensed under the GPLv3. (Copy and you get Policy 71'd!)

## Usage

**This crate uses features from Rust nightly! Make sure you switch before running:**

```
rustup toolchain install nightly
rustup default nightly
rustup target add thumbv7m-none-eabi
``` 

Cargo and STLink are required to build and flash the program:

```
cd ece198
cargo install cargo-flash
cargo install --path .
cargo flash --chip STM32F401RETx
```
Pin layout is described in `main.rs` and you can probably figure out the hardware used from that. 
