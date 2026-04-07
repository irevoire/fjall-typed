## Fjall typed

This crate adds types to the excellent [fjall](https://github.com/fjall-rs/fjall/) key-value store.

Usage is the same as fjall, except you can now specify the types that should be stored in a `Keyspace`.
[See our examples](https://github.com/irevoire/fjall-typed/blob/main/examples)

## The `Codec` system

To "type" your keyspace you have to specify a codec that implements the `Encode` and `Decode` traits.
You can implement these traits yourself by specifying how to go from the `Slice` type of fjall to your type.

### Available codecs by default

| codec          | keyspace                                                             | example                             | remark                                                                                                                                                |
| -------------- | -------------------------------------------------------------------- | ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `U8` / `I8`    | `Keyspace<U8, I8>`                                                   | `ks.insert(&12, &-4)`               | You have to dereference all the values you're passing                                                                                                 |
| `U32` / `I128` | `Keyspace<U32<byteorder::LittleEndian>, I128<byteorder::BigEndian>>` | `ks.insert(&456324548, &i128::MIN)` | For numbers larger than one byte, you have to specify the endianess to use with the [`byteorder`](https://docs.rs/byteorder/latest/byteorder/) crate. |
| `SerdeJson`    | `Keyspace<U8, SerdeJson<MyCoolStruct>>`                              | `ks.insert(&31, &my_cool_struct)`   | The `SerdeJson` codec is generic over your struct and requires your structure to implement `Serialize`.                                               |

Other codecs include:

- `Bytes` to retrieve the raw bytes without any action from this crate.
- `U8`, `U16`, `U32`, `U64`, `U128`, `I8`, `I16`, `I32`, `I64` and `I128` to serialize and deserialize direct number. You have to specify the endianess through the [byteorder](https://docs.rs/byteorder/latest/byteorder/) crate.
- `Str` to write a string.
- `FacetJson`, `FacetMsgpack`, and `FacetPostcard` to write your structure in the specified format if it implements the `Facet` trait.
- `SerdeJson`, `SerdeMsgpack`, and `SerdePostcard` to write your structure in the specified format if it implements the `Serialize` and `Deserialize` traits.
- `RoaringBitmapCodec` and `RoaringTreemapCodec` to write optimized bitmaps from the [roaring-rs](https://github.com/RoaringBitmap/roaring-rs/) crate.
- `Lazy` lets you defer the deserialization until you know you need it. That's typically useful when iterating over key/value pairs and needing to check the key to determine whether the value is useful or can be skipped.
