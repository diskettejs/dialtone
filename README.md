# `@diskette/dialtone`

Node.js native bindings for [Zenoh](https://zenoh.io) — a pub/sub/query protocol — built with [NAPI-RS](https://napi.rs).

The bindings mirror **Zenoh 1.9**'s surface as faithfully as the JS boundary allows: the operations, options, and lifecycle semantics are Zenoh's. Advanced pub/sub (from `zenoh-ext`) is folded into the regular `Publisher`/`Subscriber` surface — every declared publisher and subscriber is an advanced one. This package is **Node.js only**; there are no WASM/WASI builds for the browser.

## Install

```bash
pnpm add @diskette/dialtone
```

```ts
import { Session } from '@diskette/dialtone'
```

## Requirements

- **Node.js ≥ 20.4** — the disposal helpers (`using` / `await using`) rely on `Symbol.dispose` / `Symbol.asyncDispose`, available from Node 20.4. The runtime API itself works on older Node, but explicit resource management does not.
- **TypeScript ≥ 5.2** (or an equivalent transpiler/bundler) if you use the `using` / `await using` syntax shown throughout these examples.
- Prebuilt native binaries ship for x86_64 Windows, x86_64 / arm64 macOS, and x86_64 Linux (glibc).

## API Reference

- [Conventions](#conventions)
- [`Session`](#session) · [`SessionInfo`](#sessioninfo)
- [Publishing — `Publisher`](#publishing)
- [Subscribing — `Subscriber`, `Sample`](#subscribing)
- [Querying — `Querier`, `Query`, `Queryable`, `Reply`, `ReplyError`](#querying)
- [Liveliness — `Liveliness`, `LivelinessToken`, `LivelinessSubscriber`](#liveliness)
- [Matching — `MatchingListener`, `MatchingStatus`](#matching)
- [Sample-miss detection — `SampleMissListener`, `Miss`](#sample-miss-detection)
- [Scouting — `Scout`, `Hello`, `WhatAmIMatcher`](#scouting)
- [Connectivity — `Transport`, `Link`, event listeners](#connectivity)
- [Channel handlers — the receive surface](#channel-handlers)
- [Key expressions & selectors — `KeyExpr`, `Selector`, `Parameters`](#key-expressions--selectors)
- [Payloads & encoding — `Bytes`, `Encoding`](#payloads--encoding)
- [Serialization — `Serializer`, `Deserializer`](#serialization)
- [Endpoints & locators — `EndPoint`, `Locator`, `Metadata`](#endpoints--locators)
- [Identifiers, time & cancellation](#identifiers-time--cancellation)
- [Configuration — `Config`](#configuration)
- [Enumerations](#enumerations)

A pub/sub round-trip, for orientation:

```ts
import { Session, Config } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
await using sub = await session.declareSubscriber('demo/example/**')
await using pub = await session.declarePublisher('demo/example/greeting')

await pub.put('hello')

const sample = await sub.handler.recvAsync()
console.log(sample.keyExpr.asStr, '→', sample.payload.toString())
```

### Conventions

A handful of patterns recur across the whole surface.

**Entry point.** Everything starts from a [`Session`](#session), opened with `Session.open(config)`. The session declares publishers, subscribers, queryables, queriers, and liveliness tokens, and exposes one-shot `put` / `delete` / `get`.

**Key expressions and selectors.** Wherever a key expression is accepted as input, you may pass a `string` or a [`KeyExpr`](#keyexpr) — this is the `KeyExprArg` alias. Wherever a *selector* is accepted (a key expression plus optional `?`-parameters), you may pass a `string`, a `KeyExpr`, or a [`Selector`](#selector) — the `SelectorArg` alias.

```ts
export type KeyExprArg = string | KeyExpr
export type SelectorArg = string | KeyExpr | Selector
```

**Channels and handlers.** Every entity that *receives* values (subscribers, queryables, the reply stream of a `get`, scouts, listeners) exposes a **handler** — the receive end of a channel. Two channel kinds exist:

- **`Fifo`** (the default) — a bounded FIFO that back-pressures the network when full and drops nothing. Its handler has the full receive + introspection + `stream()` surface.
- **`Ring`** — a bounded ring that keeps the most recent `capacity` values, dropping the oldest. Its handler exposes only the receive variants.

Select the channel with the `handler` option, e.g. `{ handler: { kind: 'Ring', capacity: 32 } }`. `capacity` defaults to **256** when omitted. See [Channel handlers](#channel-handlers) for the full method set; the choice narrows the handler type at compile time.

```ts
export interface ChannelConfig {
  kind: ChannelKind   // 'Fifo' | 'Ring'
  capacity?: number   // default: 256
}
```

**Resource management.** Declared entities are released by calling `undeclare()` (or `close()` on the session, `stop()` on a scout). Most also implement the disposal protocol, so `using` / `await using` releases them at scope exit:

| Entity | Release | Disposal |
|---|---|---|
| `Session` | `close()` | `await using` |
| `Publisher`, `Subscriber`, `Queryable`, `Querier` | `undeclare()` | `await using` |
| `MatchingListener`, `SampleMissListener` | `undeclare()` | `await using` |
| `LivelinessToken`, `LivelinessSubscriber` | `undeclare()` | `await using` |
| `Scout` | `stop()` | `using` (synchronous — no network round-trip) |
| `TransportEventsListener`, `LinkEventsListener` | `undeclare()` | — (not disposable) |

Releasing is idempotent: a second `undeclare()` / `close()` / `stop()` is a no-op. The two `SessionInfo` listeners are the lone exception to disposal — they are released only via `undeclare()`, not `using` / `await using`.

---

### Session

The entry point. Open one with `Session.open(config)`; everything else is declared on it.

**Static methods**

- `Session.open(config: Config): Promise<Session>` — Opens a session with the given configuration.

**Properties**

- `zid: string` — This session's Zenoh id, as a hex string.
- `id: EntityGlobalId` — The global id of this session entity.
- `isClosed: boolean` — Whether the session has been closed.

**Lifecycle & sub-APIs**

- `close(): Promise<void>` — Closes the session, undeclaring everything declared on it.
- `liveliness(): Liveliness` — The [liveliness](#liveliness) sub-API (tokens, subscribers, `get`).
- `info(): SessionInfo` — The [connectivity](#sessioninfo) sub-API (transports, links, ids, lifecycle listeners).
- `newTimestamp(): Timestamp` — Mints a fresh [`Timestamp`](#timestamp) from the session's clock, stamped with this session's Zenoh id.

**One-shot operations**

- `put(keyExpr: KeyExprArg, payload: string | Uint8Array, options?: PutOptions): Promise<void>` — Publishes `payload` on `keyExpr`.
- `delete(keyExpr: KeyExprArg, options?: DeleteOptions): Promise<void>` — Deletes the data matching `keyExpr` (publishes a `Delete` sample).
- `get(selector: SelectorArg, options?: GetOptions): Promise<ReplyHandler>` — Sends a one-shot query and returns the [reply handler](#channel-handlers). The handler completes (disconnects) once the query is resolved.

**Declarations**

- `declareKeyexpr(keyExpr: KeyExprArg): Promise<KeyExpr>` — Declares `keyExpr` on the session, returning an optimized handle. Zenoh assigns it a numeric id, cutting wire overhead when the same key expression is reused.
- `declarePublisher(keyExpr: KeyExprArg, options?: PublisherOptions): Promise<Publisher>` — Declares a [publisher](#publishing), fixing its QoS for every publication.
- `declareSubscriber(keyExpr: KeyExprArg, options?: SubscriberOptions): Promise<Subscriber>` — Declares a [subscription](#subscribing). The handler type narrows by the chosen channel kind.
- `declareQuerier(keyExpr: KeyExprArg, options?: QuerierOptions): Promise<Querier>` — Declares a [querier](#querying), fixing its config for every `get`.
- `declareQueryable(keyExpr: KeyExprArg, options?: QueryableOptions): Promise<Queryable>` — Declares a [queryable](#querying) that answers matching queries.

#### `PutOptions`

| Field | Type | Description |
|---|---|---|
| `encoding` | `string` | Encoding of the payload (e.g. `'text/plain'`). |
| `congestionControl` | `CongestionControl` | Queue behavior when full. |
| `priority` | `Priority` | Routing priority. |
| `express` | `boolean` | Send unbatched when `true`. |
| `reliability` | `Reliability` | Reliability to route with. |
| `allowedDestination` | `Locality` | Restrict which entities receive the data. |
| `timestamp` | `Timestamp` | Explicit timestamp to stamp the sample with. |
| `attachment` | `Uint8Array` | Arbitrary bytes attached to the message. |
| `sourceInfo` | `SourceInfo` | Source id + sequence number to stamp. |

#### `DeleteOptions`

| Field | Type | Description |
|---|---|---|
| `congestionControl` | `CongestionControl` | Queue behavior when full. |
| `priority` | `Priority` | Routing priority. |
| `express` | `boolean` | Send unbatched when `true`. |
| `reliability` | `Reliability` | Reliability to route with. |
| `allowedDestination` | `Locality` | Restrict which entities receive the delete. |
| `timestamp` | `Timestamp` | Explicit timestamp to stamp the sample with. |
| `attachment` | `Uint8Array` | Arbitrary bytes attached to the message. |
| `sourceInfo` | `SourceInfo` | Source id + sequence number to stamp. |

#### `GetOptions`

| Field | Type | Description |
|---|---|---|
| `target` | `QueryTarget` | Which queryables to target. |
| `consolidation` | `ConsolidationMode` | How replies are consolidated before delivery. |
| `congestionControl` | `CongestionControl` | Queue behavior when full. |
| `priority` | `Priority` | Routing priority. |
| `express` | `boolean` | Send unbatched when `true`. |
| `allowedDestination` | `Locality` | Restrict which entities receive the query. |
| `timeout` | `number` | Query timeout in milliseconds. |
| `payload` | `Uint8Array` | Optional query payload. |
| `encoding` | `string` | Encoding of the query payload. |
| `attachment` | `Uint8Array` | Arbitrary bytes attached to the query. |
| `sourceInfo` | `SourceInfo` | Source id + sequence number to stamp. |
| `cancellationToken` | `CancellationToken` | Token that interrupts the in-flight query. |
| `handler` | `ChannelConfig` | Channel for the reply handler (default: FIFO/256). |

---

### SessionInfo

The connectivity sub-API for a session: who it is connected to, its own and its neighbours' Zenoh ids, and lifecycle-event listeners. Reached via `session.info()`.

**Identity & peers** (each resolves asynchronously)

- `zid(): Promise<string>` — This session's Zenoh id, as a hex string.
- `routersZid(): Promise<Array<string>>` — Zenoh ids of the connected routers (or the current router, if running inside one).
- `peersZid(): Promise<Array<string>>` — Zenoh ids of the connected peers.
- `locators(): Promise<Array<Locator>>` — The locators this session is listening on.

**Transports & links**

- `transports(): Promise<Array<Transport>>` — The currently-open [transports](#transport).
- `links(): Promise<Array<Link>>` — The currently-established [links](#link) across all transports.

**Lifecycle listeners**

- `transportEventsListener(options?: TransportEventsListenerOptions): Promise<TransportEventsListener>` — A listener for transports opening/closing.
- `linkEventsListener(options?: LinkEventsListenerOptions): Promise<LinkEventsListener>` — A listener for links being added/removed.

#### `TransportEventsListenerOptions` / `LinkEventsListenerOptions`

| Field | Type | Description |
|---|---|---|
| `history` | `boolean` | Replay the currently-open transports / established links before live events. |
| `handler` | `ChannelConfig` | Channel for the listener's handler (default: FIFO/256). |

---

### Publishing

A `Publisher` declared on a key expression with fixed QoS. Obtain one via `session.declarePublisher(keyExpr, options?)`. Every declared publisher is an *advanced* publisher (caching, sample-miss detection, publisher detection).

#### `Publisher`

**Properties**

- `keyExpr: KeyExpr` — The key expression this publisher publishes on.
- `id: EntityGlobalId` — The global id of this publisher entity.
- `encoding: Encoding` — The encoding applied to published data.
- `congestionControl: CongestionControl` — The congestion control applied when routing.
- `priority: Priority` — The priority of published data.

**Methods**

- `put(payload: string | Uint8Array, options?: PublisherPutOptions): Promise<void>` — Publishes `payload` on the publisher's key expression.
- `delete(options?: PublisherDeleteOptions): Promise<void>` — Publishes a `Delete` on the publisher's key expression.
- `matchingStatus(): Promise<MatchingStatus>` — Whether any subscribers currently match this publisher's key expression.
- `matchingListener(options?: MatchingListenerOptions): Promise<MatchingListener>` — Declares a [listener](#matching) that fires when subscribers appear or disappear.
- `undeclare(): Promise<void>` — Undeclares the publisher.

QoS is fixed by the publisher, so the per-publication option bags carry only payload-level fields. `sourceInfo` is managed by the advanced builder for sample-miss sequencing and has no setter.

#### `PublisherPutOptions`

| Field | Type | Description |
|---|---|---|
| `encoding` | `string` | Encoding of this payload. |
| `timestamp` | `Timestamp` | Explicit timestamp to stamp the sample with. |
| `attachment` | `Uint8Array` | Arbitrary bytes attached to the message. |

#### `PublisherDeleteOptions`

| Field | Type | Description |
|---|---|---|
| `timestamp` | `Timestamp` | Explicit timestamp to stamp the sample with. |
| `attachment` | `Uint8Array` | Arbitrary bytes attached to the message. |

#### `PublisherOptions`

| Field | Type | Description |
|---|---|---|
| `encoding` | `string` | Default encoding for published data. |
| `congestionControl` | `CongestionControl` | Queue behavior when full. |
| `priority` | `Priority` | Routing priority. |
| `express` | `boolean` | Send unbatched when `true`. |
| `reliability` | `Reliability` | Reliability to route with. |
| `allowedDestination` | `Locality` | Restrict which entities receive the data. |
| `cache` | `CacheConfig` | Enable publisher-side caching of recent samples. |
| `sampleMissDetection` | `MissDetectionConfig` | Enable sample-miss detection (heartbeat). |
| `publisherDetection` | `boolean` | Advertise this publisher for liveliness-based detection. |
| `publisherDetectionMetadata` | `string` | Metadata attached to detection announcements. |

#### `CacheConfig`

| Field | Type | Description |
|---|---|---|
| `maxSamples` | `number` | Maximum number of samples to cache. |
| `repliesConfig` | `RepliesConfig` | QoS for replies served from the cache. |

#### `RepliesConfig`

| Field | Type | Description |
|---|---|---|
| `priority` | `Priority` | Priority of reply samples (default: `Data`). |
| `congestionControl` | `CongestionControl` | Congestion control for reply samples (default: `Block`). |
| `express` | `boolean` | Send reply samples unbatched when `true`. |

#### `MissDetectionConfig`

| Field | Type | Description |
|---|---|---|
| `heartbeat` | `HeartbeatConfig` | Heartbeat settings used to signal sample sequencing. |

#### `HeartbeatConfig`

| Field | Type | Description |
|---|---|---|
| `periodMs` | `number` | **Required.** Heartbeat period in milliseconds. |
| `sporadic` | `boolean` | Emit heartbeats only when needed, rather than periodically. |

---

### Subscribing

A live `Subscriber` declared on a key expression. Obtain one via `session.declareSubscriber(keyExpr, options?)`. Values arrive through its [`handler`](#channel-handlers); the handler type narrows to the channel kind chosen at declare time. Every declared subscriber is an *advanced* subscriber (history, recovery, publisher detection).

#### `Subscriber`

**Properties**

- `keyExpr: KeyExpr` — The key expression this subscription matches.
- `id: EntityGlobalId` — The global id of this subscription entity.
- `handler: SampleHandler` — The receive end delivering [`Sample`](#sample)s. Not iterable; iterate via `subscriber.handler.stream()`.

**Methods**

- `sampleMissListener(options?: SampleMissListenerOptions): Promise<SampleMissListener>` — Declares a [listener](#sample-miss-detection) for samples missed on this subscription. Misses are only detected when the matching publisher enables `sampleMissDetection`.
- `detectPublishers(options?: LivelinessSubscriberOptions): Promise<LivelinessSubscriber>` — Declares a liveliness subscription that detects publishers matching this subscription's key expression. Only publishers that enable `publisherDetection` are detectable; a `Put` marks one appearing, a `Delete` one disappearing.
- `undeclare(): Promise<void>` — Undeclares the subscription.

#### `Sample`

A value delivered to a subscriber (or carried by a reply).

- `payload: Bytes` — The sample's payload.
- `keyExpr: KeyExpr` — The key expression it was published on.
- `kind: SampleKind` — `Put` or `Delete`.
- `encoding: Encoding` — The payload's encoding.
- `timestamp: Timestamp | null` — The sample's timestamp, if any.
- `express: boolean` — Whether it was sent express (unbatched).
- `priority: Priority` — The routing priority it was sent with.
- `congestionControl: CongestionControl` — The congestion control it was routed with.
- `reliability: Reliability` — The reliability it was routed with.
- `attachment: Bytes | null` — Attached bytes, if any.
- `sourceInfo: SourceInfo | null` — Source info, if any.

#### `SubscriberOptions`

| Field | Type | Description |
|---|---|---|
| `allowedOrigin` | `Locality` | Restrict which entities the subscription accepts data from. |
| `handler` | `ChannelConfig` | Channel for the subscription's handler (default: FIFO/256). |
| `history` | `HistoryConfig` | Replay historical samples on declaration. |
| `recovery` | `PeriodicQueriesRecovery \| HeartbeatRecovery` | Strategy for recovering missed samples. |
| `subscriberDetection` | `boolean` | Advertise this subscriber for liveliness-based detection. |
| `subscriberDetectionMetadata` | `string` | Metadata attached to detection announcements. |
| `queryTimeoutMs` | `number` | Timeout (ms) for the advanced subscriber's recovery queries. |

#### `HistoryConfig`

| Field | Type | Description |
|---|---|---|
| `detectLatePublishers` | `boolean` | Also fetch history from publishers that appear later. |
| `maxSamples` | `number` | Maximum number of historical samples to replay. |
| `maxAgeSecs` | `number` | Maximum age (seconds) of historical samples to replay. |

#### `HeartbeatRecovery`

| Field | Type | Description |
|---|---|---|
| `mode` | `'Heartbeat'` | **Required.** Recover the last missed sample by subscribing to publisher heartbeats. |

#### `PeriodicQueriesRecovery`

| Field | Type | Description |
|---|---|---|
| `mode` | `'PeriodicQueries'` | **Required.** Recover missed samples by periodically querying for them. |
| `periodMs` | `number` | **Required.** Query period in milliseconds. |

---

### Querying

The query/reply side. A one-shot query is `session.get(selector, options?)`; a reusable query handle is a `Querier`; the answering side is a `Queryable` that receives `Query` objects and produces `Reply`s.

#### `Querier`

A querier declared on a key expression, fixing its config for every `get`. Obtain one via `session.declareQuerier(keyExpr, options?)`.

**Properties**

- `keyExpr: KeyExpr` — The key expression this querier sends queries on.
- `id: EntityGlobalId` — The global id of this querier entity.
- `congestionControl: CongestionControl` — Congestion control applied when routing queries.
- `priority: Priority` — Priority of this querier's queries.
- `acceptReplies: ReplyKeyExpr` — Whether replies whose key expression doesn't match the query are accepted.

**Methods**

- `get(options?: QuerierGetOptions): Promise<ReplyHandler>` — Sends a query and returns the [reply handler](#channel-handlers). The handler completes (disconnects) once the query is resolved.
- `matchingStatus(): Promise<MatchingStatus>` — Whether any queryables currently match this querier's key expression and target.
- `matchingListener(options?: MatchingListenerOptions): Promise<MatchingListener>` — Declares a [listener](#matching) that fires when matching queryables appear or disappear.
- `undeclare(): Promise<void>` — Undeclares the querier.

#### `Queryable`

A declared queryable that answers matching queries. Obtain one via `session.declareQueryable(keyExpr, options?)`. Incoming [`Query`](#query) objects arrive through its [`handler`](#channel-handlers).

- `keyExpr: KeyExpr` — The key expression this queryable answers queries on.
- `id: EntityGlobalId` — The global id of this queryable entity.
- `handler: QueryHandler` — The receive end delivering incoming queries. Not iterable; iterate via `queryable.handler.stream()`.
- `undeclare(): Promise<void>` — Undeclares the queryable.

#### `Query`

A query received by a queryable — a request to answer with zero or more replies. A query may be answered any number of times; when it is dropped without further replies, Zenoh finalizes it (nothing needs to be called to "close" it).

**Properties**

- `selector: Selector` — The full selector (key expression + parameters).
- `keyExpr: KeyExpr` — The key expression part of the selector.
- `parameters: Parameters` — The parameters part of the selector.
- `payload: Bytes | null` — The query's payload, if any.
- `encoding: Encoding | null` — The payload's encoding, or `null` if there is no payload.
- `attachment: Bytes | null` — Attached bytes, if any.
- `sourceInfo: SourceInfo | null` — The query's source info, if any.
- `acceptReplies: ReplyKeyExpr` — Whether replies whose key expression doesn't match are accepted.
- `priority: Priority` — The priority the reply will be sent with (the query's own).
- `congestionControl: CongestionControl` — The congestion control the reply will be routed with.
- `express: boolean` — Whether the reply is sent express (unbatched).

**Methods** — a reply inherits the query's QoS; `congestionControl` / `priority` are not settable per-reply (Zenoh deprecated them as no-ops).

- `reply(keyExpr: KeyExprArg, payload: string | Uint8Array, options?: ReplyOptions): Promise<void>` — Replies with a `Put` sample on `keyExpr`. By default a query only accepts replies whose key expression intersects its own.
- `replyErr(payload: string | Uint8Array, options?: ReplyErrOptions): Promise<void>` — Replies with an error payload.
- `replyDel(keyExpr: KeyExprArg, options?: ReplyDelOptions): Promise<void>` — Replies with a `Delete` sample on `keyExpr`.

#### `Reply`

A single reply delivered to the querying side.

- `result(): Sample | ReplyError` — The reply's payload: a [`Sample`](#sample) on success, a [`ReplyError`](#replyerror) on failure.
- `replierId: EntityGlobalId | null` — The id of the replier, if known.

#### `ReplyError`

- `payload: Bytes` — The error payload.
- `encoding: Encoding` — The payload's encoding.

#### `QuerierOptions`

| Field | Type | Description |
|---|---|---|
| `target` | `QueryTarget` | Which queryables to target. |
| `consolidation` | `ConsolidationMode` | How replies are consolidated before delivery. |
| `congestionControl` | `CongestionControl` | Queue behavior when full. |
| `priority` | `Priority` | Routing priority. |
| `express` | `boolean` | Send unbatched when `true`. |
| `allowedDestination` | `Locality` | Restrict which entities receive the query. |
| `timeout` | `number` | Query timeout in milliseconds. |
| `acceptReplies` | `ReplyKeyExpr` | Whether to accept replies whose key expression doesn't match. |

#### `QuerierGetOptions`

| Field | Type | Description |
|---|---|---|
| `parameters` | `string \| Parameters` | Parameters appended to the querier's key expression for this query. |
| `payload` | `Uint8Array` | Optional query payload. |
| `encoding` | `string` | Encoding of the query payload. |
| `attachment` | `Uint8Array` | Arbitrary bytes attached to the query. |
| `sourceInfo` | `SourceInfo` | Source id + sequence number to stamp. |
| `cancellationToken` | `CancellationToken` | Token that interrupts the in-flight query. |
| `handler` | `ChannelConfig` | Channel for the reply handler (default: FIFO/256). |

#### `QueryableOptions`

| Field | Type | Description |
|---|---|---|
| `complete` | `boolean` | Advertise this queryable as having the full set of matching data. |
| `allowedOrigin` | `Locality` | Restrict which entities the queryable accepts queries from. |
| `handler` | `ChannelConfig` | Channel for the queryable's handler (default: FIFO/256). |

#### `ReplyOptions`

| Field | Type | Description |
|---|---|---|
| `encoding` | `string` | Encoding of the reply payload. |
| `express` | `boolean` | Send the reply unbatched when `true`. |
| `timestamp` | `Timestamp` | Explicit timestamp to stamp the reply with. |
| `attachment` | `Uint8Array` | Arbitrary bytes attached to the reply. |
| `sourceInfo` | `SourceInfo` | Source id + sequence number to stamp. |

#### `ReplyErrOptions`

| Field | Type | Description |
|---|---|---|
| `encoding` | `string` | Encoding of the error payload. |

#### `ReplyDelOptions`

| Field | Type | Description |
|---|---|---|
| `express` | `boolean` | Send the reply unbatched when `true`. |
| `timestamp` | `Timestamp` | Explicit timestamp to stamp the reply with. |
| `attachment` | `Uint8Array` | Arbitrary bytes attached to the reply. |
| `sourceInfo` | `SourceInfo` | Source id + sequence number to stamp. |

---

### Liveliness

The liveliness sub-API, reached via `session.liveliness()`. A liveliness token asserts a session's liveliness for a key expression; subscribers and queries observe those assertions. `Liveliness` is a borrow-free handle over the session — it has no `undeclare()` of its own.

#### `Liveliness`

- `declareToken(keyExpr: KeyExprArg): Promise<LivelinessToken>` — Declares a liveliness token on `keyExpr`. The token asserts this session's liveliness for that key expression until undeclared or dropped.
- `declareSubscriber(keyExpr: KeyExprArg, options?: LivelinessSubscriberOptions): Promise<LivelinessSubscriber>` — Subscribes to liveliness changes matching `keyExpr`.
- `get(keyExpr: KeyExprArg, options?: LivelinessGetOptions): Promise<ReplyHandler>` — Queries liveliness tokens matching `keyExpr`. Each reply's `Put` sample carries the key expression of a currently-alive token; the handler completes once the query is resolved.

#### `LivelinessToken`

- `undeclare(): Promise<void>` — Undeclares the token (consumes it; idempotent).

#### `LivelinessSubscriber`

A subscription to liveliness changes, also returned by `Subscriber.detectPublishers`.

- `keyExpr: KeyExpr` — The key expression this subscription matches.
- `id: EntityGlobalId` — The global id of this subscription entity.
- `handler: SampleHandler` — The receive end delivering [`Sample`](#sample)s. Not iterable; iterate via `subscriber.handler.stream()`.
- `undeclare(): Promise<void>` — Undeclares the subscription.

#### `LivelinessGetOptions`

| Field | Type | Description |
|---|---|---|
| `timeout` | `number` | Query timeout in milliseconds. |
| `cancellationToken` | `CancellationToken` | Token that interrupts the in-flight query. |
| `handler` | `ChannelConfig` | Channel for the reply handler (default: FIFO/256). |

#### `LivelinessSubscriberOptions`

| Field | Type | Description |
|---|---|---|
| `history` | `boolean` | Replay the currently-matching tokens on declaration. |
| `handler` | `ChannelConfig` | Channel for the subscriber's handler (default: FIFO/256). |

---

### Matching

Whether matching counterparts exist for a publisher or querier.

#### `MatchingListener`

A listener that notifies whenever the matching status of its `Publisher`/`Querier` changes. Declared via `publisher.matchingListener()` or `querier.matchingListener()`.

- `handler: FifoChannelHandlerMatchingStatus | RingChannelHandlerMatchingStatus` — The receive end delivering [`MatchingStatus`](#matchingstatus) values. Not iterable; iterate via `listener.handler.stream()`.
- `undeclare(): Promise<void>` — Undeclares the listener.

#### `MatchingStatus`

- `matching: boolean` — Whether matching entities currently exist.

#### `MatchingListenerOptions`

| Field | Type | Description |
|---|---|---|
| `handler` | `ChannelConfig` | Channel for the listener's handler (default: FIFO/256). |

---

### Sample-miss detection

#### `SampleMissListener`

A listener that notifies of samples missed on a subscription. Declared via `subscriber.sampleMissListener()`; misses are only detected when the matching publisher enables `sampleMissDetection`.

- `handler: FifoChannelHandlerMiss | RingChannelHandlerMiss` — The receive end delivering [`Miss`](#miss) values. Not iterable; iterate via `listener.handler.stream()`.
- `undeclare(): Promise<void>` — Undeclares the listener.

#### `Miss`

- `source: EntityGlobalId` — The source of the missed samples.
- `nb: number` — The number of missed samples.

#### `SampleMissListenerOptions`

| Field | Type | Description |
|---|---|---|
| `handler` | `ChannelConfig` | Channel for the listener's handler (default: FIFO/256). |

---

### Scouting

Discover Zenoh processes (routers/peers/clients) on the network via multicast. Scouting is independent of a session.

#### `Scout`

- `Scout.scout(what: WhatAmIMatcher, config: Config, options?: ScoutOptions): Promise<Scout>` — Scouts for processes matching `what`, using `config` for multicast settings. Keeps scouting until `stop()` is called or it is dropped.
- `handler: HelloHandler` — The receive end delivering [`Hello`](#hello) replies. Not iterable; iterate via `scout.handler.stream()`.
- `stop(): void` — Stops scouting (idempotent). Cancels a local task with no network round-trip, so `Scout` is synchronously `Disposable` (`using`).

#### `Hello`

A discovery reply describing a peer.

- `whatami: WhatAmI` — The kind of node (`Router`, `Peer`, or `Client`).
- `zid: string` — The node's Zenoh id, as a hex string.
- `locators(): Array<Locator>` — The locators the node is reachable at.

#### `WhatAmIMatcher`

A set of node kinds to match while scouting. Built by chaining off `WhatAmIMatcher.empty()`.

- `WhatAmIMatcher.empty(): WhatAmIMatcher` — An empty matcher.
- `router(): WhatAmIMatcher` — Adds `Router` to the matcher (chainable).
- `peer(): WhatAmIMatcher` — Adds `Peer` to the matcher (chainable).
- `client(): WhatAmIMatcher` — Adds `Client` to the matcher (chainable).
- `isEmpty: boolean` — Whether the matcher is empty.
- `matches(w: WhatAmI): boolean` — Whether `w` is in the set.
- `toStr(): string` — The matcher's canonical string form.

```ts
const what = WhatAmIMatcher.empty().router().peer()
using scout = await Scout.scout(what, Config.default())
const hello = await scout.handler.recvAsync()
```

#### `ScoutOptions`

| Field | Type | Description |
|---|---|---|
| `handler` | `ChannelConfig` | Channel for the `Hello` handler (default: FIFO/256). |

---

### Connectivity

The transports and links underlying a session, plus listeners for their lifecycle. Obtained from [`SessionInfo`](#sessioninfo).

#### `Transport`

A connection to a remote Zenoh node. Multiple transports to the same node can coexist; each carries one or more [`Link`](#link)s.

- `zid: string` — The Zenoh id of the remote node, as a hex string.
- `whatami: WhatAmI` — The kind of the remote node.
- `isQos: boolean` — Whether this transport supports QoS.
- `isMulticast: boolean` — Whether this transport is multicast.

#### `Link`

A concrete link within a transport (TCP, UDP, QUIC, …). Obtained from `SessionInfo.links` or a [`LinkEvent`](#linkevent).

- `zid: string` — The Zenoh id of the transport this link belongs to, as a hex string.
- `src: Locator` — The source locator (local endpoint).
- `dst: Locator` — The destination locator (remote endpoint).
- `group: Locator | null` — The group locator (when the link is multicast), or `null`.
- `mtu: number` — The maximum transmission unit, in bytes.
- `isStreamed: boolean` — Whether the link is streamed.
- `interfaces: Array<string>` — The network interfaces associated with the link.
- `authIdentifier: string | null` — The authentication identifier, or `null` if none.
- `priorities: LinkPriorities | null` — The priority range `{ min, max }`, or `null` if the transport does not support QoS.
- `reliability: Reliability | null` — The reliability level, or `null` if the transport does not support QoS.

`LinkPriorities` is `{ min: number; max: number }` — the numeric priority range, which may also include `0` (Control), a value not exposed in the [`Priority`](#enumerations) enum.

#### `TransportEventsListener`

Notifies of transports opening/closing. Declared via `SessionInfo.transportEventsListener`.

- `handler: FifoChannelHandlerTransportEvent | RingChannelHandlerTransportEvent` — Delivers [`TransportEvent`](#transportevent)s. Not iterable; iterate via `listener.handler.stream()`.
- `undeclare(): Promise<void>` — Undeclares the listener.

#### `TransportEvent`

- `kind: SampleKind` — `Put` if the transport opened, `Delete` if it closed.
- `transport: Transport` — The transport this event is about.

#### `LinkEventsListener`

Notifies of links being added/removed. Declared via `SessionInfo.linkEventsListener`.

- `handler: FifoChannelHandlerLinkEvent | RingChannelHandlerLinkEvent` — Delivers [`LinkEvent`](#linkevent)s. Not iterable; iterate via `listener.handler.stream()`.
- `undeclare(): Promise<void>` — Undeclares the listener.

#### `LinkEvent`

- `kind: SampleKind` — `Put` if the link was added, `Delete` if it was removed.
- `link: Link` — The link this event is about.

---

### Channel handlers

Every receiving entity exposes a **handler** — the receive end of its channel. The handler's surface depends on the channel kind selected via the `handler` option:

| Handler | Value type | Produced by |
|---|---|---|
| `FifoChannelHandlerSample` / `RingChannelHandlerSample` | `Sample` | `Subscriber`, `LivelinessSubscriber` |
| `FifoChannelHandlerReply` / `RingChannelHandlerReply` | `Reply` | `Session.get`, `Querier.get`, `Liveliness.get` |
| `FifoChannelHandlerQuery` / `RingChannelHandlerQuery` | `Query` | `Queryable` |
| `FifoChannelHandlerHello` / `RingChannelHandlerHello` | `Hello` | `Scout` |
| `FifoChannelHandlerMatchingStatus` / `RingChannelHandlerMatchingStatus` | `MatchingStatus` | `MatchingListener` |
| `FifoChannelHandlerMiss` / `RingChannelHandlerMiss` | `Miss` | `SampleMissListener` |
| `FifoChannelHandlerTransportEvent` / `RingChannelHandlerTransportEvent` | `TransportEvent` | `TransportEventsListener` |
| `FifoChannelHandlerLinkEvent` / `RingChannelHandlerLinkEvent` | `LinkEvent` | `LinkEventsListener` |

The `index.js` facade narrows the handler type by the channel kind you pass, so `declareSubscriber('k')` yields a `Subscriber<FifoChannelHandlerSample>` and `declareSubscriber('k', { handler: { kind: 'Ring' } })` a `Subscriber<RingChannelHandlerSample>`.

For `T` the value type, **FIFO handlers** expose the full surface:

**Receiving**

- `recvAsync(): Promise<T>` — Resolves with the next value when one is available. Rejects once the channel disconnects (all senders dropped).
- `tryRecv(): T | null` — Receives without blocking; `null` if the channel is currently empty.
- `recv(): Promise<T>` — Like `recvAsync`, but backed by Zenoh's synchronous blocking `recv` run on a worker thread, so the wait never freezes the event loop. Rejects on disconnect.
- `recvTimeout(timeoutMs: number): Promise<T | null>` — Resolves with the next value, or `null` if `timeoutMs` elapses first. Rejects on disconnect.
- `recvDeadline(deadlineMs: number): Promise<T | null>` — Resolves with the next value, or `null` once the wall-clock `deadlineMs` (epoch ms, e.g. `Date.now() + 50`) passes. Rejects on disconnect.
- `drain(): Array<T>` — Takes every currently-queued value as an array, without blocking. Unlike repeated `tryRecv`, no further values are fetched once this snapshot is taken.
- `stream()` — Returns this handler's matching `*Stream` class (e.g. `SampleStream` for `Sample`, `ReplyStream` for `Reply` — the full list is below): an async-iterator object for `for await…of`. The handler itself is not iterable; iteration lives here.

**Introspection**

- `len: number` — Number of values currently queued.
- `capacity: number | null` — The channel's bound, or `null` if unbounded.
- `isEmpty: boolean` — Whether the channel currently holds no values.
- `isFull: boolean` — Whether the channel is at capacity.
- `senderCount: number` — Number of senders feeding the channel.
- `receiverCount: number` — Number of receivers sharing the channel.
- `isDisconnected: boolean` — Whether the channel has disconnected (all senders dropped).
- `sameChannel(other): boolean` — Whether `other` is a handle to the same underlying channel.

**Ring handlers** expose only the receiving variants — `recvAsync`, `tryRecv`, `recv`, `recvTimeout`, `recvDeadline` — with the same semantics, except "disconnected" means the ring's strong owner has been dropped. They have no `drain`, `stream`, introspection, or `sameChannel`.

```ts
// FIFO: iterate with for await
for await (const sample of subscriber.handler.stream()) {
  console.log(sample.payload.toString())
}

// Ring: receive the most recent value
const sample = await ringSubscriber.handler.recvAsync()
```

Each FIFO handler's `stream()` returns a matching `*Stream` class (`SampleStream`, `ReplyStream`, `QueryStream`, `HelloStream`, `MatchingStatusStream`, `MissStream`, `TransportEventStream`, `LinkEventStream`) implementing `[Symbol.asyncIterator](): AsyncGenerator<T, void, undefined>`.

---

### Key expressions & selectors

#### `KeyExpr`

A validated, canonical key expression. Accepted anywhere `KeyExprArg` is.

- `new KeyExpr(expr: string)` — Constructs a key expression, rejecting any string that isn't canon.
- `KeyExpr.autocanonize(expr: string): KeyExpr` — Canonizes the input before validating it.
- `KeyExpr.fromStr(expr: string): KeyExpr` — Equivalent to the constructor.
- `asStr: string` — The canonical string form.
- `isWild: boolean` — Whether it contains any wildcard (`**` or `$*`).
- `concat(other: string): KeyExpr` — String concatenation. Prefer `join` so Zenoh can exploit the hierarchy.
- `join(other: string): KeyExpr` — Joins both sides, inserting a `/` between them. The preferred way to concatenate path segments.
- `intersects(other: string | KeyExpr): boolean` — Whether the two sets share at least one key.
- `includes(other: string | KeyExpr): boolean` — Whether `self`'s set contains every key of `other`'s.

#### `Selector`

A key expression plus optional parameters. Accepted anywhere `SelectorArg` is.

- `new Selector(keyExpr: string | KeyExpr, parameters?: string)` — Builds a selector.
- `keyExpr: KeyExpr` — The key expression part.
- `parameters: Parameters` — The parameters part.
- `split(): SelectorParts` — Deconstructs into `{ keyExpr: string; parameters: string }`.

#### `Parameters`

The `?`-parameters of a selector, in `a=b;c=d|e;f=g` form.

- `Parameters.empty(): Parameters` — Empty parameters.
- `new Parameters(params: string)` — Parses parameters from a string.
- `asStr: string` — The canonical string form.
- `isEmpty: boolean` — Whether there are no parameters.
- `isOrdered: boolean` — Whether all keys are in alphabetical order.
- `containsKey(key: string): boolean` — Whether `key` is present.
- `get(key: string): string | null` — The value for `key`, if present.
- `values(key: string): Array<string>` — All values for `key`.
- `insert(key: string, value: string): string | null` — Inserts a pair, returning the previous value if any.
- `remove(key: string): string | null` — Removes a key, returning its value if present.
- `extend(other: Parameters): void` — Extends with the entries of `other`.

---

### Payloads & encoding

#### `Bytes`

A Zenoh byte buffer — the type of every payload.

- `Bytes.new(): Bytes` — An empty buffer.
- `Bytes.fromBytes(data: Uint8Array): Bytes` — From raw bytes.
- `Bytes.fromString(value: string): Bytes` — From a UTF-8 string.
- `isEmpty: boolean` — Whether the buffer is empty.
- `len: number` — The byte length.
- `toBytes(): Uint8Array` — The contents as a `Uint8Array`.
- `toString(): string` — The contents decoded as a string.

#### `Encoding`

A content-type hint carried with payloads. Construct from a string or one of the many predefined constants.

- `Encoding.default(): Encoding` — The default encoding (equivalent to `zenohBytes()`).
- `Encoding.from(value: string): Encoding` — From a content-type string (e.g. `'text/plain;utf-8'`).
- `toString(): string` — The canonical string form.
- `withSchema(value: string): Encoding` — A copy with the given schema suffix appended.

Predefined static constructors mirror Zenoh's catalog, e.g. `zenohBytes()`, `zenohString()`, `zenohSerialized()`, `applicationOctetStream()`, `textPlain()`, `applicationJson()`, `textJson()`, `applicationCdr()`, `applicationCbor()`, `applicationYaml()`, `textYaml()`, `textJson5()`, `applicationProtobuf()`, `imagePng()`, `imageJpeg()`, `imageGif()`, `imageBmp()`, `imageWebp()`, `applicationXml()`, `textHtml()`, `textXml()`, `textCss()`, `textJavascript()`, `textMarkdown()`, `textCsv()`, `applicationSql()`, the `audio*` / `video*` families, and more. Each returns an `Encoding`.

```ts
const enc = Encoding.textPlain().withSchema('utf-8')
await session.put('demo/text', 'hi', { encoding: enc.toString() })
```

---

### Serialization

Zenoh's streaming serialization format. Write values with a `Serializer`, read them back in the same order with a `Deserializer`.

#### `Serializer`

Serializing values one after another is equivalent to serializing a tuple of them. `finish()` consumes the serializer; it cannot be used afterwards.

- `new Serializer()` — A fresh serializer.
- Fixed-width numbers: `i8`, `i16`, `i32`, `u8`, `u16`, `u32` `(value: number): void`; `i64`, `u64`, `i128`, `u128` `(value: bigint): void`; `f32`, `f64` `(value: number): void`.
- `bool(value: boolean): void`, `string(value: string): void`.
- `bytes(value: Uint8Array): void` — A byte blob (LEB128 length prefix + raw bytes); wire-compatible with `Vec<u8>` / `ZBytes`.
- `stringArray(value: Array<string>): void` — A string sequence (LEB128 count + each string); wire-compatible with `Vec<String>`.
- Typed arrays: `int8Array`, `int16Array`, `int32Array`, `uint16Array`, `uint32Array`, `float32Array`, `float64Array`, `bigInt64Array`, `bigUint64Array`.
- `varint(value: bigint): void` — A `usize` as an LEB128 variable-length integer; the length/count prefix for hand-rolled sequences, maps, and sets.
- `finish(): Bytes` — Consumes the serializer and returns the serialized [`Bytes`](#bytes). Throws if already finished.

#### `Deserializer`

Read values in the same order they were written. Use `done` to check whether the buffer is fully consumed.

- `new Deserializer(data: Bytes)` — A deserializer over `data`.
- `done: boolean` — `true` when there is no data left to deserialize.
- Fixed-width numbers: `i8`, `i16`, `i32`, `u8`, `u16`, `u32` `(): number`; `i64`, `u64`, `i128`, `u128` `(): bigint`; `f32`, `f64` `(): number`.
- `bool(): boolean`, `string(): string`, `bytes(): Uint8Array`, `stringArray(): Array<string>`.
- Typed arrays: `int8Array`, `int16Array`, `int32Array`, `uint16Array`, `uint32Array`, `float32Array`, `float64Array`, `bigInt64Array`, `bigUint64Array`.
- `varint(): bigint` — An LEB128 variable-length `usize` (the count prefix written by `Serializer.varint`).

```ts
const s = new Serializer()
s.u16(500)
s.string('test')
const bytes = s.finish()

const d = new Deserializer(bytes)
d.u16()      // 500
d.string()   // 'test'
d.done       // true
```

---

### Endpoints & locators

#### `EndPoint`

An endpoint in canonical form `<protocol>/<address>[?<metadata>][#<config>]`.

- `new EndPoint(s: string)` — Parses an endpoint from its canonical string.
- `protocol: string` — The protocol.
- `address: string` — The address.
- `asStr: string` — The canonical string form.
- `metadata(): Metadata` — The metadata view.
- `config(): string` — The config substring.
- `split(): EndPointParts` — Splits into `{ protocol, address, metadata, config }` strings.
- `toLocator(): Locator` — Demotes to a `Locator`, dropping any config component.

#### `Locator`

A locator — an endpoint without a config component.

- `new Locator(protocol: string, address: string, metadata: string)` — Constructs from parts.
- `protocol: string` — The protocol.
- `address: string` — The address.
- `asStr: string` — The canonical string form.
- `metadata(): Metadata` — The metadata view.
- `toEndpoint(): EndPoint` — Promotes to an `EndPoint`.

#### `Metadata`

The metadata view of an endpoint or locator.

- `Metadata.reliabilityKey(): string` — The key for reliability (`"rel"`).
- `Metadata.prioritiesKey(): string` — The key for priorities (`"prio"`).
- `Metadata.multistreamKey(): string` — The key for multistream (`"multistream"`).
- `Metadata.mixedReliabilityKey(): string` — The key for mixed reliability (`"mixed_rel"`).
- `asStr(): string` — The metadata substring in canonical form.
- `isEmpty(): boolean` — Whether there is no metadata.
- `get(key: string): string | null` — The first value for `key`, if any.
- `values(key: string): Array<string>` — Every value for `key`.

---

### Identifiers, time & cancellation

#### `EntityGlobalId`

The global id of an entity (session, publisher, subscriber, …).

- `zid: string` — The Zenoh id of the owning session, as a hex string.
- `eid: number` — The entity id, unique within that session.

#### `SourceInfo`

Identifies the origin of a message for sequencing.

- `new SourceInfo(sourceId: EntityGlobalId, sourceSn: number)` — Constructs from an entity id and sequence number.
- `sourceId: EntityGlobalId` — The source entity's global id.
- `sourceSn: number` — The source sequence number.

#### `Timestamp`

A hybrid-logical-clock timestamp.

- `Timestamp.parseRfc3339(s: string): Timestamp` — Parses `<rfc3339>/<hlc_id_hex>`.
- `toStringRfc3339Lossy(): string` — RFC3339 with nanosecond precision, e.g. `"2024-07-01T13:51:12.129693000Z/33"`.
- `getTime(): bigint` — The NTP64 time as its raw `u64`.
- `getId(): string` — The HLC's unique id, as a hex string.
- `getDiffDuration(other: Timestamp): number` — The time difference from `other`, in milliseconds.

#### `TimeRange`

A time range — for example, the value of a selector's time parameter.

> **Note:** `TimeRange` is exported for completeness, but no API currently returns one, and it has no public constructor — there is presently no way to obtain an instance.

- `start: string | null` — The start bound (time expression), or `null` if unbounded.
- `end: string | null` — The end bound (time expression), or `null` if unbounded.
- `contains(epochMillis: number): boolean` — Whether the given instant (epoch ms, e.g. `Date.now()`) is in range. `now(...)` offsets resolve against the current system time on each call.

#### `CancellationToken`

Interrupts in-flight `get` queries it was passed to (via the `cancellationToken` option).

- `new CancellationToken()` — A fresh, uncancelled token.
- `cancel(): Promise<void>` — Interrupts all associated queries. Resolves once cancellation completes; on failure some operations might not be cancelled.
- `isCancelled: boolean` — Whether `cancel()` has been called.

---

### Configuration

#### `Config`

The configuration used to open a session.

- `Config.default(): Config` — The default configuration.
- `Config.defaultConfigPathEnv(): string` — The environment variable name read by `fromEnv`.
- `Config.fromEnv(): Config` — Loads from the file path in the `defaultConfigPathEnv` variable.
- `Config.fromFile(path: string): Config` — Loads from the file at `path`.
- `Config.fromJson5(input: string): Config` — Loads from a JSON5 string.
- `getJson(key: string): string` — A JSON string of the configuration at `key`.
- `insertJson5(key: string, value: string): void` — Inserts the JSON5 `value` at `key`.
- `remove(key: string): void` — Removes the value at `key`.

```ts
const config = Config.default()
config.insertJson5('mode', '"peer"')
config.insertJson5('scouting/multicast/enabled', 'false')
const session = await Session.open(config)
```

---

### Enumerations

All enumerations are plain string-literal unions — compare against and pass string literals directly (e.g. `priority: 'DataHigh'`).

| Type | Values | Meaning |
|---|---|---|
| `ChannelKind` | `'Fifo'` · `'Ring'` | Which channel backs a handler. `Fifo` back-pressures and drops nothing; `Ring` keeps the most recent `capacity` values, dropping oldest. |
| `CongestionControl` | `'Drop'` · `'Block'` · `'BlockFirst'` | Behavior when the queue is full: drop the message; wait for progress; or block only the first message and drop the rest. |
| `ConsolidationMode` | `'Auto'` · `'None'` · `'Monotonic'` · `'Latest'` | How replies are consolidated. `Auto` per the queryable's preference; `None` no consolidation; `Monotonic` forward immediately, dropping superseded; `Latest` only the highest-timestamp sample per key. |
| `Priority` | `'RealTime'` · `'InteractiveHigh'` · `'InteractiveLow'` · `'DataHigh'` · `'Data'` · `'DataLow'` · `'Background'` | Routing priority of a message, highest to lowest. |
| `Reliability` | `'BestEffort'` · `'Reliable'` | Reliability applied when routing data. |
| `Locality` | `'SessionLocal'` · `'Remote'` · `'Any'` | Restricts which entities (relative to this session) data is routed to/from. |
| `QueryTarget` | `'BestMatching'` · `'All'` · `'AllComplete'` | Which queryables a query targets: the best match; all matching; or all matching declared `complete`. |
| `ReplyKeyExpr` | `'Any'` · `'MatchingQuery'` | Whether replies whose key expression doesn't match the query are accepted. |
| `SampleKind` | `'Put'` · `'Delete'` | Whether a sample is a put or a delete. |
| `WhatAmI` | `'Router'` · `'Peer'` · `'Client'` | The kind of a Zenoh node. |
