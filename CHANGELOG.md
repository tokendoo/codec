# Changelog

## 0.1.0

First crates.io release of the Tokendoo JSON codec.

### Wire-format stability

The current wire format is **T3**. The wire format is **not stable** before 1.0:
it will change. Compressed payloads must not be persisted beyond the crate
version that produced them. A `0.x` bump is a breaking change for both the Rust
API and the on-wire encoding.
