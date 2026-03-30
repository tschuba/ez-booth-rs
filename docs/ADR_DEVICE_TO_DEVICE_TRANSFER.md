# ADR: Device-to-Device Booth Data Transfer

Date: 2026-03-30
Status: Pending
Decision Maker: TBD

## Context

The ez-booth application requires the ability to transfer booth data from one device to another without requiring:

- internet connectivity
- WiFi router or access point infrastructure
- cloud services or external servers

The transfer scope for the first implementation is a single booth including booth configuration, vendors, and purchases.

### Background

QR code export and import functionality already exists, but it is limited by data density. In practice, QR transfer is not effective for booth data once the payload grows beyond very small datasets.

Real-world booth data can range from tens of kilobytes to multiple megabytes depending on the number of vendors, purchases, and retained history. This makes QR codes insufficient for the intended device-to-device workflow.

### Platform Priority

1. Windows
2. macOS
3. iOS
4. Android

### Technical Constraints

- the application runs in web browsers as a WASM app
- transfer must work without internet and without WiFi router infrastructure
- the solution should favor mature, reliable platform capabilities over experimental browser APIs
- the existing export/import infrastructure should be reused where possible

### Existing Capabilities

- export and import services already exist in `crates/storage/src/export/`
- MessagePack serialization and gzip compression are already used in the codebase
- backup validation and import conflict handling already exist
- QR transfer exists but is not sufficient for the main use case here

## Decision Drivers

- true device-to-device operation without internet or WiFi router
- strong support for Windows and macOS first
- workable support for iOS and Android
- higher data density than QR transfer
- low operational risk and predictable user experience
- reasonable implementation complexity using existing services

## Options Considered

### Option 1: Web Bluetooth Transfer

#### Concept

Use the Web Bluetooth API for direct BLE transfer between devices.

The ideal model is:

- one device advertises as a BLE peripheral
- the other device scans and connects as a BLE central
- data transfers through custom GATT characteristics in chunks

#### Pros

- true device-to-device radio communication
- no router or internet required
- suitable data density for booth transfer
- works well in principle for nearby devices

#### Cons

- critical browser limitation: Web Bluetooth in browsers does not support peripheral advertising for this use case
- browsers can scan and connect, but cannot reliably expose the app as a discoverable BLE peripheral
- iOS Safari does not provide usable Web Bluetooth support
- Firefox support is not practical
- high implementation and support complexity

#### Platform Fit

- Windows: partial browser support, but blocked by advertising limitation
- macOS: partial browser support, but blocked by advertising limitation
- iOS: not viable
- Android: partial browser support, but still limited by browser role restrictions

#### Outcome

Rejected. BLE is attractive conceptually, but browser limitations make it unsuitable for a web-based product that must work reliably across the target platforms.

### Option 2: WebRTC Data Channels With Manual Signaling

#### Concept

Use WebRTC data channels for high-density transfer and exchange signaling information manually, for example with small QR payloads or short codes.

The intended flow is:

- device A generates a WebRTC offer
- devices exchange offer and answer out of band
- WebRTC establishes a peer connection
- booth data transfers over the data channel

#### Pros

- high transfer speed once connected
- broad browser support for WebRTC itself
- good data density and reliability after connection establishment

#### Cons

- WebRTC still needs a workable network path between devices
- in practice this usually means same-network connectivity, Bluetooth PAN, or other infrastructure-like support
- it does not reliably satisfy the requirement of isolated device-to-device transfer with no WiFi router and no internet across the full platform set
- multi-step signaling adds user friction and failure cases

#### Platform Fit

- Windows: technically possible in some environments, but dependent on network path
- macOS: technically possible in some environments, but dependent on network path
- iOS: WebRTC support exists, but transport assumptions remain
- Android: workable in some scenarios, but not a universal infrastructure-free solution

#### Outcome

Rejected. WebRTC is strong for connected peers, but it does not cleanly meet the requirement of direct device-to-device transfer without relying on supporting network conditions.

### Option 3: Web Share API Plus Native OS Transfer Mechanisms

#### Concept

Export booth data as a file and let the operating system perform the actual device-to-device transfer using native share capabilities.

The flow is:

- export a single booth into a transfer file
- invoke native share where supported
- the OS handles peer discovery and transfer
- the receiving device imports the file into ez-booth

Examples include:

- AirDrop on Apple platforms
- Nearby Share or equivalent native sharing on supported platforms
- manual file transfer fallback where native peer sharing is not available in the browser context

#### Pros

- leverages mature OS-level transfer capabilities instead of fragile browser-only peer protocols
- fits iOS much better than Web Bluetooth
- reuses the existing export and import architecture
- supports large payloads far beyond QR density limits
- keeps the product aligned with a web-first architecture
- gives a practical desktop fallback through ordinary file export and import

#### Cons

- transfer is not fully contained inside a single in-app peer discovery flow
- desktop browser support for invoking native share is less uniform than on mobile
- import UX on the receiving side still needs design work

#### Platform Fit

- Windows: viable with file export/import fallback and native share where available
- macOS: viable with file export/import fallback and strong OS sharing options
- iOS: strong fit through native sharing
- Android: viable through native sharing support

#### Outcome

Recommended. This is the most practical and robust option for the current product constraints.

### Option 4: Local WiFi HTTP Server

#### Concept

Make one device act as a local server and let the other device connect through a local network.

#### Pros

- straightforward transport model if a network is available
- high data density and good throughput

#### Cons

- requires network infrastructure or a comparable setup that violates the stated constraint
- browser runtime limitations make local server behavior awkward and inconsistent
- not appropriate for the direct device-to-device requirement

#### Outcome

Rejected. This option does not satisfy the infrastructure constraint.

### Option 5: Native Application Wrapper

#### Concept

Wrap the app in a native shell such as Tauri or Electron to gain access to native platform APIs for peer-to-peer transfer, including stronger Bluetooth options.

#### Pros

- unlocks native APIs unavailable to browsers
- could enable a more seamless in-app direct transfer experience on desktop

#### Cons

- changes the deployment and support model significantly
- adds native packaging, distribution, and maintenance overhead
- does not help enough for the current web-first scope

#### Outcome

Deferred. This is a future architectural direction, not the right next step for the current product.

## Decision

Select Option 3: Web Share API plus native OS transfer mechanisms, with standard file export and import as the desktop fallback.

## Rationale

This option best matches the actual constraints and platform priorities.

- It avoids browser API gaps that make Web Bluetooth unreliable or impossible for the required role.
- It avoids the hidden network dependency in WebRTC-based approaches.
- It uses the strongest parts of the current product: export, import, validation, and structured backup data.
- It supports a practical user story on every target platform, even when the exact transfer UX differs.

For mobile devices, native OS sharing is the most realistic path to true infrastructure-free transfer.

For desktop platforms, file export and import is an acceptable fallback because Windows and macOS are the highest-priority platforms and users on those systems can reliably complete transfer through standard file-based workflows even when direct browser-triggered native sharing is inconsistent.

## Decisions Made

### Transfer Format

Use MessagePack as the preferred transfer format.

Rationale:

- better density than JSON
- already aligned with existing codebase usage
- suitable for compressed single-booth transfer files

### Initial Scope

Focus on single-booth transfer.

Rationale:

- matches the stated use case
- keeps UX and validation simpler
- full-database export already exists as a separate backup concern

### Desktop Fallback Strategy

Use standard file export and file import as the desktop fallback.

Rationale:

- works reliably on Windows and macOS
- avoids dependence on uneven browser support for native sharing
- keeps the concept understandable and supportable

## Consequences

### Positive

- practical path forward without changing the app into a native product
- strong reuse of current export and import services
- significantly better payload density than QR transfer
- clear fallback story for desktop platforms
- lower implementation risk than browser-only peer protocols

### Negative

- receiving-side import flow still needs UX design
- transfer UX will vary somewhat by platform
- the selected approach is less magical than a fully in-app peer discovery flow

### Neutral

- further documentation will be needed for operator-facing instructions
- the exact transfer file extension and file association details can be decided during implementation planning

## Open Question

The remaining open product question is the receiving-side import UX.

This ADR leaves that design open for a follow-up decision, including whether the first version should support:

- basic file picker import only
- drag and drop import
- registered file handling where supported
- a combination of the above

## Next Steps

1. decide the receiving-side import UX
2. create a focused implementation plan for single-booth transfer
3. reuse the existing export/import services with MessagePack-based transfer files
4. document platform-specific transfer flows for operators

## Related Documents

- `docs/DATA_BACKUP_IMPLEMENTATION_PLAN.md`
- `docs/VALIDATION_WORKFLOW.md`
- `docs/BRANCH_STRATEGY.md`
- `README.md`
