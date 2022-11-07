# TimeSaver project for STM32F7x7 boards
Basically, this is an implementation for a cooking timer, where an encoder is used to set desired minutes and an internal counter waits this given time, until an alarm is displayed.

More technically, this is a Rust embedded project that deals with a 16x2 character LCD, an encoder (handled with external interrupts) and a timer interrupt that dictate system events.
A simple state machine goes through an initial splash screen, then the counter setup, the actual countdown and finally the alarm.

## Dependencies
Required target is `thumbv7em-none-eabihf`, nightly. It is automatically set up thourgh `rust-toolchain.toml`.

```bash
cargo install cargo-embed
```

## Build and upload
Project can be build with `cargo build`, or build and uploaded with `cargo embed`.
Serial RTT channels can be opened with `cargo embed --config with_rtt`. Note that somehow it does not work on Clion terminals.

## Debug
Breakpoint debug can be achieved with a VSCode plugin, `probe-rs-debugger`.
Install it with:
```bash
# VSC plugin (check for new releases here: https://github.com/probe-rs/vscode/releases)
wget https://github.com/probe-rs/vscode/releases/download/v0.4.0/probe-rs-debugger-0.4.0.vsix
code --install-extension probe-rs-debugger-0.3.3.vsix

# debugger server
cargo install --force --git https://github.com/probe-rs/probe-rs probe-rs-debugger
```

The target board can be then debugged through VSCode, thanks to `.vscode/launch.json` file.
Further details on how to edit this file can be found in the [official documentation](https://probe.rs/docs/tools/vscode/).

## References
Base project configuration for STM32F7x7 boards can be obtained from `rust-embedded/cortex-m-quickstart` template:
```bash
cargo generate rust-embedded/cortex-m-quickstart
```
Main references used to achive this project:
- [cortex-m-quickstart](https://github.com/rust-embedded/cortex-m-quickstart) starting template.
- [Cargo-embed](https://probe.rs/docs/tools/cargo-embed/) documentation for upload and serial debug.
- `memory.x` files from https://github.com/stm32-rs/stm32f7xx-hal.
- [stm32f7xx-hal examples](https://github.com/stm32-rs/stm32f7xx-hal/tree/main/examples)
- [rtt_target](https://docs.rs/rtt-target/latest/rtt_target/) Crate: target side implementation of the Real-Time Transfer I/O protocol.
- [probe-rs-debugger plugin for VS-Code](https://probe.rs/docs/tools/vscode/)
