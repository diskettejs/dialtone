# Querying Guide (get & reply)

Publish/subscribe pushes data to whoever is listening. **Querying** is Zenoh's other
half: a pull-based request/response. A client issues a `get` on a selector; every
matching **queryable** receives the query and sends back zero or more replies, which the
client collects. It's the model behind lookups, RPC-style calls, and distributed storage.

This guide covers the three entry points and the options that shape them:

- `session.get(selector, options)` — a one-shot query.
- `session.declareQueryable(keyExpr, options)` — serve queries.
- `session.declareQuerier(keyExpr, options)` — a reusable query handle for repeated gets.

Every snippet is a complete, runnable program — see the [Quick Start](./quick-start.md#prerequisites)
for setup and how to run them.

## A one-shot query

`session.get` returns a `ReplyHandler` — the same channel-backed receiver as a
subscriber, so consume replies with `recv()`, `tryRecv()`, or `stream()`. Each `Reply`
is either a value or an error: `reply.result.sample` holds a `Sample` on success, and
`reply.result.error` holds a `ReplyError` on failure. The stream ends when the query
finalizes.

This program declares a queryable to answer, then queries it:

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
await using queryable = await session.declareQueryable('demo/lookup/**')

// Serve incoming queries.
void (async () => {
  for await (const query of queryable.stream()) {
    await query.reply(query.keyExpr, 'the answer is 42')
    query.drop()
  }
})()

// Issue the query and collect replies.
const replies = await session.get('demo/lookup/value')
for await (const reply of replies.stream()) {
  const { result } = reply
  if (result.error) console.log('error:', result.error.payload.tryToString())
  else console.log('reply:', result.sample.payload.tryToString())
}
// → reply: the answer is 42
```

The queryable calls `query.drop()` after each reply to finalize the query; the next
section explains what that releases.

`get` accepts the same QoS and delivery options as `put` (`congestionControl`,
`priority`, `express`, `allowedDestination`), plus a `timeout` (ms), a `payload` to send
with the query, and a `channel` to pick the reply buffer strategy.

## Serving queries

A queryable receives each request as a `Query`. Beyond its `keyExpr`, a query carries an
optional `payload` (with `encoding`), an `attachment`, and its `parameters` (see below).
Respond with one of three methods, then finalize:

- `query.reply(keyExpr, payload, options?)` — a value (a `Put` sample to the client).
- `query.replyErr(payload, options?)` — an error.
- `query.replyDel(keyExpr, options?)` — a deletion (a `Delete` sample).

A single query may be answered with **multiple** `reply` calls — for a wildcard query,
send one reply per matching key.

When you're done, **finalize** the query with `query.drop()`.

> **Drop your queries promptly.** A queryable holds the `Query` open while it responds,
> and the client's `get` stays open until every queryable it reached finishes. Calling
> `query.drop()` releases it immediately; otherwise the query lingers until the `Query`
> is garbage-collected or the query times out (10 s by default).

> **A reply's key must match the query.** By default the key you reply on must intersect
> the query's key expression, or `reply`/`replyDel` reject on the queryable side
> (*"…does not intersect with query…, despite query only allowing replies on matching key
> expressions"*). Replying on `query.keyExpr` always satisfies this. A `Querier` can lift
> the restriction with `acceptReplies: 'Any'`; `session.get` has no such option, so its
> replies must always match.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
await using queryable = await session.declareQueryable('demo/svc/**')

void (async () => {
  for await (const query of queryable.stream()) {
    const payload = query.payload?.tryToString() ?? '(none)'
    console.log(`serving ${query.keyExpr} with request payload: ${payload}`)
    await query.reply(query.keyExpr, 'done')
    query.drop()
  }
})()

const replies = await session.get('demo/svc/run', { payload: 'please run' })
for await (const reply of replies.stream()) {
  if (!reply.result.error) console.log('client got:', reply.result.sample.payload.tryToString())
}
// → serving demo/svc/run with request payload: please run
// → client got: done
```

`declareQueryable` takes `complete` (does this queryable hold the *entire* data set for
its key — see targets below), `allowedOrigin` (a `Locality`, to accept only local or
remote queries), and `channel`.

## Selectors and parameters

A selector is a key expression plus optional query parameters: `demo/sensors/**?unit=C;since=10`.
Parameters are a `;`-separated map the client passes and the queryable reads — use them
to filter, paginate, or otherwise shape the response. Build a selector as a string, or
with the `Selector` and `Parameters` classes.

```ts
import { Config, Session, Parameters } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
await using queryable = await session.declareQueryable('demo/sensors/**')

void (async () => {
  for await (const query of queryable.stream()) {
    const params = query.parameters
    const unit = params.get('unit') ?? 'K'
    await query.reply('demo/sensors/temp', `21 ${unit}`)
    query.drop()
  }
})()

// String form: "keyExpr?params"
const replies = await session.get('demo/sensors/temp?unit=C')
for await (const reply of replies.stream()) {
  if (!reply.result.error) console.log('string selector:', reply.result.sample.payload.tryToString())
}

// Or pass parameters separately.
const params = new Parameters('unit=F')
const replies2 = await session.get('demo/sensors/temp', { parameters: params })
for await (const reply of replies2.stream()) {
  if (!reply.result.error) console.log('Parameters object:', reply.result.sample.payload.tryToString())
}
// → string selector: 21 C
// → Parameters object: 21 F
```

`Parameters` also supports `insert`, `containsKey`, `remove`, and `values(key)` — the
last splits a `|`-separated list value (e.g. `fields=id|name` → `['id', 'name']`).

## Errors and deletions

The client discriminates a reply on `result.error`. Here one queryable answers three
ways depending on a parameter:

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
await using queryable = await session.declareQueryable('demo/q/**')

void (async () => {
  for await (const query of queryable.stream()) {
    switch (query.parameters.get('mode')) {
      case 'err':
        await query.replyErr('something failed')
        break
      case 'del':
        await query.replyDel('demo/q/item')
        break
      default:
        await query.reply('demo/q/item', 'ok')
    }
    query.drop()
  }
})()

for (const mode of ['ok', 'err', 'del']) {
  const replies = await session.get(`demo/q/item?mode=${mode}`)
  for await (const reply of replies.stream()) {
    const { result } = reply
    if (result.error) console.log(`[${mode}] error:`, result.error.payload.tryToString())
    else console.log(`[${mode}] ${result.sample.kind}:`, result.sample.payload.tryToString() || '(empty)')
  }
}
// → [ok] Put: ok
// → [err] error: something failed
// → [del] Delete: (empty)
```

## Fan-out and consolidation

A query can reach many queryables; `target` and `consolidation` control how widely it
fans out and how the replies are reconciled.

- **`target`** — `'BestMatching'` (default; query the best set of queryables),
  `'All'` (every matching queryable), or `'AllComplete'` (only queryables declared with
  `complete: true`).
- **`consolidation`** — how to dedup replies that share a key: `'Auto'` (default),
  `'None'` (every reply), `'Monotonic'`, or `'Latest'` (only the newest per key).

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())

// Two complete queryables on the same key.
async function serve(name: string) {
  const q = await session.declareQueryable('demo/multi', { complete: true })
  void (async () => {
    for await (const query of q.stream()) {
      await query.reply('demo/multi', `from ${name}`)
      query.drop()
    }
  })()
  return q
}
await using q1 = await serve('queryable-1')
await using q2 = await serve('queryable-2')

// Default consolidation dedups replies that share a key — you get one survivor.
const consolidated = await session.get('demo/multi', { target: 'AllComplete' })
for await (const reply of consolidated.stream()) {
  if (!reply.result.error) console.log('consolidated:', reply.result.sample.payload.tryToString())
}
// → consolidated: from queryable-2   (one survivor; which one is not guaranteed)

// consolidation: 'None' keeps every queryable's reply.
const all = await session.get('demo/multi', { target: 'AllComplete', consolidation: 'None' })
for await (const reply of all.stream()) {
  if (!reply.result.error) console.log('all:', reply.result.sample.payload.tryToString())
}
// → all: from queryable-1
// → all: from queryable-2   (order not guaranteed)
```

## A reusable querier

When you query the same key expression repeatedly, `declareQuerier` amortizes the
declaration and fixes the query options once. Its `get` takes only the per-call options
(`payload`, `parameters`, `attachment`, `channel`), and it exposes `matchingStatus()` /
`matchingListener()` to learn whether any queryable is reachable — the mirror of a
publisher's matching API.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
await using queryable = await session.declareQueryable('demo/svc/**', { complete: true })

void (async () => {
  for await (const query of queryable.stream()) {
    await query.reply('demo/svc/time', `echo ${query.payload?.tryToString() ?? ''}`)
    query.drop()
  }
})()

await using querier = await session.declareQuerier('demo/svc/time', {
  target: 'All',
  consolidation: 'Latest',
  timeout: 5000,
})

console.log('reachable?', (await querier.matchingStatus()).matching) // true

for (let i = 0; i < 3; i++) {
  const replies = await querier.get({ payload: `request ${i}` })
  for await (const reply of replies.stream()) {
    if (!reply.result.error) console.log(reply.result.sample.payload.tryToString())
  }
}
// → echo request 0
// → echo request 1
// → echo request 2
```

## Pattern: a queryable store

Combining a subscriber and a queryable on the same key expression gives a **storage**
node: the subscriber ingests published updates into local state, and the queryable
serves that state to anyone who queries. This is how a Zenoh storage answers historical
queries for data published while a client was away.

```ts
import { Config, KeyExpr, Session } from '@diskette/dialtone'
import { setTimeout as sleep } from 'node:timers/promises'

await using session = await Session.open(Config.default())
const store = new Map<string, Uint8Array>()

await using subscriber = await session.declareSubscriber('demo/store/**')
await using queryable = await session.declareQueryable('demo/store/**', { complete: true })

// Ingest updates.
void (async () => {
  for await (const sample of subscriber.stream()) {
    const key = sample.keyExpr.toString()
    if (sample.kind === 'Delete') store.delete(key)
    else store.set(key, sample.payload.toBytes())
  }
})()

// Answer queries from current state.
void (async () => {
  for await (const query of queryable.stream()) {
    for (const [key, value] of store) {
      if (query.keyExpr.intersects(new KeyExpr(key))) await query.reply(key, value)
    }
    query.drop()
  }
})()

await session.put('demo/store/a', 'apple')
await session.put('demo/store/b', 'banana')
await sleep(100) // let the subscriber ingest

const replies = await session.get('demo/store/**')
for await (const reply of replies.stream()) {
  if (!reply.result.error) {
    console.log(`${reply.result.sample.keyExpr} = ${reply.result.sample.payload.tryToString()}`)
  }
}
// → demo/store/a = apple
// → demo/store/b = banana
```

## Option reference

| Goal | Where | Option |
| --- | --- | --- |
| Send data with the query | `get` / `querier.get` | `payload`, `encoding`, `attachment` |
| Filter / shape the response | `get` | `parameters` (or `?k=v` in the selector) |
| Bound how long to wait | `get` / `declareQuerier` | `timeout` (ms) |
| Control fan-out | `get` / `declareQuerier` | `target` (`BestMatching`/`All`/`AllComplete`) |
| Dedup replies | `get` / `declareQuerier` | `consolidation` (`Auto`/`None`/`Monotonic`/`Latest`) |
| Reply buffer strategy | `get` / `querier.get` | `channel` (`FifoChannel`/`RingChannel`) |
| Mark a queryable authoritative | `declareQueryable` | `complete: true` |
| Accept only local/remote queries | `declareQueryable` | `allowedOrigin` (`Locality`) |
| Reply with value / error / deletion | `Query` | `reply` / `replyErr` / `replyDel` |
| Finalize a query | `Query` | `drop()` |
| Know if a queryable is reachable | `Querier` | `matchingStatus()` / `matchingListener()` |

## See also

- [Quick Start](./quick-start.md) — sessions, publishing, and receiving.
- [Key Expressions](./key-expressions.md) — the key/selector language `get` and queryables match on.
- [Publish & Subscribe Guide](./pub-sub.md) — the push-based half, plus matching,
  liveliness, and channel/backpressure details shared with querying.
- [Configuration & Sessions](./configuration.md) — building configs, modes, endpoints,
  and inspecting a running session.
- [Serialization & Payloads](./serialization.md) — encoding query/reply payloads with
  `Bytes`, `Encoding`, and `zd`.
- [Lifecycle & Cleanup](./lifecycle.md) — finalizing queries with `drop()`, and releasing
  queryables and queriers.
</content>
