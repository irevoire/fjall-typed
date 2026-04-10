use fjall::{Database, KeyspaceCreateOptions};
use fjall_typed::{
    codec::{FacetJson, Str, U128, U8},
    Keyspace,
};

fn main() {
    let db = Database::builder("example_typed_keyspaces.fjall")
        .open()
        .unwrap();
    let ks = db
        .keyspace("my_items", KeyspaceCreateOptions::default)
        .unwrap();
    // Here we wrap the original keyspace into a fjall_typed keyspace.
    // This one indicates that it maps `u8` with strings.
    let ks = Keyspace::<U8, Str>::new(ks);

    ks.insert(&33, "hello").unwrap();

    // The whole keyspace doesn't have to follow these types for all its keys and values.
    // You can also remap the value or key.
    ks.remap_value::<FacetJson<String>>()
        .insert(&45, &String::from("hello"))
        .unwrap();

    // When using a number larger than 1 bytes, you have to define in which order the bytes will be layed down via the `byteorder` crate.
    ks.remap_key::<U128<byteorder::BigEndian>>()
        .insert(&u128::MAX, &String::from("hello"))
        .unwrap();

    // This wouldn't work as "a" is not a `u8`.
    // ks.insert("a", "hello").unwrap();

    // You can still call all the normal "non-typed" method as usual because the fjall_typed::Keyspace deref to a fjall::Keyspace.
    let len = ks.len().unwrap();
    println!("len: {len}");
}
