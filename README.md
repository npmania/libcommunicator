# libcommunicator

A Rust library for unified communication across multiple chat platforms.

## Building

```bash
cargo build --release
```

The compiled library will be in `target/release/`:
- Linux: `libcommunicator.so`
- macOS: `libcommunicator.dylib`
- Windows: `communicator.dll`

## Testing

```bash
cargo test
```

## FFI Example

See `include/communicator.h` for the C API and `examples/` directory for usage examples.
