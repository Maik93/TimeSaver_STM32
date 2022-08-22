# STM32F7x7 Rust example
Base setup can be obtained from `rust-embedded/cortex-m-quickstart` template:
```bash
cargo generate rust-embedded/cortex-m-quickstart
```

## Dependencies
```bash
rustup target add thumbv7em-none-eabihf
cargo install cargo-embed
```

## Build and upload
Project can be build with `cargo build` and uploaded with `cargo embed`.
Serial RTT channels can be opened with `cargo embed --config with_rtt`, but somehow it does not work on Clion terminals.

Debugging is currently unknown.
[Debugonomicon](https://docs.rust-embedded.org/debugonomicon/), `cargo-embed` GDB and [semihosting](https://docs.rust-embedded.org/cortex-m-quickstart/cortex_m_semihosting/index.html) can be an entrypoint for this.

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

You can then debug the target board through VSCode, thanks to `.vscode/launch.json` file.
Further details on how to edit this file can be found in the [official documentation](https://probe.rs/docs/tools/vscode/).

## References
- [cortex-m-quickstart](https://github.com/rust-embedded/cortex-m-quickstart) starting template.
- [Cargo-embed](https://probe.rs/docs/tools/cargo-embed/) documentation for upload and serial debug.
- `memory.x` files from https://github.com/stm32-rs/stm32f7xx-hal.
- [stm32f7xx-hal examples](https://github.com/stm32-rs/stm32f7xx-hal/tree/main/examples)
- [rtt_target](https://docs.rs/rtt-target/latest/rtt_target/) Crate: target side implementation of the Real-Time Transfer I/O protocol.
- [probe-rs-debugger plugin for VS-Code](https://probe.rs/docs/tools/vscode/)
