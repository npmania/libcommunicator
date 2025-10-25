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

See `include/communicator.h` for the C API.

Basic usage from C:

```c
#include "communicator.h"
#include <stdio.h>

int main() {
    char* greeting = communicator_greet("World");
    if (greeting) {
        printf("%s\n", greeting);
        communicator_free_string(greeting);
    }
    return 0;
}
```
