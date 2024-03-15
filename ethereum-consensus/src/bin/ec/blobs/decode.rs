use crate::blobs::{
    framing::{payload_from_sized, Mode as Framing},
    Blob, Error, BITS_PER_FIELD_ELEMENT, BYTES_PER_BLOB, BYTES_PER_FIELD_ELEMENT,
};
use bitvec::prelude::*;
use std::io::{Read, Write};

pub fn unpack_from_blobs(blobs: &[Blob]) -> Result<Vec<u8>, Error> {
    let mut stream = vec![0u8; blobs.len() * BYTES_PER_BLOB];
    let stream_bits = stream.view_bits_mut::<Lsb0>();

    let mut i = 0;
    for blob in blobs {
        let blob_bits = blob.as_ref().view_bits::<Lsb0>();
        // chunks of serialized field element bits
        let mut chunks = blob_bits.chunks_exact(8 * BYTES_PER_FIELD_ELEMENT);
        for chunk in chunks.by_ref() {
            // only grab the first bits for a field element
            let src = &chunk[..BITS_PER_FIELD_ELEMENT];
            stream_bits[i * BITS_PER_FIELD_ELEMENT..(i + 1) * BITS_PER_FIELD_ELEMENT]
                .copy_from_bitslice(src);
            i += 1;
        }

        let remainder = chunks.remainder();
        assert!(remainder.is_empty());
    }

    Ok(stream)
}

// Expects a `Vec<Blob>` with `serde_json` encoding read from `reader`.
// Writes recovered byte stream to `writer`.
pub fn to_writer_from_json(
    reader: impl Read,
    mut writer: impl Write,
    framing: Framing,
) -> Result<(), Error> {
    let blobs: Vec<Blob> = serde_json::from_reader(reader)?;
    let result = unpack_from_blobs(&blobs)?;
    let result = match framing {
        Framing::Raw => result,
        Framing::Sized => payload_from_sized(result),
    };
    writer.write_all(&result)?;
    Ok(())
}
