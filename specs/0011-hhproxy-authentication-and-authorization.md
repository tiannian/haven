# HHProxy Authentication and Authorization Specification

## Overview

This specification defines the control-plane and data-plane contract for the HHProxy system.

The system is divided into two components:

- `hhproxy-server`: the control plane responsible for authentication, authorization, token issuance, refresh token handling, and public key distribution
- `hhproxyd`: the data plane responsible for accepting proxy connections, validating access tokens, and applying the upstream configuration embedded in the token

The system uses JSON Web Tokens (JWT) as access tokens. The access token contains the upstream configuration directly, so the data plane does not query the server for runtime routing data.

The system also uses refresh tokens for renewing access tokens. Refresh tokens are handled only by `hhproxy-server`.

## Scope

This specification applies to the following behavior:

- access token issuance
- refresh token issuance and renewal
- JWT signature verification
- public key retrieval by `hhproxyd`
- authorization decisions based on JWT claims
- one-connection-to-one-flow mapping
- embedded upstream configuration in access tokens
- TCP proxying behavior

This specification does not define:

- UDP support
- packet multiplexing
- connection framing protocols
- dynamic configuration updates for already issued access tokens
- routing-table manipulation
- firewall rule manipulation
- implementation details of the JWT library
- storage backend details for refresh tokens

## Non-goals

The following are explicitly out of scope for this version:

- UDP transport support
- token revocation lists beyond normal expiration handling
- upstream configuration refresh for already issued access tokens
- streaming multiple logical flows over a single proxy connection
- compatibility with legacy headers such as `X-UPSTREAM-CONFIG`

## Detailed Specifications

### 1. System roles

#### 1.1 `hhproxy-server`

`hhproxy-server` is the control-plane service.

It is responsible for:

- authenticating the user or client
- issuing refresh tokens
- issuing access tokens as JWTs
- signing JWTs with the server private key
- publishing the corresponding JWT public key for `hhproxyd`
- validating refresh tokens before issuing a new access token

`hhproxy-server` is the source of truth for token issuance.

#### 1.2 `hhproxyd`

`hhproxyd` is the data-plane proxy service.

It is responsible for:

- accepting inbound proxy connections
- validating access token signatures
- validating access token claims
- deriving upstream connection parameters from the JWT payload
- establishing a single upstream connection for each accepted client connection
- forwarding traffic between the client and the selected upstream

`hhproxyd` must not depend on runtime configuration headers such as `X-UPSTREAM-CONFIG`.

### 2. Connection model

Each client connection maps to exactly one proxy flow.

Required behavior:

- one accepted connection must correspond to one upstream stream
- the proxy must not multiplex multiple logical flows into one connection
- the proxy must not require a custom framing protocol for stream demultiplexing
- the proxy must not reconstruct multiple streams from a single client connection

This model applies to TCP proxying.

UDP is not part of this specification.

### 3. Token model

#### 3.1 Access token

An access token must be a JWT signed by `hhproxy-server`.

The access token must contain all information required by `hhproxyd` to decide whether the connection is permitted and where it should be forwarded.

The access token must include at least the following logical claims:

- subject: the identity of the authenticated client
- expiration time
- issued-at time
- unique token identifier
- authorization scope or permission set
- upstream configuration

The upstream configuration embedded in the JWT must include at least:

- upstream host name or address
- upstream port
- upstream protocol or scheme
- any permission-relevant routing attributes required by the proxy decision

The exact claim encoding may use standard JWT fields plus custom claims.

A representative claim shape is shown below for clarity:

```json
{
  "iss": "hhproxy-server",
  "sub": "client-123",
  "aud": "hhproxyd",
  "exp": 1735689600,
  "iat": 1735686000,
  "jti": "8d6f5e4f-9d1f-4c65-8d5f-2b0d4c1f0f67",
  "scope": ["proxy:connect"],
  "upstream": {
    "host": "example.internal",
    "port": 443,
    "protocol": "tcp"
  }
}
```

The access token may be treated as authoritative for upstream selection until it expires.

#### 3.2 Refresh token

A refresh token must be a credential that is accepted only by `hhproxy-server`.

Required behavior:

- refresh tokens must not be accepted by `hhproxyd`
- refresh tokens must not be used as proxy authorization tokens
- refresh tokens must be validated before a new access token is issued
- refresh tokens must allow renewal without exposing the server private key to clients

A refresh token may be opaque or structured, but its format must not require `hhproxyd` to interpret it.

### 4. Authorization rules

Authorization must be decided from the JWT presented to `hhproxyd`.

A connection is authorized only if all of the following are true:

- the JWT signature is valid
- the JWT has not expired
- the JWT audience, issuer, and subject are acceptable for the current deployment
- the requested connection is permitted by the token scope
- the embedded upstream configuration is syntactically valid
- the upstream target is allowed by policy encoded in the token

If any required condition fails, `hhproxyd` must reject the connection.

Authorization must not depend on `X-UPSTREAM-CONFIG` or other untrusted request headers.

### 5. Public key distribution

`hhproxyd` must be configured with the `hhproxy-server` address.

At startup, `hhproxyd` must retrieve the JWT public key from `hhproxy-server`.

Required behavior:

- `hhproxyd` must cache the retrieved public key locally
- `hhproxyd` must use the public key to verify JWT signatures
- if key rotation is supported, the JWT header should include a key identifier (`kid`)
- if a token references an unknown key identifier, `hhproxyd` may retry key retrieval from `hhproxy-server`

The specification requires public key retrieval, but it does not require `hhproxyd` to fetch other runtime configuration from the server.

### 6. Token issuance and refresh flow

The control-plane flow must be:

1. The client authenticates with `hhproxy-server`.
2. `hhproxy-server` validates the client credentials.
3. `hhproxy-server` issues:
   - one access token as a JWT
   - one refresh token
4. The client uses the access token to connect to `hhproxyd`.
5. When the access token expires, the client presents the refresh token to `hhproxy-server`.
6. `hhproxy-server` validates the refresh token and issues a new access token.

Required constraints:

- refresh token renewal must happen only through `hhproxy-server`
- `hhproxyd` must not mint tokens
- `hhproxyd` must not exchange refresh tokens for access tokens
- the access token lifetime should be short enough to limit exposure if it is leaked

### 7. Connection handling requirements

For each accepted connection, `hhproxyd` must:

- read and validate the presented access token
- derive the upstream target from the token
- establish exactly one upstream connection
- forward data bidirectionally until either side closes
- terminate the flow when the token is invalid or expired before connection establishment

If the access token expires after the connection is established, the proxy may continue the active stream until normal session termination unless a stricter deployment policy is defined elsewhere.

### 8. Error handling

The proxy and server must distinguish token and authorization failures from transport failures.

Recommended error categories include:

- invalid JWT signature
- expired access token
- unknown key identifier
- unauthorized scope
- invalid upstream configuration in token
- refresh token invalid
- refresh token expired
- server unavailable during public key retrieval
- upstream connection failure

Errors returned by `hhproxyd` should be precise enough to support logging, metrics, and client diagnostics.

### 9. Security requirements

The following requirements are mandatory:

- the JWT private key must remain on `hhproxy-server`
- `hhproxyd` must verify tokens using only the public key
- refresh tokens must never be accepted as proxy credentials
- access tokens must include expiration
- claims that influence authorization must be signed
- untrusted headers must not override JWT claims

Implementations should avoid storing long-lived secrets in `hhproxyd` beyond the server public key.

## References

- `specs/0001-specs.md` — repository specs directory guide
- `specs/0010-packet-backend.md` — packet backend contract used by the data plane
