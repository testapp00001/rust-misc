# Module 11: Serialization

Master the Serde ecosystem, custom serialization, format design, and schema evolution.

## Lesson Index

1. **p01_serde_deep_dive.rs** - Serialize/Deserialize traits, data model, attributes, container attributes
2. **p02_custom_serde.rs** - Custom serialize/deserialize, Visitor pattern, DeserializeSeed, stateful deser
3. **p03_json_mastery.rs** - serde_json, Value, streaming, pretty printing, arbitrary precision
4. **p04_binary_formats.rs** - bincode, MessagePack, postcard, compact encoding, zero-copy
5. **p05_schema_evolution.rs** - Versioning formats, forward/backward compatibility, serde defaults
6. **p06_serde_performance.rs** - Zero-copy deserialization, `&str` vs String, avoiding allocations, simd-json
7. **p07_validation.rs** - serde_with, custom validators, pre-deserialization checks, validator crate
8. **p08_format_design.rs** - Designing your own format, custom content, tagged enums, adjacently tagged
9. **p09_database_serde.rs** - Serde for database types, SQLx FromRow, custom database serialization
10. **p10_serde_testing.rs** - Round-trip testing, fuzzing serialized data, snapshot testing serialization

## Key Concepts

- Serde separates data format from data structure through a data model
- Custom implementations give full control over wire format and performance
- Schema evolution requires careful use of defaults, aliases, and version tags
- Zero-copy deserialization can dramatically improve performance for large payloads
