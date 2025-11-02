# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Hopper is an AT Protocol URI resolver service written in Rust. It translates AT-URIs (`at://identity/collection/rkey`) into web URLs by querying `.well-known/host-meta.json` files from various servers. The service uses HTMX for partial page rendering.

## Build and Development Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Build for release (with LTO and stripping)
cargo build --release

# Check version
cargo run -- --version
```

## Required Environment Variables

- `EXTERNAL_BASE`: The external base URL for the service (e.g., `https://hopper.at`)

## Optional Environment Variables

- `HTTP_PORT`: Port to listen on (default: 4060)
- `CERTIFICATE_BUNDLES`: Semicolon-separated list of CA certificate paths
- `USER_AGENT`: Custom user agent string (default: `hopper (<version>; +https://hopper.at/)`)
- `RUST_LOG`: Logging configuration (default: `hopper=debug,info`)

## Build Features

Templates are embedded into the binary using `minijinja-embed` for production builds.

## Architecture

### Core Components

**AT-URI Resolution Flow** (`src/model.rs`, `src/webhostmeta.rs`, `src/cache.rs`):
1. User submits an AT-URI via the web interface
2. URI is validated and parsed into `AtUri { authority, collection, rkey }`
3. Service queries multiple servers' `.well-known/host-meta.json` endpoints
4. Matches the AT-URI against template patterns in the host-meta response
5. Redirects to the resolved web URL

**Caching Strategy** (`src/cache.rs`):
- `resolve_webfinger_cache`: Caches host-meta.json results. Found entries cached indefinitely, NotFound for 10 minutes.
- `resolve_aturi_cache`: Caches resolved AT-URIs. Found entries cached for 30 minutes, NotFound for 10 minutes.
- Uses Moka with custom expiry policies

**Hard-coded Server Mappings** (`src/bin/hopper.rs:53-91`):
The application pre-populates the cache with mappings for known services (bsky.app, frontpage.fyi, whtwnd.com). When adding support for new services, add entries here following the existing pattern.

### HTTP Layer

**Server Setup** (`src/http/server.rs`):
- Axum-based web server with CORS, timeout, and tracing middleware
- Three main routes: `/` (index), `/spec`, `/policy`
- Static file serving

**Request Context** (`src/http/context.rs`):
- `WebContext`: Shared application state containing template engine, HTTP client, and caches

**Template Naming Convention** (`templates/`):
Format: `{key}.{modifier}.html`
- Key: `index`, `spec`, `alert`, `policy`
- Modifier: empty (full page), `partial` (HTMX partial), `bare` (HTMX boosted)

Example: `index.partial.html` for HTMX partial rendering of the home page

### Web Host Meta Protocol

The service implements a custom protocol using `.well-known/host-meta.json`:

```json
{
  "links": [{
    "rel": "https://hopper.at/rel/link",
    "template": "https://example.com/{authority}/{rkey}",
    "properties": {
      "https://atproto.com/ns/collection": "app.example.post"
    }
  }]
}
```

Template variables: `{authority}`, `{collection}`, `{rkey}`

The matching logic (`src/webhostmeta.rs:64-110`) finds the first link where:
- `rel` matches `REL_LINK` (`https://hopper.at/rel/link`)
- Template starts with server's HTTPS URL
- If namespace properties are present, they must match:
  - `https://atproto.com/ns/authority` - filters by authority (handle or DID)
  - `https://atproto.com/ns/collection` - filters by collection NSID
  - `https://atproto.com/ns/rkey` - filters by record key
- Properties act as filters: omitting a property makes it a wildcard (matches any value)

## Documentation

**Formal Specification**: See `documentation/SPEC.md` for the complete Hopper protocol specification, including:
- Link relations and namespaces
- AT-URI syntax and template variables
- Integration requirements and parsing rules
- Collection matching behavior
- Complete examples and error handling

## Testing

Run tests with `cargo test`. Key test files:
- `src/webhostmeta.rs`: Tests for host-meta parsing and URI matching
- `src/model.rs`: Validation tests for AT-URIs, NSIDs, hostnames, and identities (handles, DIDs)
