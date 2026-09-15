# RUST-INDUSTRIAL-INFRASTRUCTURE
# Zero-Copy Industrial Modbus Protocol Gateway (Rust)

## Executive Overview
A high-throughput, memory-safe industrial IoT gateway written in **Rust (1.70+)**. It implements **zero-copy deserialization** of binary Modbus TCP MBAP headers and PDU frames using lifetime-bound byte slices (`&'a [u8]`), strictly avoiding heap allocations, and validates packet integrity with a hardware-accelerated **CRC-16** algorithm.

## Zero-Copy Deserialization Pipeline

```mermaid
graph LR
    A[Raw Network Socket Buffer &[u8]] --> B[Lifetime-Bound Slice Parser]
    B --> C[ModbusTcpFrame<'a>]
    C --> D[Zero-Allocation Header Inspection]
    C --> E[CRC-16 Modbus Verification]
    D & E --> F[Sub-Microsecond Dispatch to PLC Workers]
```

### Source Tree
- **`src/main.rs`**: Core Rust implementation featuring zero-copy parser, CRC16 calculation, and embedded unit tests.
- **`Cargo.toml`**: Cargo package manifest.
- **`runner/run.js`**: Numerical verification harness validating frame decoding outputs.

## Frame Parsing & CRC-16 Formulation
MBAP Header format (7 bytes) followed by Function Code and PDU:
- Transaction ID: `u16` (Big Endian)
- Protocol ID: `u16` (0 for Modbus)
- Length: `u16`
- Unit ID: `u8`
- Function Code: `u8`

CRC-16 polynomial: `0xA001` (reversed standard `0x8005`).

## Native Cargo Compilation & Unit Tests
```bash
cargo test
cargo run --release
```

## Universal Verification
```bash
node runner/run.js
node orchestrator/run.js --project=33-rust
```

## Senior Interview Q&A
- **Q: What does zero-copy mean in this context?** The parsed `ModbusTcpFrame<'a>` struct contains a payload slice `&'a [u8]` that points directly to the underlying socket memory buffer. Zero bytes are copied to the heap, eliminating allocator contention and maximizing cache efficiency.
- **Q: How does Rust enforce safety without a garbage collector?** Lifetime annotations (`'a`) guarantee at compile time that the `ModbusTcpFrame` cannot outlive the underlying network buffer, preventing use-after-free bugs and dangling pointers.\n
