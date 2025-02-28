use crate::error::Result;

#[cfg(feature = "io_ipc_compression")]
#[cfg_attr(docsrs, doc(cfg(feature = "io_ipc_compression")))]
pub fn decompress_lz4(input_buf: &[u8], output_buf: &mut [u8]) -> Result<()> {
    use std::io::Read;
    let mut decoder = lz4::Decoder::new(input_buf)?;
    decoder.read_exact(output_buf).map_err(|e| e.into())
}

#[cfg(feature = "io_ipc_compression")]
#[cfg_attr(docsrs, doc(cfg(feature = "io_ipc_compression")))]
pub fn decompress_zstd(input_buf: &[u8], output_buf: &mut [u8]) -> Result<()> {
    use std::io::Read;
    let mut decoder = zstd::Decoder::new(input_buf)?;
    decoder.read_exact(output_buf).map_err(|e| e.into())
}

#[cfg(not(feature = "io_ipc_compression"))]
pub fn decompress_lz4(_input_buf: &[u8], _output_buf: &mut [u8]) -> Result<()> {
    use crate::error::ArrowError;
    Err(ArrowError::Ipc("The crate was compiled without IPC compression. Use `io_ipc_compression` to read compressed IPC.".to_string()))
}

#[cfg(not(feature = "io_ipc_compression"))]
pub fn decompress_zstd(_input_buf: &[u8], _output_buf: &mut [u8]) -> Result<()> {
    use crate::error::ArrowError;
    Err(ArrowError::Ipc("The crate was compiled without IPC compression. Use `io_ipc_compression` to read compressed IPC.".to_string()))
}

#[cfg(feature = "io_ipc_compression")]
#[cfg_attr(docsrs, doc(cfg(feature = "io_ipc_compression")))]
pub fn compress_lz4(input_buf: &[u8], output_buf: &mut Vec<u8>) -> Result<()> {
    use std::io::Write;
    let mut encoder = lz4::EncoderBuilder::new().build(output_buf).unwrap();
    encoder.write_all(input_buf)?;
    encoder.finish().1.map_err(|e| e.into())
}

#[cfg(feature = "io_ipc_compression")]
#[cfg_attr(docsrs, doc(cfg(feature = "io_ipc_compression")))]
pub fn compress_zstd(input_buf: &[u8], output_buf: &mut Vec<u8>) -> Result<()> {
    use std::io::Write;
    let mut encoder = zstd::Encoder::new(output_buf, 17)?.auto_finish();
    encoder.write_all(input_buf).map_err(|e| e.into())
}

#[cfg(not(feature = "io_ipc_compression"))]
pub fn compress_lz4(_input_buf: &[u8], _output_buf: &mut Vec<u8>) -> Result<()> {
    use crate::error::ArrowError;
    Err(ArrowError::Ipc("The crate was compiled without IPC compression. Use `io_ipc_compression` to write compressed IPC.".to_string()))
}

#[cfg(not(feature = "io_ipc_compression"))]
pub fn compress_zstd(_input_buf: &[u8], _output_buf: &mut Vec<u8>) -> Result<()> {
    use crate::error::ArrowError;
    Err(ArrowError::Ipc("The crate was compiled without IPC compression. Use `io_ipc_compression` to write compressed IPC.".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "io_ipc_compression")]
    #[test]
    fn round_trip_zstd() {
        let data: Vec<u8> = (0..200u8).map(|x| x % 10).collect();
        let mut buffer = vec![];
        compress_zstd(&data, &mut buffer).unwrap();

        let mut result = vec![0; 200];
        decompress_zstd(&buffer, &mut result).unwrap();
        assert_eq!(data, result);
    }

    #[cfg(feature = "io_ipc_compression")]
    #[test]
    fn round_trip_lz4() {
        let data: Vec<u8> = (0..200u8).map(|x| x % 10).collect();
        let mut buffer = vec![];
        compress_lz4(&data, &mut buffer).unwrap();

        let mut result = vec![0; 200];
        decompress_lz4(&buffer, &mut result).unwrap();
        assert_eq!(data, result);
    }
}
