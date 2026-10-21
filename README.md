# nucc-container

Rust middleware for NUCC XFBIN containers and typed chunks.

```text
nucc-core       shared chunk types and index records
nucc-xfbin      XFBIN header, index, chunk table, read/write
nucc-nus3       NUS3BANK reader
nucc-acb        ACB / CRI UTF reader
nucc-awb        AWB / AFS2 wave bank reader
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
nucc acb file.xfbin
nucc acb BGM.acb
nucc awb BGM.awb
```

## Supported chunks

| Chunk type | Source |
|---|---|
| `nuccChunkNull` | [nucc_chunk_null.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_null.rs) |
| `nuccChunkPage` | [nucc_chunk_page.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_page.rs) |
| `nuccChunkBinary` | [nucc_chunk_binary.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_binary.rs) |
| `nuccChunkAnm` | [nucc_chunk_anm.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_anm.rs) |
| `nuccChunkAnmStrm` | [nucc_chunk_anmstrm.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_anmstrm.rs) |
| `nuccChunkAnmStrmFrame` | [nucc_chunk_anmstrmframe.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_anmstrmframe.rs) |
| `nuccChunkParticle` | [nucc_chunk_particle.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_particle.rs) |
| `nuccChunkCamera` | [nucc_chunk_camera.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_camera.rs) |
| `nuccChunkAmbient` | [nucc_chunk_ambient.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_ambient.rs) |
| `nuccChunkBillboard` | [nucc_chunk_billboard.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_billboard.rs) |
| `nuccChunkClump` | [nucc_chunk_clump.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_clump.rs) |
| `nuccChunkCoord` | [nucc_chunk_coord.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_coord.rs) |
| `nuccChunkDynamics` | [nucc_chunk_dynamics.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_dynamics.rs) |
| `nuccChunkLayerSet` | [nucc_chunk_layerset.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_layerset.rs) |
| `nuccChunkLightDirc` | [nucc_chunk_lightdirc.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_lightdirc.rs) |
| `nuccChunkLightPoint` | [nucc_chunk_lightpoint.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_lightpoint.rs) |
| `nuccChunkLightSet` | [nucc_chunk_lightset.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_lightset.rs) |
| `nuccChunkMaterial` | [nucc_chunk_material.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_material.rs) |
| `nuccChunkModel` | [nucc_chunk_model.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_model.rs) |
| `nuccChunkModelHit` | [nucc_chunk_modelhit.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_modelhit.rs) |
| `nuccChunkModelPrimitiveBatch` | [nucc_chunk_modelprimitivebatch.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_modelprimitivebatch.rs) |
| `nuccChunkModelVertex` | [nucc_chunk_modelvertex.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_modelvertex.rs) |
| `nuccChunkMorphModel` | [nucc_chunk_morphmodel.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_morphmodel.rs) |
| `nuccChunkMorphPrimitive` | [nucc_chunk_morphprimitive.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_morphprimitive.rs) |
| `nuccChunkNub` | [nucc_chunk_nub.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_nub.rs) |
| `nuccChunkPostFilter` | [nucc_chunk_postfilter.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_postfilter.rs) |
| `nuccChunkPrimitiveVertex` | [nucc_chunk_primitivevertex.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_primitivevertex.rs) |
| `nuccChunkTexture` | [nucc_chunk_texture.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_texture.rs) |
| `nuccChunkTrail` | [nucc_chunk_trail.rs](crates/nucc-chunk/src/nucc_chunk/nucc_chunk_trail.rs) |

`nuccChunkIndex` is the XFBIN chunk-table entry and is handled by the
container index rather than a leaf chunk parser. Unsupported leaf types remain
`nuccChunkFont`, `nuccChunkSprite`, `nuccChunkSprite2`, `nuccChunkSpriteAnm`,
and `nuccChunkSprite2Anm`.

## License

MIT.
