# AMPS Rust FFI

Rust FFI bindings for the [AMPS](https://crankuptheamps.com) (Advanced Message Processing System) C++ client library.

## Prerequisites

- **Rust 1.70+** with Cargo
- **C++ compiler** (Clang 10+, GCC 9+, or MSVC 2019+)
- **CMake 3.16+**
- **AMPS C++ Client library** v5.3.5.1 — download from https://crankuptheamps.com

## Setup

```bash
# 1. Clone the repo
git clone <repo-url>
cd amps-rust-ffi

# 2. Place the AMPS C++ client in amps-client/
#    (extract so that amps-client/include/amps/ampsplusplus.hpp exists)
tar -xzf amps-c++-client-5.3.5.1-*.tar.gz
mv amps-c++-client-5.3.5.1-* amps-client

# 3. Build the C++ wrapper
mkdir -p c-wrapper/build && cd c-wrapper/build
cmake ..
make
cd ../..

# 4. Build the Rust library
cargo build

# 5. Run tests
cargo test
```

## Project Structure

```
amps-rust-ffi/
├── amps-client/            # AMPS C++ client (not checked in)
├── c-wrapper/
│   ├── include/amps_ffi.h  # C-compatible FFI header
│   ├── src/amps_ffi.cpp    # C++ wrapper implementation
│   ├── tests/              # C header compilation tests
│   └── CMakeLists.txt
├── src/
│   ├── lib.rs              # Library root
│   ├── ffi/                # Auto-generated bindgen bindings
│   ├── client.rs           # Safe Client wrapper
│   ├── error.rs            # Error types
│   ├── message.rs          # Message type
│   └── subscription.rs     # Subscription handling
├── tests/
│   └── docker/             # AMPS server for integration tests
├── PLAN.md                 # Implementation plan & checklist
├── AGENTS.md               # AI agent instructions
└── wiggum.sh               # Automated build loop
```

## Automated Development Loop (Wiggum)

`wiggum.sh` runs an AI agent ([Kimi CLI](https://github.com/MoonshotAI/kimi-cli)) in a loop to automatically complete phases from `PLAN.md`.

### Prerequisites

Install Kimi CLI:

```bash
curl -LsSf https://code.kimi.com/install.sh | bash
kimi   # run once to configure API key via /login
```

### Usage

```bash
# Run until all PLAN.md phases are complete
./wiggum.sh

# Run at most 5 iterations
./wiggum.sh --max 5

# Custom delay between iterations (default: 2s)
./wiggum.sh --delay 10
```

### How It Works

1. Checks `PLAN.md` for unchecked tasks (`- [ ]`)
2. Runs `kimi -p "start" -y --print` (non-interactive, auto-approve)
3. Kimi reads `AGENTS.md` → `PLAN.md`, picks up the next incomplete phase
4. Writes code, runs tests, commits on success
5. Repeats until all phases are done or `--max` is reached

## Plan & Progress

See [PLAN.md](PLAN.md) for the full architecture, implementation checklist, and findings.

## License

See [LICENSE](LICENSE) for details.
