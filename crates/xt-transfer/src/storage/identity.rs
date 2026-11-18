use sha2::{Digest, Sha256};
use url::Url;
use xt_error::{Error, ErrorKind, Result};

pub fn asset_key(value: &str) -> Result<String> {
    let mut url =
        Url::parse(value).map_err(|_| Error::new(ErrorKind::InvalidArgument, "Invalid asset URL"))?;
    let mut query: Vec<_> = url
        .query_pairs()
        .filter(|(name, _)| {
            let name = name.to_ascii_lowercase();
            !name.starts_with("x-amz-")
                && ![
                    "signature",
                    "expires",
                    "awsaccesskeyid",
                    "token",
                    "access_token",
                ]
                .contains(&name.as_str())
        })
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    query.sort();
    url.set_query(None);
    url.set_fragment(None);
    if !query.is_empty() {
        url.query_pairs_mut().extend_pairs(query);
    }
    Ok(hex::encode(Sha256::digest(url.as_str().as_bytes())))
}
