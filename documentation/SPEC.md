# Hopper Specification

Version: 1.0
Last Updated: 2025-11-02

## Overview

Hopper is an AT-URI redirection service that enables customizable routing of AT Protocol URIs to various web services and providers. This specification defines the protocol and integration patterns for services wishing to support Hopper-based AT-URI resolution.

This specification extends the [Web Host Metadata (RFC 6415)](https://datatracker.ietf.org/doc/html/rfc6415) specification with AT Protocol-specific link relations and properties.

## Link Relations and Namespaces

The following link relationships and namespaces are used in the context of the Web Host Metadata Spec:

### `https://hopper.at/rel/link`

**Type:** Link Relation

**Description:** A Hopper link relation type used to identify link structures that contain templates for transforming AT-URI structures into web URLs.

**Usage:** This link relationship URI is used by Hopper to find `link` structures that include a template used to transform AT-URI structures.

**Reference:** [AT-URI Scheme Specification](https://atproto.com/specs/at-uri-scheme)

### `https://atproto.com/ns/authority`

**Type:** Namespaced Property

**Description:** A filtering property that restricts which authority (handle or DID) a link template applies to.

**Usage:** When present in a link's properties, the link will only match AT-URIs with the specified authority value. If omitted, the link matches any authority.

**Example:** `"https://atproto.com/ns/authority": "alice.example.com"` - only matches URIs with authority `alice.example.com`

### `https://atproto.com/ns/collection`

**Type:** Namespaced Property

**Description:** A filtering property that restricts which collection (NSID) a link template applies to.

**Usage:** When present in a link's properties, the link will only match AT-URIs with the specified collection value. If omitted, the link matches URIs regardless of whether they have a collection or not (wildcard behavior).

**Examples:**
- `"https://atproto.com/ns/collection": "app.bsky.feed.post"` - only matches URIs with collection `app.bsky.feed.post`
- Property omitted - matches URIs with any collection or no collection

### `https://atproto.com/ns/rkey`

**Type:** Namespaced Property

**Description:** A filtering property that restricts which record key (rkey) a link template applies to.

**Usage:** When present in a link's properties, the link will only match AT-URIs with the specified rkey value. If omitted, the link matches any rkey.

**Example:** `"https://atproto.com/ns/rkey": "pinned"` - only matches records with rkey `pinned`

## AT-URI Syntax

Hopper implements the **Restricted AT-URI Syntax** as defined by the AT Protocol specification:

```
AT-URI        = "at://" AUTHORITY [ "/" COLLECTION [ "/" RKEY ] ]

AUTHORITY     = HANDLE | DID
COLLECTION    = NSID
RKEY          = RECORD-KEY
```

Where:
- **AUTHORITY**: Either a handle (e.g., `alice.example.com`) or a DID (e.g., `did:plc:abc123`, `did:web:example.com`). This identifies the repository/account.
- **COLLECTION**: A Namespaced Identifier (NSID) following the format `authority.name.recordType`
- **RKEY**: A record key identifying a specific record within the collection

**Note on Authority Parsing**: The authority component cannot be interpreted as a host:port pair due to the use of colon characters (`:`) in DIDs. Best practice is to use DIDs (not handles) when referencing records from other repositories.

### URI Prefix Support

Hopper supports both standard AT-URIs and the `web+at://` URL scheme prefix:

- `at://alice.example.com/app.bsky.feed.post/abc123`
- `web+at://alice.example.com/app.bsky.feed.post/abc123`

## Template Variables

URI templates defined in `.well-known/host-meta.json` can use the following variables:

1. **`{authority}`** - The authority portion of the AT-URI (handle or DID)
2. **`{collection}`** - The collection NSID
3. **`{rkey}`** - The record key

### Example Template

```
https://example.com/{authority}/{rkey}
```

This template would transform:
```
at://alice.example.com/app.bsky.feed.post/abc1234
     └─────┬─────┘   └────────┬────────┘  └──┬──┘
       authority          collection        rkey
```

Into:
```
https://example.com/alice.example.com/abc1234
                    └─────┬─────┘    └──┬───┘
                     {authority}      {rkey}
```

**Note:** This deviates from the standard Web Host Metadata specification, which typically only supports the `{uri}` variable. Hopper's decomposed variables provide more flexibility for service-specific URL structures.

## Integration Requirements

To integrate with Hopper, a service must provide a `.well-known/host-meta.json` file that meets the following requirements:

### Required Structure

```json
{
  "links": [
    {
      "rel": "https://hopper.at/rel/link",
      "template": "https://your-service.example/{authority}/{rkey}",
      "properties": {
        "https://atproto.com/ns/collection": "your.nsid.record.type"
      }
    }
  ]
}
```

### Parsing Rules

When a Web Host Metadata structure is parsed, the following rules are applied:

1. **Link Relation Filter**: Only links with the `rel` value of `https://hopper.at/rel/link` are used
2. **Template Requirement**: Only links with a `template` attribute are processed
3. **Hostname Validation**: The template must use the same hostname as the server providing the host-meta file
4. **Property Filtering**: If namespace properties (`https://atproto.com/ns/authority`, `https://atproto.com/ns/collection`, or `https://atproto.com/ns/rkey`) are present, the AT-URI components must match the specified values

### Optional Recommendations

- Serve the `/.well-known/host-meta.json` file with the `application/jrd+json` content type
- Support CORS to allow browser-based clients to query the endpoint
- Use HTTPS for all template URLs

## Collection Matching

When resolving an AT-URI, Hopper matches the collection in the URI against the collection property in the link definition:

- If the `https://atproto.com/ns/collection` property is present, the AT-URI's collection must exactly match the property value
- If the `https://atproto.com/ns/collection` property is omitted, the link matches URIs regardless of their collection value (wildcard)

### Collection Matching Examples

**Profile/Identity URIs (no collection filter):**
```json
{
  "rel": "https://hopper.at/rel/link",
  "template": "https://example.com/profile/{authority}",
  "properties": {}
}
```
This matches URIs with or without a collection component, making it suitable for profile pages.

**Specific Collection:**
```json
{
  "rel": "https://hopper.at/rel/link",
  "template": "https://example.com/{authority}/posts/{rkey}",
  "properties": {
    "https://atproto.com/ns/collection": "app.bsky.feed.post"
  }
}
```
This only matches URIs with the exact collection `app.bsky.feed.post`.

## Complete Example

### Service: smokesignal.events

**File:** `https://smokesignal.events/.well-known/host-meta.json`

```json
{
  "links": [
    {
      "rel": "https://hopper.at/rel/link",
      "template": "https://smokesignal.events/{authority}/{rkey}",
      "properties": {
        "https://atproto.com/ns/collection": "community.lexicon.calendar.event"
      }
    }
  ]
}
```

### Resolution Example

**Input AT-URI:**
```
at://did:plc:abc123/community.lexicon.calendar.event/xyz789
```

**Hopper Process:**
1. Queries configured servers for `.well-known/host-meta.json`
2. Finds matching link with `community.lexicon.calendar.event` collection
3. Substitutes template variables:
   - `{authority}` → `did:plc:abc123`
   - `{rkey}` → `xyz789`

**Output URL:**
```
https://smokesignal.events/did:plc:abc123/xyz789
```

## Advanced Filtering Examples

### Using Multiple Filters

You can combine multiple namespace properties to create highly specific link matching rules:

```json
{
  "links": [
    {
      "rel": "https://hopper.at/rel/link",
      "template": "https://example.com/alice/pinned",
      "properties": {
        "https://atproto.com/ns/authority": "alice.example.com",
        "https://atproto.com/ns/collection": "app.bsky.feed.post",
        "https://atproto.com/ns/rkey": "pinned"
      }
    }
  ]
}
```

This link will **only** match the AT-URI: `at://alice.example.com/app.bsky.feed.post/pinned`

### Wildcard Behavior

Properties act as filters - omitting a property makes it a wildcard:

```json
{
  "links": [
    {
      "rel": "https://hopper.at/rel/link",
      "template": "https://example.com/{authority}/posts/{rkey}",
      "properties": {
        "https://atproto.com/ns/collection": "app.bsky.feed.post"
      }
    }
  ]
}
```

This matches any post from any authority with any rkey, because only collection is filtered.

## Multiple Collections

A single host-meta file can define multiple link entries for different collections:

```json
{
  "links": [
    {
      "rel": "https://hopper.at/rel/link",
      "template": "https://example.com/profile/{authority}",
      "properties": {}
    },
    {
      "rel": "https://hopper.at/rel/link",
      "template": "https://example.com/{authority}/posts/{rkey}",
      "properties": {
        "https://atproto.com/ns/collection": "app.bsky.feed.post"
      }
    },
    {
      "rel": "https://hopper.at/rel/link",
      "template": "https://example.com/{authority}/events/{rkey}",
      "properties": {
        "https://atproto.com/ns/collection": "community.lexicon.calendar.event"
      }
    }
  ]
}
```

## Error Handling

Hopper may encounter various error conditions during resolution:

- **Invalid AT-URI**: The provided URI does not conform to AT-URI syntax
- **No Matching Server**: No configured server has a host-meta entry for the collection
- **No Template Match**: The host-meta file exists but contains no matching templates
- **HTTP Errors**: The host-meta endpoint is unreachable or returns an error

Services should ensure their `.well-known/host-meta.json` endpoint is highly available to minimize resolution failures.

## Caching Behavior

Hopper implements caching for both host-meta lookups and resolved AT-URIs:

- **Host-Meta Cache**: Successful lookups are cached indefinitely; failed lookups are cached for 10 minutes
- **AT-URI Cache**: Successful resolutions are cached for 30 minutes; failed resolutions are cached for 10 minutes

Services should expect that changes to their host-meta files may take up to 30 minutes to propagate to all Hopper users.

## Security Considerations

### Template URL Security

- Templates must use HTTPS
- Templates must match the hostname serving the host-meta file (prevents open redirects)
- Services should validate that template expansions produce safe URLs

### DID Resolution

When using DIDs as identity values, services should:
- Validate DID format before using in URLs
- Handle DID resolution appropriately for their use case
- Consider rate limiting and caching for DID resolution

### Input Validation

Services receiving traffic from Hopper should validate all path components:
- Authority values may be long (DIDs) or short (handles)
- Collection NSIDs should match expected format
- Record keys should be validated according to AT Protocol specifications

**Important**: The `{authority}` template variable contains the AUTHORITY component from the AT-URI, which may include colon characters (`:`) when DIDs are used. Services must properly handle these characters in URL paths.

## References

- [Web Host Metadata (RFC 6415)](https://datatracker.ietf.org/doc/html/rfc6415)
- [AT Protocol Specification](https://atproto.com/)
- [AT-URI Scheme](https://atproto.com/specs/at-uri-scheme)
- [Hopper Service](https://hopper.at/)

## Live Examples

- [smokesignal.events host-meta](https://smokesignal.events/.well-known/host-meta.json)
