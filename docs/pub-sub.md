# Publish & Subscribe Guide

The [Quick Start](./quick-start.md) covers declaring a publisher and subscriber and
moving messages between them. This guide goes deeper into the options on
`session.declarePublisher(key, options)` and `session.declareSubscriber(key, options)`,
organized by the problem each one solves: replaying history to late joiners, detecting
and recovering lost samples, discovering peers, tuning quality of service, and managing
backpressure.

Both entities are Zenoh's **advanced** publisher/subscriber under the hood, so these
features come from the options — not from separate APIs.

Every snippet is a complete, runnable program — see the [Quick Start](./quick-start.md#prerequisites)
for setup and how to run them.

## Tuning delivery (QoS)

`PublisherOptions` carries the quality-of-service knobs Zenoh applies to every message.
Set them once at declare time and they apply to all `put`/`delete` calls:

- **`priority`** — the scheduling class, from `'RealTime'` down through
  `'InteractiveHigh'`, `'InteractiveLow'`, `'DataHigh'`, `'Data'` (default), `'DataLow'`,
  to `'Background'`. Higher-priority traffic is sent first when links are busy.
- **`congestionControl`** — what to do when a queue is full: `'Drop'` (default for the
  `Data` priorities — shed messages rather than wait), `'Block'` (apply backpressure and
  wait for space), or `'BlockFirst'`.
- **`reliability`** — `'Reliable'` or `'BestEffort'`.
- **`express`** — `true` sends each sample immediately instead of batching it with
  others; lower latency, lower throughput.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
await using subscriber = await session.declareSubscriber('demo/qos')
await using publisher = await session.declarePublisher('demo/qos', {
  priority: 'DataHigh',
  congestionControl: 'Block',
  reliability: 'Reliable',
  express: true,
})

console.log('priority:', publisher.priority)             // DataHigh
console.log('congestion:', publisher.congestionControl)  // Block

await publisher.put('urgent')
const sample = await subscriber.recv()
console.log('sample priority:', sample.priority, '| reliability:', sample.reliability)
// → sample priority: DataHigh | reliability: Reliable
```

The QoS a sample was sent with is readable back off the received `Sample` (`priority`,
`congestionControl`, `reliability`, `express`).

## Scoping traffic to the host (Locality)

By default a publisher reaches every matching subscriber, local or remote. Use
`allowedDestination` (publisher) and `allowedOrigin` (subscriber) to restrict that to a
`Locality`: `'SessionLocal'`, `'Remote'`, or `'Any'` (the default). This is how you keep
intra-process traffic off the network, or make a subscriber ignore its own session's
publications.

```ts
import { Config, Session } from '@diskette/dialtone'
import { setTimeout as sleep } from 'node:timers/promises'

await using session = await Session.open(Config.default())

// This subscriber only accepts publications from within its own session.
await using subscriber = await session.declareSubscriber('demo/loc', {
  allowedOrigin: 'SessionLocal',
})

// A publisher aimed only at remote peers reaches nothing on this host.
await using remote = await session.declarePublisher('demo/loc', {
  allowedDestination: 'Remote',
})
await remote.put('to remote only')
await sleep(200)
console.log('from remote-scoped publisher:', subscriber.tryRecv()) // null

// A session-local publisher does reach it.
await using local = await session.declarePublisher('demo/loc', {
  allowedDestination: 'SessionLocal',
})
await local.put('to local')
await sleep(200)
console.log('from local-scoped publisher:', subscriber.tryRecv()?.payload.tryToString())
// → to local
```

## Serving late joiners (cache + history)

By default a subscriber only sees samples published *after* it declares. To let a
subscriber that joins late catch up, pair a publisher **cache** with a subscriber
**history** request:

- `cache: { maxSamples }` — the publisher retains its last N samples and answers history
  queries for them. `repliesConfig` tunes the QoS of those replies.
- `history: { maxSamples, maxAgeSecs, detectLatePublishers }` — on declare, the
  subscriber queries matching caches and replays what they hold. `detectLatePublishers`
  keeps picking up publishers that appear afterward.

> **Cache and sample-miss detection need timestamping.** Both attach sequence numbers to
> every sample, which requires timestamping on the session. Open the session with
> `timestamping/enabled` set to `true` (as below) or `declarePublisher` throws:
> *"the 'timestamping' setting must be enabled in the Zenoh configuration."*

```ts
import { Config, Session } from '@diskette/dialtone'

const config = Config.default()
config.insertJson5('timestamping/enabled', 'true')
await using session = await Session.open(config)

// Publisher caches its recent samples.
await using publisher = await session.declarePublisher('demo/history', {
  cache: { maxSamples: 10 },
})
for (let i = 0; i < 3; i++) await publisher.put(`sample ${i}`)

// A subscriber declared *after* those puts still receives them.
await using subscriber = await session.declareSubscriber('demo/history', {
  history: { maxSamples: 10 },
})
for (let i = 0; i < 3; i++) {
  console.log('replayed:', (await subscriber.recv()).payload.tryToString())
}
// → replayed: sample 0
// → replayed: sample 1
// → replayed: sample 2
```

## Reliable delivery (miss detection + recovery)

Over a lossy link, samples can be dropped. Zenoh can detect the gap and recover the
missing samples from the publisher's cache. It takes cooperation from both sides:

- **Publisher** — `sampleMissDetection: { heartbeat: { periodMs, sporadic } }` numbers
  samples and periodically advertises the latest sequence number so subscribers can spot
  a gap even when the stream goes quiet. Keep a `cache` too, so lost samples can be
  refetched.
- **Subscriber** — `recovery` turns on retransmission requests. Two strategies:
  - `{ mode: 'Heartbeat' }` — react to the publisher's heartbeats.
  - `{ mode: 'PeriodicQueries', periodMs }` — proactively poll for anything missed.

```ts
import { Config, Session } from '@diskette/dialtone'

const config = Config.default()
config.insertJson5('timestamping/enabled', 'true')
await using session = await Session.open(config)

await using publisher = await session.declarePublisher('demo/reliable', {
  cache: { maxSamples: 100 },
  sampleMissDetection: { heartbeat: { periodMs: 500 } },
})

await using subscriber = await session.declareSubscriber('demo/reliable', {
  history: { maxSamples: 100 },
  recovery: { mode: 'Heartbeat' },
})

await publisher.put('reliable payload')
console.log('received:', (await subscriber.recv()).payload.tryToString())
// → received: reliable payload
```

On a single host there is nothing to lose, so recovery never triggers here — but the
same declaration is what makes delivery resilient across a real network.

## Reacting to gaps (the sample-miss listener)

When a subscriber does detect missed samples, `sampleMissListener()` reports them. Each
`Miss` names the `source` publisher (`EntityGlobalId`) and `nb`, the number of samples
missed. Pair it with a publisher that has `sampleMissDetection` enabled.

```ts
import { Config, Session } from '@diskette/dialtone'

const config = Config.default()
config.insertJson5('timestamping/enabled', 'true')
await using session = await Session.open(config)

await using publisher = await session.declarePublisher('demo/gaps', {
  cache: { maxSamples: 100 },
  sampleMissDetection: { heartbeat: { periodMs: 500 } },
})
await using subscriber = await session.declareSubscriber('demo/gaps', {
  recovery: { mode: 'Heartbeat' },
})

await using misses = await subscriber.sampleMissListener()

// Report every gap as it's detected. On a lossy link this logs the source and count;
// on a local session it simply stays quiet.
void (async () => {
  for await (const miss of misses.stream()) {
    console.log(`missed ${miss.nb} sample(s) from ${miss.source.zid}`)
  }
})()

await publisher.put('data')
console.log('received:', (await subscriber.recv()).payload.tryToString())
```

## Presence and discovery

Sometimes you need to know *who* is out there, not just move data. There are two
complementary tools.

**Does anyone hear me?** A publisher can watch whether any subscriber currently matches
its key. `matchingStatus()` is a snapshot; `matchingListener()` streams every change.
Use it to skip expensive work when nobody is listening.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
await using publisher = await session.declarePublisher('demo/presence')

console.log('anyone matching?', (await publisher.matchingStatus()).matching) // false

await using listener = await publisher.matchingListener()

// A subscriber appears → the listener reports the change.
await using subscriber = await session.declareSubscriber('demo/presence')
console.log('now matching?', (await listener.recv()).matching) // true
```

**Which publishers exist?** With the publisher advertising `publisherDetection: true`,
a subscriber's `detectPublishers()` returns a liveliness subscriber that emits a `Put`
sample when a matching publisher appears and a `Delete` when one goes away. (The sample's
`keyExpr` is the publisher's internal liveliness token, not your data key.)

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
await using subscriber = await session.declareSubscriber('demo/detect')
await using detector = await subscriber.detectPublishers()

await using publisher = await session.declarePublisher('demo/detect', {
  publisherDetection: true,
})

const event = await detector.recv()
console.log('publisher appeared:', event.kind) // Put
```

The mirror image, `subscriberDetection: true` on the subscriber, makes subscribers
discoverable to the matching machinery above.

## Liveliness tokens (group membership)

Liveliness is a general presence primitive, independent of any data flow. An instance
declares a **token** on a key; while it lives, subscribers on that key see it as alive,
and when it drops (undeclare, or the process dies) they're notified. This is the
idiomatic way to track group membership or service presence.

A liveliness subscriber receives a `Put` sample when a token appears and a `Delete` when
it disappears. Pass `history: true` to also replay tokens already alive at declare time.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
const liveliness = session.liveliness()

await using members = await liveliness.declareSubscriber('group/**', { history: true })

const token = await liveliness.declareToken('group/member-1')
const joined = await members.recv()
console.log('joined:', joined.kind, joined.keyExpr.toString()) // Put group/member-1

token.undeclare()
const left = await members.recv()
console.log('left:', left.kind, left.keyExpr.toString())       // Delete group/member-1
```

`liveliness().get(keyExpr)` also does a one-shot query for tokens currently alive,
returning a reply handler you consume like any other.

## Backpressure (Fifo vs Ring channels)

A subscriber buffers incoming samples in a channel. The `channel` option picks the
strategy — the same choice applies to any receiving entity (subscribers, listeners,
query handlers):

- **`FifoChannel(capacity)`** (the default) — a bounded queue. When it fills, delivery
  applies backpressure. Use it when every sample matters.
- **`RingChannel(capacity)`** — keeps the most recent `capacity` samples and drops the
  oldest. Use it for a real-time consumer that only cares about the latest state and
  must not fall behind.

```ts
import { Config, Session, RingChannel } from '@diskette/dialtone'
import { setTimeout as sleep } from 'node:timers/promises'

await using session = await Session.open(Config.default())
await using publisher = await session.declarePublisher('demo/chan')

// Keep only the single most recent sample.
await using subscriber = await session.declareSubscriber('demo/chan', {
  channel: new RingChannel(1),
})

for (let i = 0; i < 5; i++) await publisher.put(`msg ${i}`)
await sleep(100)

let kept = 0
let last: string | undefined
for (let s = subscriber.tryRecv(); s !== null; s = subscriber.tryRecv()) {
  last = s.payload.tryToString()
  kept++
}
console.log(`ring kept ${kept}, last: ${last}`)
// → ring kept 1, last: msg 4
```

## Option reference

| Goal | Publisher option | Subscriber option |
| --- | --- | --- |
| Prioritize / control congestion | `priority`, `congestionControl`, `reliability`, `express` | — |
| Scope to local or remote | `allowedDestination` | `allowedOrigin` |
| Replay history to late joiners | `cache` | `history` |
| Detect & recover lost samples | `sampleMissDetection` (+ `cache`) | `recovery` (+ `history`) |
| Be notified of gaps | `sampleMissDetection` | `subscriber.sampleMissListener()` |
| Know if anyone is listening | `publisher.matchingStatus()` / `matchingListener()` | — |
| Discover publishers | `publisherDetection` | `subscriber.detectPublishers()` |
| Be discoverable | — | `subscriberDetection` |
| Choose backpressure strategy | — | `channel` (`FifoChannel` / `RingChannel`) |

## See also

- [Quick Start](./quick-start.md) — the basics of sessions, publishing, and receiving.
- [Key Expressions](./key-expressions.md) — the topic language publishers and subscribers match on.
- [Querying Guide](./querying.md) — the pull-based half: `get`, queryables, and queriers.
- [Configuration & Sessions](./configuration.md) — modes, endpoints, scouting, and enabling `timestamping`.
- [Serialization & Payloads](./serialization.md) — `Bytes`, `Encoding`, and the `zd` schema system.
- [Lifecycle & Cleanup](./lifecycle.md) — releasing entities with `using` / `await using` or by hand.
