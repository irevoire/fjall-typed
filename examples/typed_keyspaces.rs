use fjall::{Database, KeyspaceCreateOptions};
use fjall_typed_keyspace::{
    codec::{Str, U8},
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

    ks.insert(&42, "hello").unwrap();
    // ks.insert("a", "hello").unwrap();
}
