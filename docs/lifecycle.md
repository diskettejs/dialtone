# Lifecycle & Cleanup

Most Node objects are pure memory — you drop the last reference and the garbage
collector reclaims them on its own schedule, and you never think about it. Dialtone
entities are different. Each one is a handle to a **live Zenoh resource**: a network
declaration, a background task, an open transport. Those resources stay active until you
release them, so *when* they go away is something you control, not something you leave to
the GC.

You release an entity one of two ways:

- Tie it to a scope with [`using`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/using)
  / [`await using`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/await_using),
  and it's released when the scope ends.
- Call its release method (`undeclare()`, `close()`, or `stop()`) yourself.

> **Don't lean on the garbage collector.** An entity you simply stop referencing keeps
> its declaration alive — a subscriber goes on buffering samples, a publisher stays
> advertised, a session holds its transport open — until the collector eventually
> reclaims the wrapper. GC timing is non-deterministic, so "let it fall out of scope" is
> not a cleanup strategy. Release entities explicitly.

Every snippet is a complete, runnable program — see the [Quick Start](./quick-start.md#prerequisites)
for setup and how to run them.

## Two kinds of disposal

The release method depends on the entity, and so does the keyword:

- A **`Session`** releases **asynchronously** — `close()` returns a promise — so it needs
  `await using`.
- **Everything else** releases **synchronously**, so plain `using` is enough. Most
  entities release with `undeclare()`; a `Scout` uses `stop()`.

| Entity | Release method | `using` form |
| --- | --- | --- |
| `Session` | `close()` — async | `await using` |
| `Publisher`, `Subscriber` | `undeclare()` | `using` |
| `Queryable`, `Querier` | `undeclare()` | `using` |
| `MatchingListener`, `SampleMissListener` | `undeclare()` | `using` |
| `LivelinessToken`, `LivelinessSubscriber` | `undeclare()` | `using` |
| `Scout` | `stop()` | `using` |
| `Query` | `drop()` — **no `using`** | manual only (see below) |

## Scope-bound cleanup (the default)

`using` / `await using` is the least error-prone option: the entity is released the moment
control leaves the block, whether that's a normal exit, a `return`, or a thrown error.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
using publisher = await session.declarePublisher('demo/scope')
using subscriber = await session.declareSubscriber('demo/scope')

await publisher.put('scoped')
console.log((await subscriber.recv()).payload.tryToString()) // → scoped

// At the end of this scope: subscriber and publisher undeclare, then the session closes.
```

**Declaration order matters.** Disposals run in the reverse order of declaration, so
declaring the `Session` *first* means it closes *last* — after the publisher and
subscriber it owns have undeclared. Declare a session before the entities that depend on
it, and the teardown order takes care of itself.

## Releasing manually

When an entity outlives the block it's created in — you store it on an object, hand it to
another function, or release it only under some condition — bind it with `const` and call
the release method yourself.

```ts
import { Config, Session } from '@diskette/dialtone'

const session = await Session.open(Config.default())
const publisher = await session.declarePublisher('demo/manual')

await publisher.put('done')

publisher.undeclare()   // release the publisher
await session.close()   // then the session
console.log(session.isClosed) // → true
```

Once released, an entity is spent — reusing it rejects with Zenoh's own error, rather
than silently no-op'ing:

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
const publisher = await session.declarePublisher('demo/spent')

publisher.undeclare()
await publisher.put('too late').catch((err) => console.log(err.message))
// → Undeclared publisher
```

> **Pick one strategy per entity — `using` or manual, not both.** Releasing is not
> idempotent: a second `undeclare()` throws *"Undeclared publisher"*. Because `using`
> calls `undeclare()` for you at scope exit, a manual release *inside* that scope makes
> the automatic one fail. Bind with `using` **or** release by hand; never both on the
> same entity.

## Don't `using` something you return

`using` and `await using` dispose at the end of the **enclosing block** — and a function
body is a block. So a factory that binds an entity with `using` and then returns it hands
back an *already-released* entity: the disposal ran on the way out.

```ts
import { Config, Session } from '@diskette/dialtone'

// ⚠️ Wrong: `await using` closes the session when `connect` returns.
async function connect() {
  await using session = await Session.open(Config.default())
  return session
}

const session = await connect()
console.log(session.isClosed) // → true — already closed
await session.put('demo/x', 'hi').catch((err) => console.log(err.message))
// → session closed
```

The fix: a factory doesn't *own* what it returns — the caller does. Bind with `const` and
hand back a live entity; let the **caller** decide its lifetime.

```ts
import { Config, Session } from '@diskette/dialtone'

// ✅ Right: return a live session; ownership transfers to the caller.
async function connect() {
  const session = await Session.open(Config.default())
  return session
}

await using session = await connect() // the caller's scope now owns it
await session.put('demo/x', 'hi')
console.log(session.isClosed)          // → false
```

The rule generalizes: only `using` an entity in the scope that should *end* its life.
Anything you return, store on an object, or pass onward outlives the current block, so
release it manually or make its new owner responsible.

## Owning entities on an object

For a long-lived component that holds Zenoh entities, make the **component** the unit of
cleanup: open what it needs, and implement `Symbol.asyncDispose` (or a `close()`) that
releases everything in one place. Now the component is itself `await using`-able, and its
scope drives the teardown of everything inside it.

```ts
import { Config, Session, type Publisher } from '@diskette/dialtone'

class Telemetry {
  #session: Session
  #publisher: Publisher

  private constructor(session: Session, publisher: Publisher) {
    this.#session = session
    this.#publisher = publisher
  }

  static async start() {
    const session = await Session.open(Config.default())
    const publisher = await session.declarePublisher('demo/telemetry')
    return new Telemetry(session, publisher)
  }

  report(value: number) {
    return this.#publisher.put(`reading:${value}`)
  }

  async [Symbol.asyncDispose]() {
    this.#publisher.undeclare() // release children first,
    await this.#session.close() // then the session
  }
}

await using telemetry = await Telemetry.start()
await telemetry.report(21)
// Leaving this scope disposes `telemetry`, which undeclares the publisher and closes the session.
```

Note the constructor is `private` and the entities are opened in an async `start()`
factory — `Session.open` and `declarePublisher` are async, so they can't run in a
constructor. This is the deliberate, higher-level pattern; the raw entities keep Zenoh's
own surface.

## Finalizing a query

A `Query` is the one lifecycle object that isn't a declared entity and has no `using`
form. A queryable holds it open while responding, and the client's `get` stays open until
every queryable it reached finalizes. Release it with `query.drop()` once you've replied:

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
await using queryable = await session.declareQueryable('demo/q/**')

void (async () => {
  for await (const query of queryable.stream()) {
    await query.reply(query.keyExpr, 'answer')
    query.drop() // finalize — don't wait for GC or the query timeout
  }
})()

const replies = await session.get('demo/q/value')
for await (const reply of replies.stream()) {
  if (!reply.result.error) console.log(reply.result.sample.payload.tryToString()) // → answer
}
```

See the [Querying Guide](./querying.md) for the full request/response model.

## Reference

| Task | API |
| --- | --- |
| Scope-bound release (async) | `await using x = ...` → `close()` |
| Scope-bound release (sync) | `using x = ...` → `undeclare()` / `stop()` |
| Release a session manually | `await session.close()`; check `session.isClosed` |
| Release an entity manually | `entity.undeclare()` (or `scout.stop()`) |
| Finalize a query | `query.drop()` |
| Own entities in a component | implement `Symbol.asyncDispose` / a `close()` method |

## See also

- [Quick Start](./quick-start.md) — the first look at `using` / `await using`.
- [Key Expressions](./key-expressions.md) — the topic language the entities are declared on.
- [Publish & Subscribe Guide](./pub-sub.md) — publishers, subscribers, and listeners.
- [Querying Guide](./querying.md) — queryables, queriers, and `query.drop()`.
- [Configuration & Sessions](./configuration.md) — opening and closing sessions.
