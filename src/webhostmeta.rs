use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

use crate::model::AtUri;

pub const REL_LINK: &str = "https://hopper.at/rel/link";
pub const NS_AUTHORITY: &str = "https://atproto.com/ns/authority";
pub const NS_COLLECTION: &str = "https://atproto.com/ns/collection";
pub const NS_RKEY: &str = "https://atproto.com/ns/rkey";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Link {
    pub(crate) rel: String,
    pub(crate) template: Option<String>,

    #[serde(default)]
    pub(crate) properties: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct WebHostMeta {
    #[serde(default)]
    pub(crate) properties: HashMap<String, String>,

    #[serde(default)]
    pub(crate) links: Vec<Link>,
}

pub(crate) async fn query(http_client: &reqwest::Client, hostname: &str) -> Result<WebHostMeta> {
    let url = format!("https://{}/.well-known/host-meta.json", hostname,);

    http_client
        .get(url)
        .send()
        .await
        .context("web host meta get failed")?
        .json()
        .await
        .context("web host meta parse failed")
}

impl Link {
    pub fn new(template: &str, collection: Option<&str>) -> Self {
        let properties = collection
            .map(|collection| HashMap::from([(NS_COLLECTION.to_string(), collection.to_string())]))
            .unwrap_or_default();
        Self {
            rel: REL_LINK.to_string(),
            template: Some(template.to_string()),
            properties,
        }
    }
}

impl WebHostMeta {
    pub fn new(links: Vec<Link>) -> Self {
        Self {
            properties: Default::default(),
            links,
        }
    }

    pub(crate) fn match_uri(&self, server: &str, aturi: &AtUri) -> Option<String> {
        let prefix = format!("https://{}/", server);
        for link in &self.links {
            if link.rel != REL_LINK {
                continue;
            }

            if link.template.is_none() {
                continue;
            }

            let template = link.template.as_ref().unwrap();

            if !template.starts_with(prefix.as_str()) {
                continue;
            }

            // Property-based matching: if a property is present, the corresponding
            // URI component must match the property value.

            // Check NS_AUTHORITY: if present, authority must match
            if let Some(required_authority) = link.properties.get(NS_AUTHORITY) {
                if &aturi.authority != required_authority {
                    continue;
                }
            }

            // Check NS_COLLECTION: if present, collection must match
            if let Some(required_collection) = link.properties.get(NS_COLLECTION) {
                match &aturi.collection {
                    Some(collection) if collection == required_collection => {
                        // Match, continue checking
                    }
                    _ => continue, // No match, skip this link
                }
            }

            // Check NS_RKEY: if present, rkey must match
            if let Some(required_rkey) = link.properties.get(NS_RKEY) {
                match &aturi.rkey {
                    Some(rkey) if rkey == required_rkey => {
                        // Match, continue checking
                    }
                    _ => continue, // No match, skip this link
                }
            }

            // Template variable substitution:
            // {authority} - The AUTHORITY component from the AT-URI (handle or DID)
            // {collection} - The COLLECTION component (NSID)
            // {rkey} - The RKEY component (record key)
            let mut result = template.replace("{authority}", &aturi.authority);
            if let Some(collection) = &aturi.collection {
                result = result.replace("{collection}", collection);
            }
            if let Some(nsid) = &aturi.rkey {
                result = result.replace("{rkey}", nsid);
            }

            return Some(result);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Link, WebHostMeta};

    #[test]
    fn test_deserialize() {
        let webfinger = serde_json::from_str::<WebHostMeta>(
            r##"{
  "links": [
    {
      "rel": "https://hopper.at/rel/link",
      "template": "https://smokesignal.events/{authority}/{rkey}",
      "properties": {
         "https://atproto.com/ns/collection": "community.lexicon.calendar.event"
      }
    }
  ]
}"##,
        );
        println!("{:?}", webfinger);
        assert!(webfinger.is_ok());

        let webfinger = webfinger.unwrap();
        assert_eq!(webfinger.links.len(), 1);
    }

    #[test]
    fn test_match_uri_no_collection_filter() {
        let hostname = "smokesignal.events".to_string();
        let web_finger = WebHostMeta {
            links: vec![Link {
                rel: "https://hopper.at/rel/link".to_string(),
                template: Some("https://smokesignal.events/profile/{authority}".to_string()),
                properties: HashMap::new(), // No collection filter
            }],
            properties: Default::default(),
        };

        // Should match: no collection filter, URI has no collection
        assert_eq!(
            web_finger.match_uri(
                &hostname,
                &crate::model::AtUri {
                    authority: "ngerakines.me".to_string(),
                    collection: None,
                    rkey: None,
                }
            ),
            Some("https://smokesignal.events/profile/ngerakines.me".into())
        );

        // Should ALSO match: no collection filter means accept any collection
        assert_eq!(
            web_finger.match_uri(
                &hostname,
                &crate::model::AtUri {
                    authority: "smokesignal.events".to_string(),
                    collection: Some("app.bsky.feed.post".into()),
                    rkey: Some("s0xnr5kqnp".into()),
                }
            ),
            Some("https://smokesignal.events/profile/smokesignal.events".into())
        );
    }

    #[test]
    fn test_match_uri_specific_collection() {
        let hostname = "example.com".to_string();
        let web_finger = WebHostMeta {
            links: vec![Link {
                rel: "https://hopper.at/rel/link".to_string(),
                template: Some("https://example.com/{authority}/posts/{rkey}".to_string()),
                properties: HashMap::from([(
                    super::NS_COLLECTION.into(),
                    "app.bsky.feed.post".into(),
                )]),
            }],
            properties: Default::default(),
        };

        // Should match: collection matches
        assert_eq!(
            web_finger.match_uri(
                &hostname,
                &crate::model::AtUri {
                    authority: "alice.example.com".to_string(),
                    collection: Some("app.bsky.feed.post".into()),
                    rkey: Some("abc123".into()),
                }
            ),
            Some("https://example.com/alice.example.com/posts/abc123".into())
        );

        // Should NOT match: different collection
        assert_eq!(
            web_finger.match_uri(
                &hostname,
                &crate::model::AtUri {
                    authority: "alice.example.com".to_string(),
                    collection: Some("app.bsky.feed.like".into()),
                    rkey: Some("abc123".into()),
                }
            ),
            None,
        );

        // Should NOT match: no collection
        assert_eq!(
            web_finger.match_uri(
                &hostname,
                &crate::model::AtUri {
                    authority: "alice.example.com".to_string(),
                    collection: None,
                    rkey: None,
                }
            ),
            None,
        );
    }

    #[test]
    fn test_match_uri_authority_filter() {
        let hostname = "example.com".to_string();
        let web_finger = WebHostMeta {
            links: vec![Link {
                rel: "https://hopper.at/rel/link".to_string(),
                template: Some("https://example.com/special/{authority}".to_string()),
                properties: HashMap::from([(
                    super::NS_AUTHORITY.into(),
                    "alice.example.com".into(),
                )]),
            }],
            properties: Default::default(),
        };

        // Should match: authority matches
        assert_eq!(
            web_finger.match_uri(
                &hostname,
                &crate::model::AtUri {
                    authority: "alice.example.com".to_string(),
                    collection: None,
                    rkey: None,
                }
            ),
            Some("https://example.com/special/alice.example.com".into())
        );

        // Should NOT match: different authority
        assert_eq!(
            web_finger.match_uri(
                &hostname,
                &crate::model::AtUri {
                    authority: "bob.example.com".to_string(),
                    collection: None,
                    rkey: None,
                }
            ),
            None,
        );
    }

    #[test]
    fn test_match_uri_rkey_filter() {
        let hostname = "example.com".to_string();
        let web_finger = WebHostMeta {
            links: vec![Link {
                rel: "https://hopper.at/rel/link".to_string(),
                template: Some("https://example.com/pinned".to_string()),
                properties: HashMap::from([(super::NS_RKEY.into(), "pinned".into())]),
            }],
            properties: Default::default(),
        };

        // Should match: rkey matches
        assert_eq!(
            web_finger.match_uri(
                &hostname,
                &crate::model::AtUri {
                    authority: "alice.example.com".to_string(),
                    collection: Some("app.bsky.feed.post".into()),
                    rkey: Some("pinned".into()),
                }
            ),
            Some("https://example.com/pinned".into())
        );

        // Should NOT match: different rkey
        assert_eq!(
            web_finger.match_uri(
                &hostname,
                &crate::model::AtUri {
                    authority: "alice.example.com".to_string(),
                    collection: Some("app.bsky.feed.post".into()),
                    rkey: Some("abc123".into()),
                }
            ),
            None,
        );

        // Should NOT match: no rkey
        assert_eq!(
            web_finger.match_uri(
                &hostname,
                &crate::model::AtUri {
                    authority: "alice.example.com".to_string(),
                    collection: Some("app.bsky.feed.post".into()),
                    rkey: None,
                }
            ),
            None,
        );
    }

    #[test]
    fn test_match_uri_no_filters() {
        let hostname = "example.com".to_string();
        let web_finger = WebHostMeta {
            links: vec![Link {
                rel: "https://hopper.at/rel/link".to_string(),
                template: Some("https://example.com/{authority}/{collection}/{rkey}".to_string()),
                properties: HashMap::new(), // No filters
            }],
            properties: Default::default(),
        };

        // Should match: no filters means accept anything
        assert_eq!(
            web_finger.match_uri(
                &hostname,
                &crate::model::AtUri {
                    authority: "alice.example.com".to_string(),
                    collection: Some("app.bsky.feed.post".into()),
                    rkey: Some("abc123".into()),
                }
            ),
            Some("https://example.com/alice.example.com/app.bsky.feed.post/abc123".into())
        );

        // Should also match with different values
        assert_eq!(
            web_finger.match_uri(
                &hostname,
                &crate::model::AtUri {
                    authority: "bob.example.com".to_string(),
                    collection: Some("app.bsky.feed.like".into()),
                    rkey: Some("xyz789".into()),
                }
            ),
            Some("https://example.com/bob.example.com/app.bsky.feed.like/xyz789".into())
        );
    }
}
