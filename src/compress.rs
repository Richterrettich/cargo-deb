use std::ops;
use crate::error::*;

pub enum Compressed {
    Gz(Vec<u8>),
    Xz(Vec<u8>),
}

impl ops::Deref for Compressed {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Gz(data) |
            Self::Xz(data) => &data,
        }
    }
}

impl Compressed {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Gz(_) => "gz",
            Self::Xz(_) => "xz",
        }
    }
}

/// Compresses data using the [native Rust implementation of Zopfli](https://github.com/carols10cents/zopfli).
#[cfg(not(feature = "lzma"))]
pub fn xz_or_gz(data: &[u8], _fast: bool) -> CDResult<Compressed> {
    use zopfli::{self, Format, Options};

    // Compressed data is typically half to a third the original size
    let mut compressed = Vec::with_capacity(data.len() >> 1);
    zopfli::compress(&Options::default(), &Format::Gzip, data, &mut compressed)?;

    Ok(Compressed::Gz(compressed))
}

/// Compresses data using the xz2 library
#[cfg(feature = "lzma")]
pub fn xz_or_gz(data: &[u8], fast: bool) -> CDResult<Compressed> {
    use std::io::Write;
    use xz2::stream;
    use xz2::write::XzEncoder;

    // Compressed data is typically half to a third the original size
    let buf = Vec::with_capacity(data.len() >> 1);

    // Compression level 6 is a good trade off between size and [ridiculously] long compression time
    let encoder = stream::MtStreamBuilder::new()
        .threads(num_cpus::get() as u32)
        .preset(if fast { 1 } else { 6 })
        .encoder()
        .map_err(|e| CargoDebError::LzmaCompressionError(e))?;

    let mut writer = XzEncoder::new_stream(buf, encoder);
    writer.write_all(data).map_err(|e| CargoDebError::Io(e))?;

    let compressed = writer.finish().map_err(|e| CargoDebError::Io(e))?;

    Ok(Compressed::Xz(compressed))
}
