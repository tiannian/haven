# Packet Backend Specification

## Overview

This specification defines the contract for a platform packet backend that exposes an IP layer only interface for receiving and sending packets.

The backend is responsible for moving packets between the operating system and the core network engine without exposing link layer concepts to the public API. The core engine works only with IPv4 and IPv6 packets, while platform specific details such as Ethernet headers, address resolution, and injection mechanics remain internal to the backend.

## Scope

This specification applies to all packet backends used by the project, including but not limited to:

- Linux packet access backends
- Windows packet diversion backends
- macOS and BSD packet access backends

The specification covers:

- packet data model
- receiver and sender behavior
- batch processing semantics
- metadata and hints
- error handling requirements
- backend invariants

The specification does not define:

- routing policy
- connection tracking logic
- network address translation policy
- application protocol handling
- link layer serialization details exposed to the core engine

## Detailed Specifications

### 1. Public packet model

The public packet model must represent an Internet layer packet only.

A packet object must include at least the following fields:

- packet family: IPv4 or IPv6
- source address
- destination address
- transport protocol identifier
- time to live or hop limit
- payload bytes
- optional packet metadata

The public packet model must not expose any of the following fields as part of the core contract:

- source media access control address
- destination media access control address
- Ethernet type
- virtual local area network tag
- address resolution protocol payload
- neighbor discovery protocol payload

If a backend needs link layer data for internal operation, that data must remain private to the backend implementation.

A representative Rust model is shown below for clarity:

```rust
use std::net::IpAddr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpFamily {
    V4,
    V6,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransportProtocol {
    Tcp,
    Udp,
    Icmp,
    Icmpv6,
    Other(u8),
}

#[derive(Clone, Debug)]
pub struct Packet {
    pub family: IpFamily,
    pub src: IpAddr,
    pub dst: IpAddr,
    pub protocol: TransportProtocol,
    pub ttl_or_hop_limit: u8,
    pub payload: Vec<u8>,
    pub metadata: PacketMetadata,
}
```

### 2. Packet metadata

The packet model may carry metadata needed by the core engine or backend.

A representative Rust metadata model is shown below:

```rust
#[derive(Clone, Debug, Default)]
pub struct PacketMetadata {
    pub ingress_interface: Option<u32>,
    pub egress_interface_hint: Option<u32>,
    pub synthetic: bool,
    pub flow_id: Option<u64>,
    pub dscp: Option<u8>,
    pub flow_label: Option<u32>,
    pub path_mtu_hint: Option<usize>,
    pub mark: Option<u32>,
}
```

Required metadata concepts are:

- ingress interface identifier
- egress interface hint
- synthetic packet flag
- flow identifier when available

Optional metadata concepts are:

- differentiated services code point
- IPv6 flow label
- path maximum transmission unit hint
- packet mark used for loop prevention or backend filtering

Metadata must not change the fact that the packet contract remains Internet layer only.

### 3. Receiver contract

A receiver consumes packets from the operating system or an equivalent backend source and produces public packet objects.

The receiver contract must support batch retrieval.

Required receiver behavior:

- return one or more packets per call when available
- preserve packet ordering within the same backend source stream unless the platform makes that impossible
- report end of stream only when the backend is shut down or detached
- never expose link layer frames to the caller

Receiver function signature requirements:

- the function name must clearly indicate packet reception
- the function must be asynchronous
- the function must return a result type with an error model
- the function must accept an output buffer or return a batch directly

The receiver contract must be async-ready and define readiness behavior through its asynchronous API rather than a synchronous blocking call.

If a backend needs link layer data or readiness polling internally, that detail must remain private to the backend implementation.

A representative Rust receiver contract is shown below:

```rust
pub trait L3Receiver {
    type Error;

    async fn recv_batch(&mut self, output: &mut Vec<Packet>) -> Result<usize, Self::Error>;
}
```

### 4. Sender contract

A sender accepts public packet objects and transmits them toward the operating system or network.

The sender contract must support batch submission.

Required sender behavior:

- accept Internet layer packets only
- preserve packet family and transport protocol semantics
- allow the backend to choose the final link layer representation internally
- reject packets that cannot be serialized or transmitted on the selected backend

Sender function signature requirements:

- the function name must clearly indicate packet sending
- the function must be asynchronous
- the function must return a result type with an error model
- the function must accept a batch of packets or a single packet

The sender contract must be async-ready and define completion behavior through its asynchronous API rather than a synchronous blocking call.

The sender may use operating system assistance for address resolution or may perform address resolution internally, but the public contract must not require the caller to supply link layer details.

A representative Rust sender contract is shown below:

```rust
pub trait L3Sender {
    type Error;

    async fn send_batch(&mut self, packets: &[Packet]) -> Result<usize, Self::Error>;
}
```

### 5. Batch processing requirements

Batch processing is a required capability.

The backend must support batch receive and batch send semantics because the project targets high throughput packet handling.

Batch semantics requirements:

- zero returned packets is valid when no packet is ready
- partial batch completion is valid when the backend can only process part of the input or output set
- batch processing must not reorder packets within the same batch unless the backend documents a platform limitation
- batch interfaces must be safe to use repeatedly in a loop

### 6. Backend responsibilities

The backend is responsible for the following internal tasks when required by the platform:

- packet serialization and deserialization
- checksum validation or checksum correction when needed for transmission
- local neighbor resolution for IPv4 and IPv6 when the operating system does not provide it through the chosen send path
- loop prevention for packets generated by the same backend
- interface and path selection hints
- platform specific packet injection and capture mechanics

These responsibilities are internal implementation details and must not leak into the public packet interface.

### 7. Receiver and sender lifecycle

A packet backend must have a clear lifecycle.

Required lifecycle behavior:

- the backend can be created with explicit platform configuration
- the backend can be opened and closed deterministically
- once closed, the backend must stop receiving and sending packets
- resources associated with the backend must be released on shutdown

If the backend supports asynchronous operation, the lifecycle contract must still define clear open, close, and shutdown semantics.

### 8. Error handling

All receiver and sender operations must return structured errors.

A representative Rust error model is shown below:

```rust
#[derive(thiserror::Error, Debug)]
pub enum BackendError {
    #[error("backend not initialized")]
    NotInitialized,
    #[error("backend closed")]
    Closed,
    #[error("packet decode failure")]
    PacketDecodeFailure,
    #[error("packet encode failure")]
    PacketEncodeFailure,
    #[error("packet rejected by policy")]
    RejectedByPolicy,
    #[error("transmit failure")]
    TransmitFailure,
    #[error("receive failure")]
    ReceiveFailure,
    #[error("platform specific configuration failure")]
    PlatformConfigurationFailure,
}
```

Error types should distinguish at least the following cases:

- backend not initialized
- backend closed
- packet decode failure
- packet encode failure
- packet rejected by policy
- transmit failure
- receive failure
- platform specific configuration failure

Errors must be precise enough for the core engine to decide whether to drop a packet, retry the operation, or disable the backend.

### 9. Platform specific requirements

#### Linux

The Linux backend may use packet socket mechanisms for packet capture and transmission.

If the backend uses a link layer oriented mechanism internally, it must still present Internet layer packets to the public API.

#### Windows

The Windows backend may use packet diversion mechanisms for packet capture and reinjection.

The public API must still present Internet layer packets only.

#### macOS and BSD

The macOS and BSD backend may use Berkeley Packet Filter mechanisms or equivalent platform facilities.

The public API must still present Internet layer packets only.

### 10. Interoperability requirements

The packet backend must be usable by a core engine that performs the following operations independently of platform:

- packet parsing
- flow classification
- connection tracking
- policy evaluation
- packet rewriting
- packet generation
- packet forwarding

The backend must not require the core engine to know whether the packet was captured from the network or generated synthetically.

A representative Rust backend trait is shown below:

```rust
pub trait PacketBackend: L3Receiver + L3Sender {
    fn open(&mut self) -> Result<(), BackendError>;
    fn close(&mut self) -> Result<(), BackendError>;
}
```

### 11. Non requirements

The packet backend specification does not require:

- a virtual network interface
- modification of the system routing table
- modification of the system firewall rules
- exposure of link layer headers to the application layer
- a single implementation strategy across all operating systems

## References

- `specs/0001-specs.md` for the specification file naming and content rules
- Linux packet socket documentation
- Windows WinDivert documentation
- Berkeley Packet Filter documentation for macOS and BSD
- project level architecture and packet engine specifications when added later
