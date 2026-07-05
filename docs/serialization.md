# Serialization & Payloads

Zenoh moves **opaque bytes**. It doesn't know or care what your payload means — that's
your contract with the other end. Dialtone surfaces this directly:

- Anywhere you **send** data (`publisher.put`, `query.reply`, `session.put`, attachments,
  …) the argument is a `Payload` — **`string | Uint8Array`**.
- Anywhere you **receive** data (`sample.payload`, `query.payload`, `reply` errors, …)
  you get a **`Bytes`** object.
- **`Encoding`** is an advisory label describing what the bytes are — metadata, not
  enforcement.

At the end, `zd` adds a convenience layer for typed structured serialization.

Every snippet is a complete, runnable program — see the [Quick Start](./quick-start.md#prerequisites)
for setup and how to run them.

## Payloads in, `Bytes` out

Sending is as simple as passing a string or a `Uint8Array`:

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
await using subscriber = await session.declareSubscriber('demo/pay')
await using publisher = await session.declarePublisher('demo/pay')

// A string payload.
await publisher.put('a string')
console.log('string:', (await subscriber.recv()).payload.tryToString())
// → string: a string

// A binary payload.
await publisher.put(new Uint8Array([1, 2, 3, 250]))
const sample = await subscriber.recv()
console.log('bytes:', Array.from(sample.payload.toBytes()))
// → bytes: [ 1, 2, 3, 250 ]
```

## The `Bytes` class

`Bytes` wraps Zenoh's `ZBytes`. Construct one from a string or raw bytes, and read it back
as either:

```ts
import { Bytes } from '@diskette/dialtone'

const text = Bytes.fromString('hello 🌍')
console.log(text.len, text.isEmpty, text.tryToString())
// → 10 false hello 🌍   (len is the BYTE length — the emoji is 4 bytes)

const raw = Bytes.fromBytes(new Uint8Array([0, 255, 128]))
console.log(Array.from(raw.toBytes())) // → [ 0, 255, 128 ]

console.log(Bytes.new().isEmpty) // → true (an empty payload)
```

| Member | Meaning |
| --- | --- |
| `Bytes.fromString(s)` / `Bytes.fromBytes(u8)` / `Bytes.new()` | Construct. |
| `.len` | Length **in bytes**. |
| `.isEmpty` | Whether the payload is empty. |
| `.toBytes()` | The raw bytes as a `Uint8Array`. |
| `.tryToString()` | Decode as UTF-8 — **throws** on non-UTF-8. |

> **`tryToString()` can throw.** Zenoh payloads are arbitrary bytes; decoding non-UTF-8
> data fails with *"invalid utf-8 sequence…"*. For data that may not be text, use
> `toBytes()`. A safe-decode helper is handy in receive loops:
>
> ```ts
> function decode(bytes: import('@diskette/dialtone').Bytes): string {
>   try {
>     return bytes.tryToString()
>   } catch (err) {
>     return err instanceof Error ? err.message : String(err)
>   }
> }
> ```

## Encoding: describing your bytes

`Encoding` travels alongside a payload to tell the receiver how to interpret it. Zenoh
**does not enforce** it — a subscriber can send `text/plain` bytes with an
`application/json` encoding, and nothing stops it. It's a hint for your code and for
interop.

Set it on `put` (as a string) and read it off the received sample:

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
await using subscriber = await session.declareSubscriber('demo/enc')
await using publisher = await session.declarePublisher('demo/enc')

await publisher.put(JSON.stringify({ a: 1 }), { encoding: 'application/json' })
const sample = await subscriber.recv()
console.log('encoding:', sample.encoding.toString())
console.log('value:', JSON.parse(sample.payload.tryToString()))
// → encoding: application/json
// → value: { a: 1 }
```

The `Encoding` class provides constants for the common types and lets you build custom
ones:

```ts
import { Encoding } from '@diskette/dialtone'

console.log(Encoding.textPlain().toString())        // → text/plain
console.log(Encoding.applicationJson().toString())  // → application/json
console.log(Encoding.zenohSerialized().toString())  // → zenoh/serialized
console.log(Encoding.from('application/vnd.acme').toString()) // → application/vnd.acme

// withSchema appends a `;schema` suffix for a finer-grained hint.
console.log(Encoding.applicationJson().withSchema('user').toString())
// → application/json;user
```

There are ~50 built-in constructors (`textPlain`, `applicationJson`, `applicationCbor`,
`imagePng`, `videoH264`, …). `Encoding.from(str)` covers anything else.

## Structured data: bring your own format

Zenoh carries bytes, so *how* you turn a structure into bytes is your choice. The
simplest portable option is JSON with a matching encoding:

```ts
import { Config, Session } from '@diskette/dialtone'

type Reading = { id: number; celsius: number }

await using session = await Session.open(Config.default())
await using subscriber = await session.declareSubscriber('demo/json')
await using publisher = await session.declarePublisher('demo/json')

const reading: Reading = { id: 7, celsius: 21.5 }
await publisher.put(JSON.stringify(reading), { encoding: 'application/json' })

const sample = await subscriber.recv()
const decoded = JSON.parse(sample.payload.tryToString()) as Reading
console.log('decoded:', decoded) // → decoded: { id: 7, celsius: 21.5 }
```

The same shape works for Protobuf, CBOR, MessagePack, or any scheme you like — serialize
to a `Uint8Array`, send it, and set an encoding the other side understands.

## `zd`: typed serialization

`zd` is Dialtone's convenience layer over
[zenoh-ext](https://docs.rs/zenoh-ext)'s serializer. You declare a **schema** once, then
`serialize`/`deserialize` values against it — with TypeScript inferring the value type.
Its wire format is **byte-compatible with zenoh-ext**, so a `zd` payload interoperates
with Zenoh apps written in Rust and other languages.

> **The `zd` API is not yet stable.** It's a Dialtone convenience and may change in a
> future release. It's great for typed payloads between Dialtone/Zenoh services today,
> but for long-lived or externally-consumed wire formats, prefer an explicit format
> (JSON, Protobuf, CBOR) over raw `Bytes`, which is the stable foundation.

Build a schema from the `zd.*` builders and let `ZSchema<T>` carry the type:

```ts
import { zd } from '@diskette/dialtone'

const Reading = zd.object({
  id: zd.u32(),
  temp: zd.f64(),
  tags: zd.array(zd.string()),
})

const bytes = Reading.serialize({ id: 7, temp: 21.5, tags: ['a', 'b'] })
const value = Reading.deserialize(bytes)
console.log(value) // → { id: 7, temp: 21.5, tags: [ 'a', 'b' ] }  (typed as { id: number; temp: number; tags: string[] })
```

**Builders:**

- Leaves — `bool`, `string`, `bytes` (`Uint8Array`), `i8`/`i16`/`i32`, `u8`/`u16`/`u32`,
  `i64`/`u64` (**bigint**), `f32`/`f64`.
- Composites — `array(item)`, `set(item)` (JS `Set`), `map(key, value)` (JS `Map`),
  `tuple([...])` (fixed-arity, heterogeneous), `object({...})`.

A few semantics worth knowing — all enforced, none silently coerced:

- **`object` is positional on the wire.** Fields serialize in **declaration order**; that
  order is the cross-language contract (an object is a Rust tuple to peers). Field
  *names* are local — a Rust peer matches by position, not name. Extra keys on the input
  are ignored (width subtyping); a missing field throws.
- **64-bit integers are `bigint`** in and out — `zd.i64().serialize(5)` (a `number`)
  throws; pass `5n`.
- **Narrow integers are range-checked** — `zd.u8().serialize(300)` throws rather than
  wrapping.
- **Decoding validates length** — trailing bytes ("schema and payload disagree") or a
  truncated payload throw, so a mismatched schema fails loudly.

## `zd` end-to-end with pub/sub

Serialize on the way out, deserialize on the way in. Note the seam: `serialize` returns a
`Bytes`, but `put` wants a `string | Uint8Array`, so send `.toBytes()`; a received
`sample.payload` is already a `Bytes`, which `deserialize` takes directly. Tag the
payload with `zenoh/serialized` so consumers know the format.

```ts
import { Config, Session, zd, Encoding } from '@diskette/dialtone'

const Reading = zd.object({ id: zd.u32(), temp: zd.f64(), tags: zd.array(zd.string()) })

await using session = await Session.open(Config.default())
await using subscriber = await session.declareSubscriber('demo/zd')
await using publisher = await session.declarePublisher('demo/zd')

const payload = Reading.serialize({ id: 7, temp: 21.5, tags: ['a', 'b'] })
await publisher.put(payload.toBytes(), { encoding: Encoding.zenohSerialized().toString() })

const sample = await subscriber.recv()
console.log('decoded:', Reading.deserialize(sample.payload))
console.log('encoding:', sample.encoding.toString())
// → decoded: { id: 7, temp: 21.5, tags: [ 'a', 'b' ] }
// → encoding: zenoh/serialized
```

`zd` also exposes free functions if you prefer them to methods:
`zd.serialize(schema, value)` and `zd.deserialize(schema, bytes)`.

## See also

- [Quick Start](./quick-start.md) — sessions, publishing, and receiving.
- [Key Expressions](./key-expressions.md) — the topic language these payloads are addressed with.
- [Publish & Subscribe Guide](./pub-sub.md) — where these payloads flow.
- [Querying Guide](./querying.md) — query/reply payloads and attachments.
- [Configuration & Sessions](./configuration.md) — building configs and sessions.
- [Lifecycle & Cleanup](./lifecycle.md) — releasing the entities these payloads flow through.
