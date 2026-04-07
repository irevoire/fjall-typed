use fjall::Slice;
use roaring::{RoaringBitmap, RoaringTreemap};

use crate::codec::{Decode, Encode};

/// Encode a roaring bitmap with [the standard on-disk format](https://github.com/RoaringBitmap/RoaringFormatSpec).
pub enum RoaringBitmapCodec {}

impl Encode for RoaringBitmapCodec {
    type Item = RoaringBitmap;
    type Error = std::io::Error;

    fn encode(item: &Self::Item) -> Result<Slice, Self::Error> {
        let mut bytes = Vec::new();
        item.serialize_into(&mut bytes)?;
        Ok(bytes.into())
    }
}

impl Decode for RoaringBitmapCodec {
    type Item = RoaringBitmap;
    type Error = std::io::Error;

    fn decode(bytes: Slice) -> Result<Self::Item, Self::Error> {
        RoaringBitmap::deserialize_from(&mut bytes.as_ref())
    }
}

/// Encode a roaring treemap with [the standard on-disk format](https://github.com/RoaringBitmap/RoaringFormatSpec).
pub enum RoaringTreemapCodec {}

impl Encode for RoaringTreemapCodec {
    type Item = RoaringTreemap;
    type Error = std::io::Error;

    fn encode(item: &Self::Item) -> Result<Slice, Self::Error> {
        let mut bytes = Vec::new();
        item.serialize_into(&mut bytes)?;
        Ok(bytes.into())
    }
}

impl Decode for RoaringTreemapCodec {
    type Item = RoaringTreemap;
    type Error = std::io::Error;

    fn decode(bytes: Slice) -> Result<Self::Item, Self::Error> {
        RoaringTreemap::deserialize_from(&mut bytes.as_ref())
    }
}
