use fjall::{Database, KeyspaceCreateOptions};
use fjall_typed::{
    codec::{FacetJson, Str, U8},
    Keyspace,
};

fn main() {
    let db = Database::builder("example_typed_keyspaces.fjall")
        .open()
        .unwrap();
    let ks = db
        .keyspace("my_items", KeyspaceCreateOptions::default)
        .unwrap();
    let ks = Keyspace::<U8, Str>::new(ks);

    ks.insert(&33, "hello").unwrap();
    ks.remap_value::<FacetJson<String>>()
        .insert(&45, &String::from("hello"))
        .unwrap();
    // ks.insert("a", "hello").unwrap();
}
