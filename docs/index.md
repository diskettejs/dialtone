---
layout: home

hero:
  name: Dialtone
  text: Zenoh for Node.js
  tagline: Native bindings to Zenoh — pub/sub, query, and liveliness, surfaced as async iterators and channels.
  actions:
    - theme: brand
      text: Quick Start
      link: /quick-start
    - theme: alt
      text: API Reference
      link: /api/
    - theme: alt
      text: GitHub
      link: https://github.com/diskettejs/dialtone

features:
  - title: Thin proxy over Zenoh
    details: Sessions, publishers, subscribers, queryables, and config mirror Zenoh's own surface — no invented safety, no hidden guarantees.
  - title: Channels & async iterators
    details: Every receiver exposes recv() / tryRecv() and a stream() you can `for await`. No callbacks.
  - title: Explicit lifecycles
    details: Entities are live Zenoh resources released with `using` / `await using`. Ownership is yours, not the GC's.
  - title: Typed payloads with zd
    details: Declare a schema once and serialize/deserialize with full type inference — wire-compatible with zenoh-ext.
---

## Install

```sh
pnpm add @diskette/dialtone
```

Requires **Node.js ≥ 24** (the docs use `await using`). Prebuilt binaries ship for macOS, Linux, and Windows.

## Hello, Zenoh

```ts twoslash
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
using publisher = await session.declarePublisher('demo/hello')
using subscriber = await session.declareSubscriber('demo/hello')

await publisher.put('Hello, Zenoh!')

const sample = await subscriber.recv()
console.log(`${sample.keyExpr} → ${sample.payload.tryToString()}`)
// → demo/hello → Hello, Zenoh!
```

Start at the [Quick Start](./quick-start), then dig into [Key Expressions](./key-expressions),
[Pub/Sub](./pub-sub), and [Querying](./querying). The [API Reference](./api/) is generated
from the shipped type declarations.
