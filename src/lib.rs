//! # ssb-uri-rs
//!
//! Utilities for recognising and converting Secure Scuttlebutt (SSB) URIs.
//!
//! Written according to the [SSB URI Specification](https://github.com/ssb-ngi-pointer/ssb-uri-spec).
//!
//! ## Example
//!
//!```
//! use anyhow::Result;
//! use ssb_uri_rs;
//!
//! fn example() -> Result<()> {
//!     let example_uri = "ssb:message/sha256/g3hPVPDEO1Aj_uPl0-J2NlhFB2bbFLIHlty-YuqFZ3w=";
//!
//!     assert!(ssb_uri_rs::is_classic_msg_uri(example_uri)?);
//!
//!     let example_sigil = ssb_uri_rs::msg_uri_to_sigil(example_uri)?;
//!
//!     assert_eq!(example_sigil, "%g3hPVPDEO1Aj/uPl0+J2NlhFB2bbFLIHlty+YuqFZ3w=.sha256");
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Documentation
//!
//! Use `cargo doc` to generate and serve the Rust documentation for this library:
//!
//! ```bash
//! git clone git@github.com:ssb-ngi-pointer/ssb-uri-rs.git
//! cd ssb-uri-rs
//! cargo doc --no-deps --open
//! ```
//!
//! ## Related Work
//!
//! [ssb-uri2](https://github.com/staltz/ssb-uri2) - TypeScript version of this library
//! [multiserver](https://github.com/ssb-js/multiserver) - JavaScript module
//! [multiserver-address](https://github.com/ssbc/multiserver-address) - JavaScript module
//!
//! ## License
//!
//! LGPL-3.0.
use anyhow::{anyhow, Result};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use regex::Regex;
use url::Url;

/* HELPER FUNCTIONS */

/// Replace all instances of the URL-safe character set with the standard equivalent.
pub fn safe_to_unsafe_base64(input: &str) -> String {
    input.replace("-", "+").replace("_", "/")
}

/// Replace all instance of the standard character set with the URL-safe equivalent.
pub fn unsafe_to_safe_base64(input: &str) -> String {
    input.replace("+", "-").replace("/", "_")
}

/// Extract and return the base64 data from a URI pathname.
pub fn extract_base64_data(pathname: &str) -> Result<Option<String>> {
    let re = Regex::new(r#"(:|/)([\w_\-=]+)$"#)?;
    // `caps` will be `None` if no capture is found (hence `Option` in return type)
    let caps = re.captures(pathname);
    let last_portion = caps.map(|caps| caps[2].to_string());

    match last_portion {
        Some(data) => Ok(Some(safe_to_unsafe_base64(&data))),
        None => Ok(None),
    }
}

/* SSB URI TYPE AND FORMAT CHECKING FUNCTIONS */

/// Ensure a URI is formatted according to the specification for the given `type` and `format`.
pub fn check_type_format(uri: &str, uri_type: &str, uri_format: &str) -> Result<bool> {
    let parsed_uri = Url::parse(uri)?;
    if uri.starts_with(&format!("ssb:{}:{}:", uri_type, uri_format))
        || uri.starts_with(&format!("ssb:{}/{}/", uri_type, uri_format))
        || uri.starts_with(&format!("ssb://{}/{}/", uri_type, uri_format))
            && extract_base64_data(parsed_uri.path())?.is_some()
    {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Check whether the given URI is a classic feed URI.
pub fn is_classic_feed_uri(uri: &str) -> Result<bool> {
    check_type_format(uri, "feed", "ed25519")
}

/// Check whether the given URI is a Bendy Butt feed URI.
pub fn is_bendy_butt_v1_feed_uri(uri: &str) -> Result<bool> {
    check_type_format(uri, "feed", "bendybutt-v1")
}

/// Check whether the given URI is a Gabby Grove feed URI.
pub fn is_gabby_grove_v1_feed_uri(uri: &str) -> Result<bool> {
    check_type_format(uri, "feed", "gabbygrove-v1")
}

/// Check whether the given URI is a classic message URI.
pub fn is_classic_msg_uri(uri: &str) -> Result<bool> {
    check_type_format(uri, "message", "sha256")
}

/// Check whether the given URI is a Bendy Butt message URI.
pub fn is_bendy_butt_v1_msg_uri(uri: &str) -> Result<bool> {
    check_type_format(uri, "message", "bendybutt-v1")
}

/// Check whether the given URI is a Gabby Grove message URI.
pub fn is_gabby_grove_v1_msg_uri(uri: &str) -> Result<bool> {
    check_type_format(uri, "message", "gabbygrove-v1")
}

/// Check whether the given URI is a blob URI.
pub fn is_blob_uri(uri: &str) -> Result<bool> {
    check_type_format(uri, "blob", "sha256")
}

/// Check whether the given URI is a multiserver address URI.
pub fn is_multiserver_uri(uri: &str) -> Result<bool> {
    let parsed_uri = Url::parse(uri)?;
    let query = parsed_uri
        .query()
        // convert the `Option` returned from `query()` into a `Result`
        // this allows the use of the `?` operator to unwrap the query string
        .ok_or_else(|| anyhow!("uri does not include a query string: {}", uri))?;

    if uri.starts_with("ssb:address:multiserver")
        || uri.starts_with("ssb:address/multiserver")
        || uri.starts_with("ssb://address/multiserver") && query.starts_with("multiserverAddress")
    {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Check whether the given URI is an encryption key (box2 Diffie-Hellman) URI.
pub fn is_encryption_key_box2_uri(uri: &str) -> Result<bool> {
    check_type_format(uri, "encryption-key", "box-dm-dh")
}

/// Check whether the given URI is an identity PO-box URI.
pub fn is_identity_po_box_uri(uri: &str) -> Result<bool> {
    check_type_format(uri, "identity", "po-box")
}

/// Check whether the given URI is an experimental URI.
pub fn is_experimental_uri(uri: &str) -> Result<bool> {
    if uri.starts_with("ssb:experimental") || uri.starts_with("ssb://experimental") {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Check whether the given URI matches any of the SSB URI specifications.
pub fn is_ssb_uri(uri: &str) -> Result<bool> {
    if is_classic_feed_uri(uri)?
        || is_bendy_butt_v1_feed_uri(uri)?
        || is_gabby_grove_v1_feed_uri(uri)?
        || is_classic_msg_uri(uri)?
        || is_bendy_butt_v1_msg_uri(uri)?
        || is_gabby_grove_v1_msg_uri(uri)?
        || is_blob_uri(uri)?
        || is_multiserver_uri(uri)?
        || is_encryption_key_box2_uri(uri)?
        || is_identity_po_box_uri(uri)?
        || is_experimental_uri(uri)?
    {
        Ok(true)
    } else {
        Ok(false)
    }
}

/* TODO: fix this
pub fn is_experimental_uri_with_action(uri: &str, action: &str) -> Result<bool> {
    let parsed_uri = Url::parse(uri)?;
    let query = parsed_uri
        .query()
        .ok_or(anyhow!("uri does not include a query string: {}", uri))?;

    let query_pairs = parsed_uri.query_pairs().into_owned();

    if is_experimental_uri(uri)? == true && query_pairs.find(|x| x == &("action".to_string(), action.to_string()).is_some() {

        Ok(true)
    } else {
        Ok(false)
    }
}
*/

/* SSB URI CONVERSION FUNCTIONS */

/// Convert a classic feed URI to a sigil-based (`@`) `ed25519` identifier.
pub fn feed_uri_to_sigil(uri: &str) -> Result<String> {
    match is_classic_feed_uri(uri)? {
        true => {
            let parsed_uri = Url::parse(uri)?;
            let base64_data = extract_base64_data(parsed_uri.path())?;
            match base64_data {
                Some(data) => Ok(format!("@{}.ed25519", data)),
                None => Err(anyhow!("failed to extract base64 data from uri: {}", uri)),
            }
        }
        false => Err(anyhow!(
            "uri is not type `feed` and format `ed25519`: {}",
            uri
        )),
    }
}

/// Convert a classic message URI to a sigil-based (`%`) `sha256` identifier.
pub fn msg_uri_to_sigil(uri: &str) -> Result<String> {
    match is_classic_msg_uri(uri)? {
        true => {
            let parsed_uri = Url::parse(uri)?;
            let base64_data = extract_base64_data(parsed_uri.path())?;
            match base64_data {
                Some(data) => Ok(format!("%{}.sha256", data)),
                None => Err(anyhow!("unable to extract base64 data from uri: {}", uri)),
            }
        }
        false => Err(anyhow!(
            "uri is not type `message` and format `sha256`: {}",
            uri
        )),
    }
}

/// Convert a blob URI to a sigil-based (`&`) `sha256` identifier.
pub fn blob_uri_to_sigil(uri: &str) -> Result<String> {
    match is_blob_uri(uri)? {
        true => {
            let parsed_uri = Url::parse(uri)?;
            let base64_data = extract_base64_data(parsed_uri.path())?;
            match base64_data {
                Some(data) => Ok(format!("&{}.sha256", data)),
                None => Err(anyhow!("unable to extract base64 data from uri: {}", uri)),
            }
        }
        false => Err(anyhow!(
            "uri is not of type `blob` and format `sha256`: {}",
            uri
        )),
    }
}

/// Convert a multiserver address URI to a multiserver address.
pub fn multiserver_uri_to_address(uri: &str) -> Result<String> {
    let parsed_uri = Url::parse(uri)?;
    let query = parsed_uri
        .query()
        .ok_or_else(|| anyhow!("uri does not include a query string: {}", uri))?;

    match query.starts_with("multiserverAddress") {
        true => Ok(query.to_string()),
        false => Err(anyhow!(
            "uri query string does not start with `multiserverAddress`: {}",
            uri
        )),
    }
}

/* SIGIL CONVERSION FUNCTIONS */

/// Convert a sigil-based (`@`) `ed25519` feed identifier to a URI.
pub fn feed_sigil_to_uri(sigil: &str) -> Result<String> {
    let data = &sigil
        .strip_suffix(".ed25519")
        .ok_or_else(|| anyhow!("feed sigil reference has an invalid suffix: {}", sigil))?;
    // ignore the prefix ('sigil') and perform base64 conversion
    let base64_data = unsafe_to_safe_base64(&data[1..]);
    Ok(format!("ssb:feed/ed25519/{}", base64_data))
}

/// Convert a sigil-based (`%`) `sha256` message identifier to a URI.
pub fn msg_sigil_to_uri(sigil: &str) -> Result<String> {
    let data = &sigil
        .strip_suffix(".sha256")
        .ok_or_else(|| anyhow!("message sigil reference has an invalid suffix: {}", sigil))?;
    let base64_data = unsafe_to_safe_base64(&data[1..]);
    Ok(format!("ssb:message/sha256/{}", base64_data))
}

/// Convert a sigil-based (`&`) `sha256` blob identifier to a URI.
pub fn blob_sigil_to_uri(sigil: &str) -> Result<String> {
    let data = &sigil
        .strip_suffix(".sha256")
        .ok_or_else(|| anyhow!("blob sigil reference has an invalid suffix: {}", sigil))?;
    let base64_data = unsafe_to_safe_base64(&data[1..]);
    Ok(format!("ssb:blob/sha256/{}", base64_data))
}

/* MULTISERVER ADDRESS CONVERSION FUNCTION */

/// Convert a multiserver address to a URI.
pub fn multiserver_address_to_uri(ms_addr: &str) -> String {
    let encoded = utf8_percent_encode(ms_addr, NON_ALPHANUMERIC).to_string();
    format!("ssb:address/multiserver?multiserverAddress={}", encoded)
}

/* TODO: Validation */

/* TODO: Composition and decomposition */

#[cfg(test)]
mod tests {
    mod fixtures;

    use crate::tests::fixtures::{ADDRESS_URIS, BLOB_URIS, FEED_URIS, MSG_URIS};
    use crate::*;

    #[test]
    fn safe_to_unsafe() {
        let safe_uri = "g3hPVPDEO1Aj_uPl0-J2NlhFB2bbFLIHlty-YuqFZ3w=";
        let unsafe_uri = safe_to_unsafe_base64(safe_uri);
        assert_eq!(unsafe_uri, "g3hPVPDEO1Aj/uPl0+J2NlhFB2bbFLIHlty+YuqFZ3w=");
    }

    #[test]
    fn unsafe_to_safe() {
        let unsafe_uri = "g3hPVPDEO1Aj/uPl0+J2NlhFB2bbFLIHlty+YuqFZ3w=";
        let safe_uri = unsafe_to_safe_base64(unsafe_uri);
        assert_eq!(safe_uri, "g3hPVPDEO1Aj_uPl0-J2NlhFB2bbFLIHlty-YuqFZ3w=");
    }

    #[test]
    fn extract_data() {
        let pathname = "ssb:feed/bendybutt-v1/APaWWDs8g73EZFUMfW37RBULtFEjwKNbDczvdYiRXtA=";
        let data = extract_base64_data(pathname);
        if let Ok(Some(string)) = data {
            assert_eq!(string, "APaWWDs8g73EZFUMfW37RBULtFEjwKNbDczvdYiRXtA=");
        }
    }

    #[test]
    fn type_format_checks() {
        let uris = [
            "ssb:message/sha256/g3hPVPDEO1Aj_uPl0-J2NlhFB2bbFLIHlty-YuqFZ3w=",
            "ssb:feed/bendybutt-v1/APaWWDs8g73EZFUMfW37RBULtFEjwKNbDczvdYiRXtA=",
            "ssb:blob:sha256:sbBmsB7XWvmIzkBzreYcuzPpLtpeCMDIs6n_OJGSC1U=",
        ];
        let types = ["message", "feed", "blob"];
        let formats = ["sha256", "bendybutt-v1", "sha256"];
        for i in 1..3 {
            let result = check_type_format(uris[i], types[i], formats[i]);
            if let Ok(boolean) = result {
                assert_eq!(boolean, true);
            }
        }
    }

    #[test]
    fn msg_uris_recognised() {
        let ssb_result = is_ssb_uri(MSG_URIS[1].1);
        assert!(ssb_result.is_ok());
        assert_eq!(ssb_result.unwrap(), true);

        for i in 1..3 {
            let result = is_classic_msg_uri(MSG_URIS[i].1);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), true)
        }

        let bb_result = is_bendy_butt_v1_msg_uri(MSG_URIS[4].1);
        assert!(bb_result.is_ok());
        assert_eq!(bb_result.unwrap(), true);

        let gg_result = is_gabby_grove_v1_msg_uri(MSG_URIS[5].1);
        assert!(gg_result.is_ok());
        assert_eq!(gg_result.unwrap(), true);
    }

    #[test]
    fn msg_from_sigil_to_uri() {
        let uri = msg_sigil_to_uri(MSG_URIS[0].1);
        assert!(uri.is_ok());
        assert_eq!(uri.unwrap(), MSG_URIS[1].1);
    }

    #[test]
    fn msg_from_uri_to_sigil() {
        let sigil = msg_uri_to_sigil(MSG_URIS[1].1);
        assert!(sigil.is_ok());
        assert_eq!(sigil.unwrap(), MSG_URIS[0].1);
    }

    #[test]
    fn feed_uris_recognised() {
        let ssb_result = is_ssb_uri(FEED_URIS[1].1);
        assert!(ssb_result.is_ok());
        assert_eq!(ssb_result.unwrap(), true);

        for i in 1..3 {
            let result = is_classic_feed_uri(FEED_URIS[i].1);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), true)
        }

        let bb_result = is_bendy_butt_v1_feed_uri(FEED_URIS[4].1);
        assert!(bb_result.is_ok());
        assert_eq!(bb_result.unwrap(), true);

        let gg_result = is_gabby_grove_v1_feed_uri(FEED_URIS[5].1);
        assert!(gg_result.is_ok());
        assert_eq!(gg_result.unwrap(), true);
    }

    #[test]
    fn feed_from_sigil_to_uri() {
        let uri = feed_sigil_to_uri(FEED_URIS[0].1);
        assert!(uri.is_ok());
        assert_eq!(uri.unwrap(), FEED_URIS[1].1);
    }

    #[test]
    fn feed_from_uri_to_sigil() {
        let sigil = feed_uri_to_sigil(FEED_URIS[1].1);
        assert!(sigil.is_ok());
        assert_eq!(sigil.unwrap(), FEED_URIS[0].1);
    }

    #[test]
    fn blob_uris_recognised() {
        let ssb_result = is_ssb_uri(BLOB_URIS[1].1);
        assert!(ssb_result.is_ok());
        assert_eq!(ssb_result.unwrap(), true);

        for i in 1..3 {
            let result = is_blob_uri(BLOB_URIS[i].1);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), true)
        }
    }

    #[test]
    fn blob_from_sigil_to_uri() {
        let uri = blob_sigil_to_uri(BLOB_URIS[0].1);
        assert!(uri.is_ok());
        assert_eq!(uri.unwrap(), BLOB_URIS[1].1);
    }

    #[test]
    fn blob_from_uri_to_sigil() {
        let sigil = blob_uri_to_sigil(BLOB_URIS[1].1);
        assert!(sigil.is_ok());
        assert_eq!(sigil.unwrap(), BLOB_URIS[0].1);
    }

    #[test]
    fn address_uris_recognised() {
        let ssb_result = is_ssb_uri(ADDRESS_URIS[1].1);
        assert!(ssb_result.is_ok());
        assert_eq!(ssb_result.unwrap(), true);

        for i in 1..3 {
            let result = is_multiserver_uri(ADDRESS_URIS[i].1);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), true)
        }
    }

    /*
    // TODO: make these pass

    #[test]
    fn multiserver_addr_to_uri() {
        let uri = multiserver_address_to_uri(ADDRESS_URIS[0].1);
        assert_eq!(uri, ADDRESS_URIS[1].1);
    }

    #[test]
    fn multiserver_uri_to_addr() {
        let sigil = multiserver_uri_to_address(ADDRESS_URIS[1].1);
        assert!(sigil.is_ok());
        assert_eq!(sigil.unwrap(), ADDRESS_URIS[0].1);
    }
    */
}