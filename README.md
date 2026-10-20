# nucc-container

Rust middleware for NUCC XFBIN containers and typed chunks.

```text
nucc-core       shared chunk types and index records
nucc-xfbin      XFBIN header, index, chunk table, read/write
nucc-nus3       NUS3BANK reader
nucc-chunk      typed nuccChunk* payloads
nucc-container  high-level NuccXfbinDocument
nucc-cli        inspect and list XFBIN contents
```

## Usage

```toml
[dependencies]
nucc-container = { path = "crates/nucc-container" }
```

```rust
use nucc_container::NuccXfbinDocument;

let document = NuccXfbinDocument::parse(&bytes)?;
for chunk in &document.chunks {
    println!("{} {}", chunk.chunk_type(), chunk.chunk_name());
}
```

## CLI

```text
nucc inspect file.xfbin
nucc chunks file.xfbin
```

## Scope

This repository owns the XFBIN container and NUCC chunk envelope. It does not
contain game assets. Leaf formats such as NUD, NUT, NSH, and NFH are read by
the separate `nu` library. `nucc-nus3` handles the NUS3BANK payload carried by
`nuccChunkNub`.

## License

MIT.
