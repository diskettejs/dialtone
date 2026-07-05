# Quick Start

Dialtone (`@diskette/dialtone`) is Node.js native bindings for [Zenoh](https://zenoh.io).
This guide gets a publisher and subscriber talking in a few minutes.

## Prerequisites

- **Node.js ≥ 24** — the snippets use `await using` (explicit resource management).
- Install the package:

  ```sh
  pnpm add @diskette/dialtone
  ```

Every snippet in this guide — and throughout the docs — is a complete, self-contained
program. Save one as `hello.ts` and run it with `node hello.ts`. The code is plain
ESM — TypeScript works too, and type definitions ship with the package.

## Hello, Zenoh

A publisher and a subscriber on the same session exchange a message locally — no
router, no network setup. This is the fastest way to confirm your install works.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
using publisher = await session.declarePublisher('demo/hello')
using subscriber = await session.declareSubscriber('demo/hello')

await publisher.put('Hello, Zenoh!')

const sample = await subscriber.recv()
console.log(`${sample.keyExpr} → ${sample.payload.tryToString()}`)
// → demo/hello → Hello, Zenoh!
```

[`using`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/using)/[`await using`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/await_using) tie each entity's lifetime to the enclosing scope. The publisher and
subscriber undeclare synchronously, so plain `using` is enough; the session's `close()` is
async, so it needs `await using` to be awaited when the program ends.

## Opening a session

Everything starts with a `Session`. `Config.default()` builds Zenoh's default
configuration; see the [Configuration Guide](./configuration.md) for loading a config
from JSON5, a file, or the environment.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
console.log('Session zid:', session.zid)
```

## Reading a sample

Each message arrives as a `Sample`. The payload is raw bytes — decode it with
`tryToString()` (UTF-8) or `toBytes()` (a `Uint8Array`). Other fields describe the
message: `keyExpr`, `kind` (`'Put'` or `'Delete'`), `encoding`, and an optional
`attachment` and `timestamp`.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
using publisher = await session.declarePublisher('demo/sample')
using subscriber = await session.declareSubscriber('demo/sample')

await publisher.put('inspect me', { encoding: 'text/plain' })

const sample = await subscriber.recv()
console.log('key:     ', sample.keyExpr.toString())
console.log('kind:    ', sample.kind)
console.log('encoding:', sample.encoding.toString())
console.log('payload: ', sample.payload.tryToString())
console.log('bytes:   ', sample.payload.toBytes())
```

> `tryToString()` throws if the payload is not valid UTF-8 — Zenoh payloads are
> arbitrary bytes. Reach for `toBytes()` when the data may not be text.

## Two ways to receive

A subscriber exposes the same messages two ways. Pull one at a time with `recv()`
(async) or `tryRecv()` (non-blocking, returns `null` when empty), or iterate with
`stream()`. Both draw from the same underlying channel.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
using publisher = await session.declarePublisher('demo/recv')
using subscriber = await session.declareSubscriber('demo/recv')

const total = 3
for (let i = 0; i < total; i++) await publisher.put(`msg ${i}`)

// recv(): pull the next message
console.log('recv():  ', (await subscriber.recv()).payload.tryToString())

// stream(): iterate the rest
let seen = 1
for await (const sample of subscriber.stream()) {
  console.log('stream():', sample.payload.tryToString())
  if (++seen === total) break
}
// → recv():   msg 0
// → stream(): msg 1
// → stream(): msg 2
```

## Publishing with options, and deleting

`put` takes optional `encoding` and an `attachment` (side-band bytes). `delete`
publishes a tombstone for the key — the subscriber receives it as a `Delete`-kind
sample.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
using publisher = await session.declarePublisher('demo/options')
using subscriber = await session.declareSubscriber('demo/options')

await publisher.put('with metadata', { encoding: 'text/plain', attachment: 'v1' })
const put = await subscriber.recv()
console.log(put.kind, '|', put.payload.tryToString(), '|', put.attachment?.tryToString())
// → Put | with metadata | v1

await publisher.delete({ attachment: 'removed' })
const del = await subscriber.recv()
console.log(del.kind, '|', del.attachment?.tryToString())
// → Delete | removed
```

## Across two processes

In practice the publisher and subscriber live in separate processes and discover each
other automatically. Save these as two files and run each in its own terminal — the
subscriber's `demo/example/**` key expression matches everything the publisher sends.

**`sub.ts`**

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
using subscriber = await session.declareSubscriber('demo/example/**')

console.log('Listening on demo/example/** — Ctrl-C to stop')
for await (const sample of subscriber.stream()) {
  console.log(`${sample.keyExpr} → ${sample.payload.tryToString()}`)
}
```

**`pub.ts`**

```ts
import { setTimeout as sleep } from 'node:timers/promises'
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
using publisher = await session.declarePublisher('demo/example/hello')

console.log('Publishing to demo/example/hello — Ctrl-C to stop')
for (let i = 0; ; i++) {
  await publisher.put(`Hello #${i}`)
  await sleep(1000)
}
```

Start `sub.ts` first, then `pub.ts`, and watch the messages arrive.

## Lifecycle and cleanup

`using`/`await using` is the recommended way to manage entities, but you can also declare
and release them manually. Undeclaring a publisher or subscriber (or closing the session)
is explicit:

```ts
import { Config, Session } from '@diskette/dialtone'

const session = await Session.open(Config.default())
const publisher = await session.declarePublisher('demo/manual')

await publisher.put('done')

publisher.undeclare()
await session.close()
```

Once an entity is undeclared, using it again rejects — Dialtone surfaces Zenoh's
lifecycle rather than silently ignoring the call:

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
const publisher = await session.declarePublisher('demo/guard')

publisher.undeclare()
await publisher.put('too late').catch((err) => console.log(err.message))
// → Undeclared publisher
```

The [Lifecycle & Cleanup](./lifecycle.md) guide covers manual release, ownership across
function boundaries, and disposal ordering in depth.

## Next steps

- Tune delivery with declare-time `PublisherOptions` (`priority`, `congestionControl`,
  `express`, `reliability`) and `SubscriberOptions`.
- Control backpressure by passing a `FifoChannel` or `RingChannel` as the `channel`
  option when declaring a subscriber.
- Observe matching subscribers with `publisher.matchingStatus()` /
  `publisher.matchingListener()`.
