# Key Expressions

A **key expression** is how Zenoh names things. Every publisher, subscriber, queryable,
querier, query, and sample is addressed by one — it's the topic the whole system routes
on. Conceptually a key expression is a `/`-separated path like `demo/room1/temp`, but it
can also carry **wildcards**, so a single expression like `demo/**` names a *set* of keys
rather than one.

That set-of-keys idea is the entire point. A subscriber declares the set it wants to
hear, a publisher sends on a concrete key, and Zenoh delivers whenever the two sets
overlap. Matching (`intersects`) *is* the routing rule. Everything else in this guide —
constructing, validating, joining, testing for wildcards — serves that model.

This guide covers:

- The shape of a key: chunks, the rules, and the wildcards.
- Passing keys as a `string` or a `KeyExpr`.
- Constructing and validating a `KeyExpr`, and canonical form.
- Matching two keys with `intersects` and `includes`.
- Building keys with `join` and `concat`.
- Registering a hot key with `session.declareKeyexpr`.

Every snippet is a complete, runnable program — see the [Quick Start](./quick-start.md#prerequisites)
for setup and how to run them. A key expression is a plain local value, so most snippets
here don't even need a session.

## The shape of a key

A key expression is a `/`-separated list of non-empty UTF-8 **chunks**. A few rules make
a string a valid key:

- It may not start or end with `/`, nor contain `//` (no empty chunks).
- The characters `#`, `?` are forbidden, and `$` is only allowed as part of `$*`.
- It must be in **canonical form** (see below).

Wildcards are what turn a single expression into a set:

| Wildcard | Matches | Example |
| --- | --- | --- |
| `*` | exactly one chunk | `demo/*/temp` matches `demo/room1/temp` but not `demo/a/b/temp` |
| `**` | zero or more chunks | `demo/**` matches `demo`, `demo/room1`, `demo/room1/temp`, … |
| `$*` | zero or more characters within one chunk | `demo/room$*` matches `demo/room`, `demo/room1`, `demo/room2`, … |

A key with no wildcards names a single key; a key with wildcards names the set of every
concrete key it matches.

## `string` or `KeyExpr`

Every Dialtone API that takes a key accepts a `KeyExprArg` — which is simply
**`string | KeyExpr`**. Pass a string for convenience and it's parsed and validated on
the spot; pass a `KeyExpr` to reuse one you've already built and validated.

```ts
import { Config, Session, KeyExpr } from '@diskette/dialtone'

await using session = await Session.open(Config.default())

// A plain string — parsed and validated on the spot.
await using a = await session.declareSubscriber('demo/room1/temp')

// A KeyExpr instance — build it once, reuse it across calls.
const key = new KeyExpr('demo/room1/temp')
await using publisher = await session.declarePublisher(key)

await publisher.put('21C')
console.log((await a.recv()).keyExpr.toString()) // → demo/room1/temp
```

The two forms are interchangeable everywhere: `session.put`, `session.get`,
`declarePublisher`, `declareSubscriber`, `declareQueryable`, `declareQuerier`,
`query.reply`, `liveliness().declareToken`, and so on.

## Constructing and validating

`new KeyExpr(str)` parses a string and **throws** if it isn't a valid, canonical key.
`KeyExpr.fromStr(str)` is an alias for the constructor.

**Canonical form** is Zenoh's normalized spelling of a set: two strings that describe the
same set of keys always canonicalize to the same string, so set equality is just string
equality. Redundant forms like `**/**` aren't canonical and are rejected by the
constructor — use `KeyExpr.autocanonize(str)` to rewrite a string into canonical form
before building.

```ts
import { KeyExpr } from '@diskette/dialtone'

// Construct from a string.
const key = new KeyExpr('demo/room1/temp')
console.log(key.toString()) // → demo/room1/temp

// fromStr is the same as the constructor.
console.log(KeyExpr.fromStr('demo/**').toString()) // → demo/**

// A non-canonical form is rejected...
try {
  new KeyExpr('demo/**/**')
} catch (err) {
  console.log((err as Error).message.split(' at ')[0])
}
// → Invalid Key Expr `demo/**/**`: `**/**` must be replaced by `**` to reach canon-form

// ...but autocanonize rewrites it first.
console.log(KeyExpr.autocanonize('demo/**/**').toString()) // → demo/**
```

> **Validation is strict, and the message tells you why.** An empty chunk or a leading/
> trailing slash throws *"empty chunks are forbidden, as well as leading and trailing
> slashes"*; a stray `#`/`?` throws *"`#` and `?` are forbidden characters"*. Dialtone
> surfaces Zenoh's own error rather than trying to fix your key — reach for
> `autocanonize` when you want the fix.

## Matching: `intersects` and `includes`

Two key expressions relate in one of a few ways, and Dialtone exposes the two you need:

- **`a.intersects(b)`** — do the sets share at least one key? This is symmetric, and it's
  exactly the rule Zenoh uses to route: a publisher's key intersecting a subscriber's key
  expression is what triggers delivery.
- **`a.includes(b)`** — is *every* key in `b` also in `a`? This is directional: `a`
  covers `b`.

Both methods take a `KeyExpr`, not a string — construct one first. (The `declare*` methods
accept a `KeyExprArg`; these two don't.)

```ts
import { KeyExpr } from '@diskette/dialtone'

const sub = new KeyExpr('demo/**')          // a subscriber's set
const key = new KeyExpr('demo/room1/temp')  // a concrete published key

// intersects: the two sets share a key — so a message on `key` reaches `sub`.
console.log(sub.intersects(key)) // → true

// includes: demo/** covers the single key, but not the other way around.
console.log(sub.includes(key)) // → true
console.log(key.includes(sub)) // → false

// Two wildcard sets can overlap without either covering the other.
const a = new KeyExpr('demo/*/temp')
const b = new KeyExpr('demo/room1/*')
console.log(a.intersects(b)) // → true   (they meet on demo/room1/temp)
console.log(a.includes(b))   // → false
```

## Detecting wildcards: `isWild`

`isWild` is a getter that reports whether a key contains any wildcard character (`*`,
`**`, or `$*`) — i.e. whether it names a set rather than a single key.

```ts
import { KeyExpr } from '@diskette/dialtone'

console.log(new KeyExpr('demo/room1/temp').isWild) // → false
console.log(new KeyExpr('demo/**').isWild)         // → true
console.log(new KeyExpr('demo/*/temp').isWild)     // → true
```

## Building keys: `join` and `concat`

Two ways to extend a key, and they are not the same:

- **`join(str)`** inserts a `/` between the two sides and canonizes the result. This is
  the right tool for adding path segments.
- **`concat(str)`** is raw string concatenation with **no** separator — use it to finish
  a chunk that was split across values.

```ts
import { KeyExpr } from '@diskette/dialtone'

const base = new KeyExpr('demo/room1')

// join inserts a `/`.
console.log(base.join('temp').toString()) // → demo/room1/temp

// join canonizes, so redundant `**` collapse.
console.log(new KeyExpr('demo/**').join('**/sensors').toString())
// → demo/**/sensors

// concat is raw append — here it completes the `room1` chunk.
console.log(new KeyExpr('demo/room').concat('1/temp').toString())
// → demo/room1/temp
```

> **`concat` guards against a `*` collision.** If the left side ends with `*` and the
> right starts with `*`, `concat` throws (*"Tried to concatenate … which would likely
> have caused bugs…"*) rather than silently producing something like `***`. Prefer
> `join` whenever you're gluing path segments — it inserts the `/` and canonizes for you.

## Registering a hot key: `session.declareKeyexpr`

When you use the same key over and over, `session.declareKeyexpr(key)` registers it with
the network once and hands back a `KeyExpr`. Zenoh can then refer to that key by a short
numeric id on the wire instead of resending the full string on every operation — an
optimization for high-rate paths. Use the returned key exactly like any other.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())

// Register the key once; reuse the returned KeyExpr on every put.
const key = await session.declareKeyexpr('demo/room1/temp')
await using subscriber = await session.declareSubscriber(key)

for (let i = 0; i < 3; i++) await session.put(key, `${20 + i}C`)

console.log((await subscriber.recv()).payload.tryToString()) // → 20C
```

## Where key expressions surface

Key expressions flow back out of nearly every entity, always as a `KeyExpr`:
`sample.keyExpr`, `publisher.keyExpr`, `subscriber.keyExpr`, `query.keyExpr`,
`queryable.keyExpr`. A received sample's key is the **concrete** key it was published on,
even when the subscriber matched with a wildcard — which is how you learn *which* key in
your set actually arrived.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
await using subscriber = await session.declareSubscriber('demo/**')
await using publisher = await session.declarePublisher('demo/room1/temp')

await publisher.put('21C')

const sample = await subscriber.recv()
// Subscribed on a wildcard set, but the sample carries the concrete key.
console.log(sample.keyExpr.toString()) // → demo/room1/temp
```

A **`Selector`** pairs a key expression with query parameters (`demo/sensors/**?unit=C`)
and is what `session.get` addresses — see the [Querying Guide](./querying.md) for that.

## Reference

| Task | API |
| --- | --- |
| Construct (validate) | `new KeyExpr(str)` / `KeyExpr.fromStr(str)` |
| Construct, fixing the form | `KeyExpr.autocanonize(str)` |
| Back to a string | `keyExpr.toString()` |
| Do two sets overlap? | `a.intersects(b)` (symmetric — the routing rule) |
| Does one set cover another? | `a.includes(b)` (directional) |
| Any wildcard present? | `keyExpr.isWild` |
| Add a path segment | `keyExpr.join(str)` — inserts `/`, canonizes |
| Raw string append | `keyExpr.concat(str)` |
| Accept a string or a `KeyExpr` | `KeyExprArg` = `string \| KeyExpr` |
| Optimize a repeated key | `session.declareKeyexpr(key)` |

## See also

- [Quick Start](./quick-start.md) — sessions, publishing, and receiving.
- [Publish & Subscribe Guide](./pub-sub.md) — where publishers and subscribers match on
  these keys.
- [Querying Guide](./querying.md) — selectors, parameters, and query/reply matching.
- [Configuration & Sessions](./configuration.md) — opening the session these keys are
  declared on.
- [Lifecycle & Cleanup](./lifecycle.md) — releasing the entities addressed by these keys.
