/// Represents a parsed AT-URI following the syntax:
/// AT-URI = "at://" AUTHORITY [ "/" COLLECTION [ "/" RKEY ] ]
#[derive(Debug, Clone)]
pub(crate) struct AtUri {
    /// The AUTHORITY component (handle or DID) from the AT-URI
    pub(crate) authority: String,
    /// The COLLECTION component (NSID)
    pub(crate) collection: Option<String>,
    /// The RKEY component (record key)
    pub(crate) rkey: Option<String>,
}

pub(crate) fn validate_aturi<S: Into<String>>(aturi: S) -> Option<AtUri> {
    let aturi = aturi.into();
    let aturi = aturi.trim();

    let aturi = if let Some(trimmed) = aturi.strip_prefix("web+") {
        trimmed
    } else {
        aturi
    };

    let stripped = aturi.strip_prefix("at://");
    if stripped.is_none() {
        return None;
    }

    let stripped = stripped.unwrap();

    let parts = stripped.split('/').collect::<Vec<&str>>();

    if !parts.is_empty() && !is_valid_authority(parts[0]) {
        return None;
    }
    if parts.len() > 1 && !is_valid_nsid(parts[1]) {
        return None;
    }
    if parts.len() > 3 {
        return None;
    }

    return Some(AtUri {
        authority: parts[0].to_string(),
        collection: parts.get(1).map(|s| s.to_string()),
        rkey: parts.get(2).map(|s| s.to_string()),
    });
}

pub(crate) fn is_valid_nsid(nsid: &str) -> bool {
    fn is_valid_char(byte: u8) -> bool {
        byte.is_ascii_lowercase()
            || byte.is_ascii_uppercase()
            || byte.is_ascii_digit()
            || byte == b'-'
            || byte == b'.'
    }
    !(nsid.bytes().any(|byte| !is_valid_char(byte))
        || nsid.split('.').count() < 3
        || nsid.split('.').any(|label| {
            label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-')
        })
        || nsid.is_empty()
        || nsid.len() > 253)
}

pub(crate) fn is_valid_hostname(hostname: &str) -> bool {
    fn is_valid_char(byte: u8) -> bool {
        byte.is_ascii_lowercase()
            || byte.is_ascii_uppercase()
            || byte.is_ascii_digit()
            || byte == b'-'
            || byte == b'.'
    }
    !(hostname.ends_with(".localhost")
        || hostname.ends_with(".internal")
        || hostname.ends_with(".arpa")
        || hostname.ends_with(".local")
        || hostname.bytes().any(|byte| !is_valid_char(byte))
        || hostname.split('.').any(|label| {
            label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-')
        })
        || hostname.is_empty()
        || hostname.len() > 253)
}

enum InputType {
    Handle(String),
    Plc(String),
    Web(String),
}

/// Validates the AUTHORITY component of an AT-URI.
/// The AUTHORITY can be a handle or a DID (did:plc: or did:web:).
pub(crate) fn is_valid_authority(authority: &str) -> bool {
    let authority_type = if authority.starts_with("did:web:") {
        InputType::Web(authority.to_string())
    } else if authority.starts_with("did:plc:") {
        InputType::Plc(authority.to_string())
    } else {
        InputType::Handle(authority.to_string())
    };

    match authority_type {
        InputType::Handle(handle) => is_valid_hostname(&handle) && handle.chars().any(|c| c == '.'),
        InputType::Plc(did) => did
            .strip_prefix("did:plc:")
            .is_some_and(|remaining| remaining.len() == 24),
        InputType::Web(did) => {
            let parts = did
                .strip_prefix("did:web:")
                .map(|trimmed| trimmed.split(":").collect::<Vec<&str>>());

            parts.is_some_and(|inner_parts| {
                !inner_parts.is_empty()
                    && inner_parts.first().is_some_and(|hostname| {
                        is_valid_hostname(hostname) && hostname.chars().any(|c| c == '.')
                    })
            })
        }
    }
}
