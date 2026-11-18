use crate::domain::Checksum;
use sha2::Digest;
use std::fs;
use std::io::Read;
use std::path::Path;
use xt_error::{DownloadError, Error, ErrorKind, Result};

pub(crate) fn validate_checksum(checksum: Option<&Checksum>) -> Result<()> {
    if let Some(checksum) = checksum {
        let length = match checksum
            .algorithm
            .to_ascii_lowercase()
            .replace('-', "")
            .as_str()
        {
            "md5" => 32,
            "sha1" => 40,
            "sha256" => 64,
            "sha512" => 128,
            _ => return Err(Error::new(ErrorKind::InvalidArgument, "Unsupported checksum algorithm")),
        };
        if checksum.value.len() != length || !checksum.value.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "Checksum must be a hexadecimal string of the correct length",
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify(path: &Path, checksum: Option<&Checksum>) -> Result<()> {
    let mut file = fs::File::open(path)?;
    let mut first = [0u8; 512];
    let count = file.read(&mut first)?;
    let prefix = String::from_utf8_lossy(&first[..count])
        .trim_start()
        .to_ascii_lowercase();
    if prefix.starts_with("<!doctype html") || prefix.starts_with("<html") {
        return Err(Error::new(
            ErrorKind::Download(DownloadError::ChecksumMismatch),
            "Received HTML; not publishing it as a data file",
        ));
    }
    let Some(checksum) = checksum else {
        return Ok(());
    };
    use std::io::Seek;
    use std::io::SeekFrom;
    file.seek(SeekFrom::Start(0))?;
    fn hash<D: Digest>(mut file: fs::File) -> Result<String> {
        let mut digest = D::new();
        let mut buffer = [0u8; 128 * 1024];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            digest.update(&buffer[..n]);
        }
        Ok(hex::encode(digest.finalize()))
    }
    let value = match checksum
        .algorithm
        .to_ascii_lowercase()
        .replace('-', "")
        .as_str()
    {
        "md5" => hash::<md5::Md5>(file)?,
        "sha1" => hash::<sha1::Sha1>(file)?,
        "sha256" => hash_sha256(file)?,
        "sha512" => hash::<sha2::Sha512>(file)?,
        _ => unreachable!(),
    };
    if !value.eq_ignore_ascii_case(&checksum.value) {
        return Err(Error::new(
            ErrorKind::Download(DownloadError::ChecksumMismatch),
            "File checksum mismatch",
        ));
    }
    Ok(())
}

fn hash_sha256(mut file: fs::File) -> Result<String> {
    let size = file.metadata()?.len();
    let mut digest = hya_net::stream_digest::StreamDigest::new(0);
    let mut buffer = [0u8; 128 * 1024];
    let mut offset = 0;
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        digest.write(offset, &buffer[..n]);
        offset += n as u64;
    }
    digest.finish(size).ok_or_else(|| {
        Error::new(
            ErrorKind::Download(DownloadError::Incomplete),
            "Hya cannot verify the complete file digest",
        )
    })
}
