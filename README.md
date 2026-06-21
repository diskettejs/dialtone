# `@diskette/dialtone`

Node.js native bindings for [Zenoh](https://zenoh.io) — a pub/sub/query protocol — built with [NAPI-RS](https://napi.rs).

The bindings mirror Zenoh's surface as faithfully as the JS boundary allows: the operations, options, and lifecycle semantics are Zenoh's. Advanced pub/sub (from `zenoh-ext`) is folded into the regular `Publisher`/`Subscriber` surface — every declared publisher and subscriber is an advanced one. This package is **Node.js only**; there are no WASM/WASI builds for the browser.

## Install

```bash
pnpm add @diskette/dialtone
```

```ts
import { Session } from '@diskette/dialtone'
```

## Quick start

```ts
import { Session } from '@diskette/dialtone'

// Open a session (peer mode, default config).
await using session = await Session.open()

// Subscribe, then publish, then receive.
const subscriber = await session.declareSubscriber('demo/example/**')
await session.put('demo/example/greeting', 'hello')

const sample = await subscriber.recv()
console.log(sample!.keyExpr.toString(), sample!.payload.toString()) // demo/example/greeting hello

subscriber.undeclare()
// Leaving the `await using` scope closes the session.
```

Subscribers, queryables, queriers, scouts, and reply/query channels are **async iterables**, so the receive loop is usually a `for await`:

```ts
const subscriber = await session.declareSubscriber('demo/example/**')
for await (const sample of subscriber) {
  console.log(sample.payload.toString())
}
```

---

# API Reference

## Exports at a glance

| Export                                                                        | Kind             | Summary                                                                           |
| ----------------------------------------------------------------------------- | ---------------- | --------------------------------------------------------------------------------- |
| [`Session`](#session)                                                         | class            | An open connection to the Zenoh network; the entry point for everything else.     |
| [`SessionInfo`](#sessioninfo)                                                 | class            | Identity of a session and the routers/peers it is connected to.                   |
| [`Config`](#config)                                                           | class            | Session configuration; built via static factory methods.                          |
| [`KeyExpr`](#keyexpr)                                                         | class            | A `/`-separated key expression addressing a set of keys.                          |
| [`Publisher`](#publisher)                                                     | class            | A publisher bound to a key expression, with QoS fixed at declaration.             |
| [`Subscriber`](#subscriber)                                                   | class            | Delivers [`Sample`](#sample)s through a channel.                                  |
| [`Sample`](#sample)                                                           | class            | A received data sample: payload plus metadata.                                    |
| [`Queryable`](#queryable)                                                     | class            | Delivers incoming [`Query`](#query)s through a channel.                           |
| [`Query`](#query)                                                             | class            | A received query, answered with `reply` / `replyErr` / `replyDel`.                |
| [`Querier`](#querier)                                                         | class            | A reusable query handle, the query analog of a `Publisher`.                       |
| [`Replies`](#replies)                                                         | class            | Delivers [`ReplySample`](#replysample) / [`ReplyError`](#replyerror) for a `get`. |
| [`ReplySample`](#replysample) / [`ReplyError`](#replyerror)                   | class            | The two arms of a query reply.                                                    |
| [`Liveliness`](#liveliness) / [`LivelinessToken`](#livelinesstoken)           | class            | Declare, query, and subscribe to liveliness.                                      |
| [`MatchingListener`](#matchinglistener) / [`MatchingStatus`](#matchingstatus) | class / type     | Notifications about whether matching entities exist.                              |
| [`SampleMissListener`](#samplemisslistener) / [`Miss`](#miss)                 | class / type     | Notifications of missed samples (advanced subscribers).                           |
| [`scout`](#scout) / [`Scout`](#scout-class) / [`Hello`](#hello)               | function / class | Discover Zenoh nodes on the network.                                              |
| Option interfaces & enums                                                     | types            | See [Options](#options) and [Enums & string unions](#enums--string-unions).       |

Everywhere a key expression is accepted you may pass a `string` **or** a [`KeyExpr`](#keyexpr); getters that expose one return a `KeyExpr`. Payloads accept a `string` or a `Uint8Array` (a `Buffer` works); received payloads are returned as a `Buffer`.

---

## Session

An open connection to the Zenoh network — the entry point from which every publisher, subscriber, and query is declared.

```ts
class Session {
  static open(config?: Config | null): Promise<Session>
  close(): Promise<void>

  put(
    keyExpr: string | KeyExpr,
    payload: string | Uint8Array,
    options?: PutOptions | null,
  ): Promise<void>
  delete(keyExpr: string | KeyExpr, options?: DeleteOptions | null): Promise<void>
  get(selector: string, options?: GetOptions | null): Promise<Replies>

  declarePublisher(keyExpr: string | KeyExpr, options?: PublisherOptions | null): Promise<Publisher>
  declareSubscriber(
    keyExpr: string | KeyExpr,
    options?: SubscriberOptions | null,
  ): Promise<Subscriber>
  declareQueryable(keyExpr: string | KeyExpr, options?: QueryableOptions | null): Promise<Queryable>
  declareQuerier(keyExpr: string | KeyExpr, options?: QuerierOptions | null): Promise<Querier>

  liveliness(): Liveliness
  newTimestamp(): Timestamp
  info(): SessionInfo

  get zid(): string // this session's Zenoh ID, as a hex string
  get isClosed(): boolean

  [Symbol.asyncDispose](): Promise<void> // == close(); enables `await using`
}
```

- **`open(config?)`** — opens a session with the given [`Config`](#config), or the default configuration when omitted.
- **`close()`** — closes the session, undeclaring every entity declared on it.
- **`put` / `delete`** — publish a `Put` / `Delete` sample to every subscriber whose key expression matches.
- **`get(selector, …)`** — query a selector (a key expression optionally followed by `?params`) and receive replies through a [`Replies`](#replies) channel.
- **`declare*`** — declare a [`Publisher`](#publisher), [`Subscriber`](#subscriber), [`Queryable`](#queryable), or [`Querier`](#querier). Every publisher and subscriber is an _advanced_ one — see [Advanced pub/sub](#advanced-pubsub).
- **`liveliness()`** — access the session's [`Liveliness`](#liveliness) API.
- **`newTimestamp()`** — mint a [`Timestamp`](#timestamp) from this session's hybrid logical clock, to pass back via the `timestamp` publication option.

## SessionInfo

Returned by [`Session.info()`](#session). Each accessor is async.

```ts
class SessionInfo {
  zid(): Promise<string> // this session's Zenoh ID
  routersZid(): Promise<string[]> // connected routers' Zenoh IDs
  peersZid(): Promise<string[]> // connected peers' Zenoh IDs
}
```

---

## Config

Session configuration (mirrors `zenoh::Config`). The tree is opaque — construct it with a factory method; there is intentionally no public constructor.

```ts
class Config {
  static default(): Config // peer mode, default endpoints
  static fromJson5(json5: string): Config
  static fromFile(path: string): Config
  static fromEnv(): Config // path from the ZENOH_CONFIG env var
}
```

---

## KeyExpr

A Zenoh key expression: a `/`-separated expression addressing a set of keys (mirrors `zenoh::key_expr::KeyExpr`). To be valid it must be _canon_.

```ts
class KeyExpr {
  constructor(keyExpr: string) // throws if not canon
  static autocanonize(keyExpr: string): KeyExpr // canonizes first

  intersects(other: string | KeyExpr): boolean // share ≥1 matching key?
  includes(other: string | KeyExpr): boolean // does this match everything `other` does?
  equals(other: string | KeyExpr): boolean

  join(other: string): KeyExpr // insert a `/` separator (preferred over concat)
  concat(other: string): KeyExpr // no separator inserted
  toString(): string // canon string form
}
```

```ts
const ke = new KeyExpr('demo/zenoh-ts/**')
ke.intersects('demo/zenoh-ts/value') // true
KeyExpr.autocanonize('demo/**/**/x').toString() // 'demo/**/x'
```

---

## Publishing

### Publisher

A publisher bound to a key expression, with QoS fixed at declaration time (via [`PublisherOptions`](#publisheroptions)). Create one with [`Session.declarePublisher`](#session). Because every publisher is an advanced publisher and owns sequencing, `put`/`delete` may override only payload-level fields, never QoS.

```ts
class Publisher {
  put(payload: string | Uint8Array, options?: PublisherPutOptions | null): Promise<void>
  delete(options?: PublisherDeleteOptions | null): Promise<void>

  matchingStatus(): Promise<MatchingStatus>
  matchingListener(handler?: ChannelHandler | null): Promise<MatchingListener>

  undeclare(): void // subsequent operations error; resolves synchronously

  get keyExpr(): KeyExpr
  get encoding(): string
  get congestionControl(): CongestionControl
  get priority(): Priority
  get reliability(): Reliability
  get id(): EntityGlobalId

  [Symbol.dispose](): void // == undeclare(); enables `using`
}
```

- **`matchingStatus()`** — whether any subscribers currently match.
- **`matchingListener(handler?)`** — a [`MatchingListener`](#matchinglistener) that notifies whenever the set of matching subscribers changes.

### MatchingListener

Notifies of changes to a publisher's (or querier's) [`MatchingStatus`](#matchingstatus). Obtain it from [`Publisher.matchingListener`](#publisher) or [`Querier.matchingListener`](#querier).

```ts
class MatchingListener {
  recv(): Promise<MatchingStatus | null> // null once closed
  tryRecv(): MatchingStatus | null // null if empty; throws once closed
  undeclare(): void
  [Symbol.asyncIterator](): AsyncGenerator<MatchingStatus, void, undefined>
  [Symbol.dispose](): void
}
```

### MatchingStatus

```ts
interface MatchingStatus {
  matching: boolean // true if ≥1 matching entity currently exists
}
```

---

## Subscribing

### Subscriber

Delivers [`Sample`](#sample)s through a channel. Consume it with `for await (const sample of subscriber)`, or pull samples with `recv()` / `tryRecv()`. Create one with [`Session.declareSubscriber`](#session) (configured via [`SubscriberOptions`](#subscriberoptions)).

```ts
class Subscriber {
  recv(): Promise<Sample | null> // null once undeclared, or closed & drained
  tryRecv(): Sample | null // null if empty; throws once disconnected
  undeclare(): void

  // Advanced-only (not available on liveliness subscribers):
  sampleMissListener(handler?: ChannelHandler | null): Promise<SampleMissListener>
  detectPublishers(handler?: ChannelHandler | null): Promise<Subscriber>

  get keyExpr(): KeyExpr
  get id(): EntityGlobalId

  [Symbol.asyncIterator](): AsyncGenerator<Sample, void, undefined>
  [Symbol.dispose](): void
}
```

- **`recv` vs `tryRecv`** — `recv()` awaits the next sample and yields `null` at end of stream. `tryRecv()` never blocks: it returns `null` when the buffer is _empty_ but **throws** once the subscriber has _disconnected_, letting a polling loop distinguish "nothing yet" from "closed".
- **`undeclare()`** — ends iteration; buffered samples are _dropped_ with the handler (as in Zenoh), not drained.
- **`sampleMissListener` / `detectPublishers`** — see [Advanced pub/sub](#advanced-pubsub). These reject on liveliness subscribers.

### Sample

A received data sample (or query reply value): the payload plus all its metadata. Fields are lazy getters; the payload is only copied into a `Buffer` when accessed.

```ts
class Sample {
  get keyExpr(): KeyExpr
  get payload(): Buffer
  get kind(): SampleKind // 'Put' | 'Delete'
  get encoding(): string
  get timestamp(): Timestamp | null
  get congestionControl(): CongestionControl
  get priority(): Priority
  get express(): boolean
  get reliability(): Reliability
  get attachment(): Buffer | null
  get sourceInfo(): SourceInfo | null
}
```

---

## Querying

### Session.get / Replies

`session.get(selector, options?)` issues a query and returns a [`Replies`](#replies) channel of [`ReplySample`](#replysample) / [`ReplyError`](#replyerror). A query is not a declared entity, so there is nothing to undeclare — the channel ends once every queryable has answered or the query times out.

```ts
class Replies {
  recv(): Promise<ReplySample | ReplyError | null> // null once complete & drained
  tryRecv(): ReplySample | ReplyError | null // null if none ready; throws once done
  [Symbol.asyncIterator](): AsyncGenerator<ReplySample | ReplyError, void, undefined>
}
```

Discriminate the two arms with `if (reply.sample)`:

```ts
const replies = await session.get('demo/example/**')
for await (const reply of replies) {
  if (reply.sample) {
    console.log('value:', reply.sample.payload.toString())
  } else {
    console.log('error:', reply.payload.toString())
  }
}
```

#### ReplySample

```ts
class ReplySample {
  get sample(): Sample
  get replierId(): EntityGlobalId | null
}
```

#### ReplyError

```ts
class ReplyError {
  get sample(): null // always null — the discriminant against ReplySample
  get payload(): Buffer
  get encoding(): string
  get replierId(): EntityGlobalId | null
}
```

### Queryable

Delivers incoming [`Query`](#query)s through a channel. Create one with [`Session.declareQueryable`](#session) ([`QueryableOptions`](#queryableoptions)).

```ts
class Queryable {
  recv(): Promise<Query | null> // null once undeclared, or closed & drained
  tryRecv(): Query | null // null if empty; throws once disconnected
  undeclare(): void
  get keyExpr(): KeyExpr
  get id(): EntityGlobalId
  [Symbol.asyncIterator](): AsyncGenerator<Query, void, undefined>
  [Symbol.dispose](): void
}
```

### Query

A query received by a [`Queryable`](#queryable), answered with `reply` / `replyErr` / `replyDel` (any number of times, including none). The query is finalized when dropped, so keep it alive until you have sent every reply you intend to.

```ts
class Query {
  get selector(): string // key expression + parameters
  get keyExpr(): KeyExpr
  get parameters(): string // the part after `?`
  get payload(): Buffer | null
  get encoding(): string | null
  get attachment(): Buffer | null
  get sourceInfo(): SourceInfo | null
  get acceptsReplies(): ReplyKeyExpr
  get priority(): Priority
  get congestionControl(): CongestionControl
  get express(): boolean

  reply(
    keyExpr: string | KeyExpr,
    payload: string | Uint8Array,
    options?: ReplyOptions | null,
  ): Promise<void>
  replyErr(payload: string | Uint8Array, options?: ReplyErrOptions | null): Promise<void>
  replyDel(keyExpr: string | KeyExpr, options?: ReplyDelOptions | null): Promise<void>
}
```

```ts
const queryable = await session.declareQueryable('demo/example/q')
for await (const query of queryable) {
  await query.reply('demo/example/q', 'answer')
}
```

### Querier

A querier bound to a key expression, with query settings fixed at declaration time ([`QuerierOptions`](#querieroptions)) — the query analog of a [`Publisher`](#publisher). Create one with [`Session.declareQuerier`](#session); per-`get`, only the payload and parameters may vary.

```ts
class Querier {
  get(options?: QuerierGetOptions | null): Promise<Replies>
  matchingStatus(): Promise<MatchingStatus>
  matchingListener(handler?: ChannelHandler | null): Promise<MatchingListener>
  undeclare(): void

  get keyExpr(): KeyExpr
  get congestionControl(): CongestionControl
  get priority(): Priority
  get acceptReplies(): ReplyKeyExpr
  get id(): EntityGlobalId

  [Symbol.dispose](): void
}
```

---

## Advanced pub/sub

Advanced pub/sub (from `zenoh-ext`) is integrated into the regular surface: every declared publisher and subscriber is an advanced one, so the capabilities live directly on the `declarePublisher` / `declareSubscriber` options.

- **Cache & history** — a publisher with [`cache`](#publisheroptions) keeps recent samples; a late-joining subscriber with [`history`](#subscriberoptions) queries that cache on startup and replays it.
- **Miss detection & recovery** — a publisher with [`sampleMissDetection`](#publisheroptions) tags samples with sequence numbers; a subscriber can then observe gaps via [`sampleMissListener`](#samplemisslistener) and request retransmission via [`recovery`](#recoveryconfig). Recovery requires the publisher to enable **both** `cache` and `sampleMissDetection`.
- **Entity detection** — a publisher with [`publisherDetection`](#publisheroptions) advertises itself (via liveliness); a subscriber's [`detectPublishers()`](#subscriber) then sees each matching publisher appear as a `Put` and vanish as a `Delete`. Symmetrically, [`subscriberDetection`](#subscriberoptions) advertises a subscriber.

```ts
const publisher = await session.declarePublisher('demo/adv', {
  cache: { maxSamples: 10 },
  sampleMissDetection: { heartbeat: { periodMs: 500 } },
  publisherDetection: true,
})

const subscriber = await session.declareSubscriber('demo/adv', {
  history: { detectLatePublishers: true, maxSamples: 10 },
  recovery: { heartbeat: true },
})
```

### SampleMissListener

Notifies of samples a subscriber detected as missed. Obtain it from [`Subscriber.sampleMissListener`](#subscriber). Misses are only detectable from publishers that enable `sampleMissDetection`.

```ts
class SampleMissListener {
  recv(): Promise<Miss | null> // null once closed
  tryRecv(): Miss | null // null if empty; throws once closed
  undeclare(): void
  [Symbol.asyncIterator](): AsyncGenerator<Miss, void, undefined>
  [Symbol.dispose](): void
}
```

### Miss

```ts
interface Miss {
  source: EntityGlobalId // the publisher the missed samples were from
  nb: number // how many consecutive samples were missed
}
```

---

## Liveliness

Declare liveliness tokens, query existing ones, and subscribe to liveliness changes. Obtain it with [`Session.liveliness()`](#session).

```ts
class Liveliness {
  declareToken(keyExpr: string | KeyExpr): Promise<LivelinessToken>
  declareSubscriber(
    keyExpr: string | KeyExpr,
    options?: LivelinessSubscriberOptions | null,
  ): Promise<Subscriber>
  get(keyExpr: string | KeyExpr, options?: LivelinessGetOptions | null): Promise<Replies>
}
```

A subscriber over liveliness receives a `Put` sample when a matching token appears and a `Delete` when one vanishes.

### LivelinessToken

A token whose liveliness is tied to the [`Session`](#session) that declared it; monitoring applications see it as alive until it is undeclared or dropped (or the session loses connectivity / stops). Tokens are automatically undeclared when dropped.

```ts
class LivelinessToken {
  undeclare(): void // subsequent calls are no-ops; resolves synchronously
  [Symbol.dispose](): void
}
```

```ts
const liveliness = session.liveliness()
const subscriber = await liveliness.declareSubscriber('demo/liveliness/**')
const token = await liveliness.declareToken('demo/liveliness/token')

;(await subscriber.recv())!.kind // 'Put'  — token appeared
token.undeclare()
;(await subscriber.recv())!.kind // 'Delete' — token vanished
```

---

## Scouting

### scout

```ts
function scout(
  what: WhatAmI[], // node kinds to scout for; [] matches all
  config: Config,
  handler?: ChannelHandler | null,
): Promise<Scout>
```

Spawns a task that periodically sends scout messages and delivers the [`Hello`](#hello) replies through the returned [`Scout`](#scout-class) handle.

### Scout (class)

```ts
class Scout {
  recv(): Promise<Hello | null> // null once stopped
  tryRecv(): Hello | null // null if empty; throws once stopped
  stop(): void
  [Symbol.asyncIterator](): AsyncGenerator<Hello, void, undefined>
  [Symbol.dispose](): void // == stop()
}
```

### Hello

A discovered node's identity, kind, and reachable locators (mirrors `zenoh::scouting::Hello`).

```ts
class Hello {
  get whatami(): WhatAmI
  get zid(): string // the node's Zenoh ID, as a hex string
  get locators(): string[]
}
```

```ts
const handle = await scout(['Peer', 'Router', 'Client'], Config.default())
for await (const hello of handle) {
  console.log(hello.whatami, hello.zid, hello.locators)
}
```

---

## Channels & async iteration

Subscribers, queryables, queriers, scouts, listeners, and the `get` reply stream all deliver items through a channel and share the same shape:

- **`for await (const item of x)`** — the idiomatic consumer; iteration ends (the generator returns) when the channel closes.
- **`recv(): Promise<Item | null>`** — await one item; resolves to `null` at end of stream.
- **`tryRecv(): Item | null`** — non-blocking: `null` while the buffer is _empty_, but **throws** once the channel has _closed_, so a polling loop can tell "nothing yet" from "done".

The backing channel is chosen with a [`ChannelHandler`](#channelhandler):

```ts
// FIFO (default): bounded queue, applies backpressure to Zenoh when full.
await session.declareSubscriber('demo/**', { handler: { kind: 'Fifo', capacity: 256 } })

// Ring: bounded ring buffer, drops the oldest item when full (never blocks).
await session.declareSubscriber('demo/**', { handler: { kind: 'Ring', capacity: 1 } })
```

## Resource management

Every declared entity exposes `undeclare()` (or `stop()` for a scout, `close()` for a session), and each is wired to a disposal symbol so it works with explicit resource management:

- [`Session`](#session) is **`AsyncDisposable`** → `await using session = await Session.open()`; scope exit awaits `close()`.
- `Publisher`, `Subscriber`, `Queryable`, `Querier`, `MatchingListener`, `SampleMissListener`, `LivelinessToken`, and `Scout` are **`Disposable`** → `using sub = await session.declareSubscriber(...)`; scope exit calls `undeclare()` / `stop()` synchronously.

```ts
{
  await using session = await Session.open()
  using sub = await session.declareSubscriber('demo/example/**')
  await session.put('demo/example/x', 'hi')
  console.log((await sub.recv())!.payload.toString())
} // LIFO: sub.undeclare() runs, then session.close() is awaited.
```

---

## Options

### PutOptions

For [`Session.put`](#session).

| Field                | Type                                      | Default    | Notes                                             |
| -------------------- | ----------------------------------------- | ---------- | ------------------------------------------------- |
| `encoding`           | `string`                                  | —          | e.g. `"text/plain"`, `"application/json"`.        |
| `attachment`         | `string \| Uint8Array`                    | —          | Carried alongside the payload.                    |
| `congestionControl`  | [`CongestionControl`](#congestioncontrol) | `Drop`     |                                                   |
| `priority`           | [`Priority`](#priority)                   | `Data`     |                                                   |
| `express`            | `boolean`                                 | `false`    | Send unbatched (lower latency, lower throughput). |
| `reliability`        | [`Reliability`](#reliability)             | `Reliable` |                                                   |
| `allowedDestination` | [`Locality`](#locality)                   | `Any`      | Which matching subscribers receive the data.      |
| `timestamp`          | [`Timestamp`](#timestamp)                 | —          | From [`Session.newTimestamp`](#session).          |
| `sourceInfo`         | [`SourceInfo`](#sourceinfo)               | —          | Producing entity + sequence number.               |

### DeleteOptions

For [`Session.delete`](#session). Same as [`PutOptions`](#putoptions) minus `encoding`: `attachment`, `congestionControl` (default `Drop`), `priority` (`Data`), `express`, `reliability` (`Reliable`), `allowedDestination` (`Any`), `timestamp`, `sourceInfo`.

### GetOptions

For [`Session.get`](#session).

| Field                | Type                                      | Default         | Notes                             |
| -------------------- | ----------------------------------------- | --------------- | --------------------------------- |
| `payload`            | `string \| Uint8Array`                    | —               | Sent alongside the query.         |
| `encoding`           | `string`                                  | —               | Encoding of the query payload.    |
| `attachment`         | `string \| Uint8Array`                    | —               |                                   |
| `congestionControl`  | [`CongestionControl`](#congestioncontrol) | `Block`         |                                   |
| `priority`           | [`Priority`](#priority)                   | `Data`          |                                   |
| `express`            | `boolean`                                 | `false`         |                                   |
| `target`             | [`QueryTarget`](#querytarget)             | `BestMatching`  | Which queryables answer.          |
| `consolidation`      | [`ConsolidationMode`](#consolidationmode) | `Auto`          | How replies are consolidated.     |
| `allowedDestination` | [`Locality`](#locality)                   | `Any`           |                                   |
| `timeout`            | `number`                                  | —               | Milliseconds to wait for replies. |
| `acceptReplies`      | [`ReplyKeyExpr`](#replykeyexpr)           | `MatchingQuery` |                                   |
| `sourceInfo`         | [`SourceInfo`](#sourceinfo)               | —               |                                   |
| `handler`            | [`ChannelHandler`](#channelhandler)       | FIFO            | Backs reply delivery.             |

### PublisherOptions

For [`Session.declarePublisher`](#session). These are fixed for the publisher's lifetime; per-publication `put`/`delete` may override only payload fields, not QoS.

| Field                        | Type                                          | Default    | Notes                                                       |
| ---------------------------- | --------------------------------------------- | ---------- | ----------------------------------------------------------- |
| `encoding`                   | `string`                                      | —          | Default encoding for publications.                          |
| `congestionControl`          | [`CongestionControl`](#congestioncontrol)     | `Drop`     |                                                             |
| `priority`                   | [`Priority`](#priority)                       | `Data`     |                                                             |
| `express`                    | `boolean`                                     | `false`    |                                                             |
| `reliability`                | [`Reliability`](#reliability)                 | `Reliable` |                                                             |
| `allowedDestination`         | [`Locality`](#locality)                       | `Any`      |                                                             |
| `cache`                      | [`CacheConfig`](#cacheconfig)                 | —          | Cache recent samples for history/recovery.                  |
| `sampleMissDetection`        | [`MissDetectionConfig`](#missdetectionconfig) | —          | Tag samples with sequence numbers.                          |
| `publisherDetection`         | `boolean`                                     | `false`    | Advertise via liveliness so subscribers can detect it.      |
| `publisherDetectionMetadata` | `string`                                      | —          | Metadata appended to the detection token / cache queryable. |

### PublisherPutOptions

For [`Publisher.put`](#publisher): `encoding` (overrides the publisher default), `attachment`, `timestamp` (overrides the publisher's automatic timestamp).

### PublisherDeleteOptions

For [`Publisher.delete`](#publisher): `attachment`, `timestamp`.

### SubscriberOptions

For [`Session.declareSubscriber`](#session).

| Field                         | Type                                | Default | Notes                                                               |
| ----------------------------- | ----------------------------------- | ------- | ------------------------------------------------------------------- |
| `allowedOrigin`               | [`Locality`](#locality)             | `Any`   | Which publishers' samples are accepted.                             |
| `handler`                     | [`ChannelHandler`](#channelhandler) | FIFO    |                                                                     |
| `history`                     | [`HistoryConfig`](#historyconfig)   | —       | Query for historical data on startup.                               |
| `recovery`                    | [`RecoveryConfig`](#recoveryconfig) | —       | Ask for retransmission of detected lost samples.                    |
| `subscriberDetection`         | `boolean`                           | `false` | Advertise this subscriber via liveliness.                           |
| `subscriberDetectionMetadata` | `string`                            | —       | Metadata appended to the detection token.                           |
| `queryTimeoutMs`              | `number`                            | —       | Timeout for the queries this subscriber issues (history, recovery). |

### QueryableOptions

For [`Session.declareQueryable`](#session).

| Field           | Type                                | Default | Notes                                                                                                           |
| --------------- | ----------------------------------- | ------- | --------------------------------------------------------------------------------------------------------------- |
| `complete`      | `boolean`                           | `false` | Whether this queryable can answer the full queried key expression alone (reachable by `AllComplete` targeting). |
| `allowedOrigin` | [`Locality`](#locality)             | `Any`   | Which queriers' queries are accepted.                                                                           |
| `handler`       | [`ChannelHandler`](#channelhandler) | FIFO    |                                                                                                                 |

### QuerierOptions

For [`Session.declareQuerier`](#session). Fixed for the querier's lifetime; per-`get`, only payload and parameters vary.

| Field                | Type                                      | Default         | Notes                             |
| -------------------- | ----------------------------------------- | --------------- | --------------------------------- |
| `congestionControl`  | [`CongestionControl`](#congestioncontrol) | `Block`         |                                   |
| `priority`           | [`Priority`](#priority)                   | `Data`          |                                   |
| `express`            | `boolean`                                 | `false`         |                                   |
| `target`             | [`QueryTarget`](#querytarget)             | `BestMatching`  |                                   |
| `consolidation`      | [`ConsolidationMode`](#consolidationmode) | `Auto`          |                                   |
| `allowedDestination` | [`Locality`](#locality)                   | `Any`           |                                   |
| `timeout`            | `number`                                  | —               | Milliseconds to wait for replies. |
| `acceptReplies`      | [`ReplyKeyExpr`](#replykeyexpr)           | `MatchingQuery` |                                   |

### QuerierGetOptions

For [`Querier.get`](#querier): `parameters` (the part after `?`), `payload`, `encoding`, `attachment`, `sourceInfo`, `handler`.

### ReplyOptions / ReplyErrOptions / ReplyDelOptions

For the [`Query`](#query) reply methods.

- **`ReplyOptions`** (`reply`): `encoding`, `attachment`, `express`, `timestamp`, `sourceInfo`.
- **`ReplyErrOptions`** (`replyErr`): `encoding`.
- **`ReplyDelOptions`** (`replyDel`): `attachment`, `express`, `timestamp`, `sourceInfo`.

### LivelinessSubscriberOptions

For [`Liveliness.declareSubscriber`](#liveliness): `history` (`boolean`, default `false` — when `true`, query the network for currently-live tokens on declaration, delivering each as a `Put`), `handler`.

### LivelinessGetOptions

For [`Liveliness.get`](#liveliness): `timeout` (ms; defaults to the session's configured query timeout), `handler`.

### CacheConfig

Attaches a cache to a publisher so matching subscribers can recover history and/or missed samples.

| Field           | Type                              | Default | Notes                                  |
| --------------- | --------------------------------- | ------- | -------------------------------------- |
| `maxSamples`    | `number`                          | `1`     | Samples kept per resource.             |
| `repliesConfig` | [`RepliesConfig`](#repliesconfig) | —       | QoS for replies served from the cache. |

### RepliesConfig

QoS applied to the samples a publisher's cache sends back: `priority` (default `Data`), `congestionControl` (default `Block`), `express`.

### MissDetectionConfig

Enables sample-miss detection on a publisher (per-publisher sequence numbers). Optional `heartbeat`: a [`HeartbeatConfig`](#heartbeatconfig) that additionally allows the _last_ sample to be recovered.

### HeartbeatConfig

| Field      | Type      | Notes                                                                                                          |
| ---------- | --------- | -------------------------------------------------------------------------------------------------------------- |
| `periodMs` | `number`  | Heartbeat period, in milliseconds.                                                                             |
| `sporadic` | `boolean` | When `true`, advertise the sequence number only when it changed (`sporadicHeartbeat`); otherwise every period. |

### HistoryConfig

Enables a subscriber to query for historical samples on startup (served by publishers that enable `cache`): `detectLatePublishers` (`boolean`), `maxSamples` (`number`, per resource), `maxAgeSecs` (`number`).

### RecoveryConfig

Configures recovery of detected lost samples. Exactly **one** mode must be set — they are mutually exclusive, checked at declaration time:

| Field               | Type      | Notes                                                    |
| ------------------- | --------- | -------------------------------------------------------- |
| `heartbeat`         | `boolean` | Recover by subscribing to publisher heartbeats.          |
| `periodicQueriesMs` | `number`  | Recover by querying for missed samples on this interval. |

Recovery requires the matching publisher to enable both `cache` and `sampleMissDetection`.

### ChannelHandler

```ts
interface ChannelHandler {
  kind: ChannelType // 'Fifo' | 'Ring'
  capacity?: number // defaults to Zenoh's default channel size
}
```

### SourceInfo

```ts
interface SourceInfo {
  sourceId: EntityGlobalId // the entity that produced the sample
  sourceSn: number // the source's sequence number for it
}
```

### EntityGlobalId

```ts
interface EntityGlobalId {
  zid: string // owning session's Zenoh ID, as a hex string
  eid: number // session-local entity id
}
```

### Timestamp

A hybrid-logical-clock time plus the id of the clock that produced it. Obtain one from [`Session.newTimestamp`](#session).

```ts
interface Timestamp {
  time: bigint // NTP64-encoded time component (64-bit)
  id: string // source clock id, as a hex string (a Zenoh ID)
}
```

---

## Enums & string unions

All of these are plain string-literal unions.

### ChannelType

`'Fifo'` (bounded FIFO, applies backpressure when full) · `'Ring'` (bounded ring buffer, drops the oldest item when full — never blocks).

### CongestionControl

`'Drop'` (default for `put`/`delete`) · `'Block'` (wait for queue room) · `'BlockFirst'` (block only the first such message; drop the rest).

### Priority

Highest → lowest, default `Data`:
`'RealTime'` · `'InteractiveHigh'` · `'InteractiveLow'` · `'DataHigh'` · `'Data'` · `'DataLow'` · `'Background'`.

### Reliability

`'BestEffort'` (may be lost) · `'Reliable'` (default). _Note: as in Zenoh, reliability does not currently trigger wire retransmission; it is a marker that may influence link selection._

### Locality

`'SessionLocal'` (same session only) · `'Remote'` (other sessions only) · `'Any'` (default).

### QueryTarget

`'BestMatching'` (default) · `'All'` (every matching queryable) · `'AllComplete'` (every matching queryable declared `complete`).

### ConsolidationMode

`'Auto'` (default) · `'None'` (no consolidation) · `'Monotonic'` (forward immediately, drop older-or-equal timestamps per key) · `'Latest'` (only the latest-timestamped sample per key).

### ReplyKeyExpr

`'Any'` (reply key need not match the query) · `'MatchingQuery'` (default; only replies whose key matches the query).

### SampleKind

`'Put'` · `'Delete'`.

### WhatAmI

`'Router'` · `'Peer'` (default mode) · `'Client'`.

---

## Release package

Ensure you have set your **NPM_TOKEN** in the `GitHub` project setting.

In `Settings -> Secrets`, add **NPM_TOKEN** into it.

When you want to release the package:

```bash
npm version [<newversion> | major | minor | patch | premajor | preminor | prepatch | prerelease [--preid=<prerelease-id>] | from-git]

git push
```

GitHub actions will do the rest job for you.

> WARN: Don't run `npm publish` manually.
