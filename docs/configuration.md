# Configuration & Sessions

Every session opens against a **`Config`**. Zenoh's configuration is a large JSON5
document with defaults for everything; Dialtone exposes it two ways:

- The **`Config`** class — a thin proxy over Zenoh's own config: load it from a file, a
  JSON5 string, or the environment, then read and edit individual keys.
- **`defineConfig()`** — a typed convenience that builds a `Config` from a plain object
  with editor autocompletion (covered at the end).

A **`Session`** is your handle to the Zenoh network: open it against a config, use it to
declare publishers/subscribers/queryables, and close it when done.

Every snippet is a complete, runnable program — see the [Quick Start](./quick-start.md#prerequisites)
for setup and how to run them.

## Opening and closing a session

`Session.open` takes a `Config` instance — not a plain object — so you always go through
`Config` or `defineConfig`. `Config.default()` is the zero-config starting point.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.default())
console.log('zid:', session.zid)
console.log('closed?', session.isClosed) // false
```

`await using` closes the session when the scope ends (it implements `Symbol.asyncDispose`).
To manage it manually, call `await session.close()` and check `session.isClosed`.

## Building a Config

There are four ways to get a `Config`:

```ts
import { Config } from '@diskette/dialtone'

const fromDefault = Config.default()                     // Zenoh's defaults
const fromString = Config.fromJson5('{ mode: "peer" }')  // a JSON5 string
// const fromDisk = Config.fromFile('./zenoh.json5')     // a JSON5/JSON file
// const fromEnv  = Config.fromEnv()                      // file at $ZENOH_CONFIG

console.log(fromString.getJson('mode')) // "peer"
```

`Config.fromEnv()` reads the path held in the environment variable named by
`Config.defaultConfigPathEnv()` (`ZENOH_CONFIG`). JSON5 means comments and unquoted keys
are allowed, so a config file stays readable.

## Reading and editing keys

Edit any key with `insertJson5(key, value)` and read it back with `getJson(key)`. Two
things to know:

- The **value is JSON5**, so string values must be quoted — `JSON.stringify('client')`
  produces `'"client"'`. An unquoted `'client'` is not valid JSON5 and is rejected.
- **Nested keys use `/`** as the path separator, e.g. `scouting/multicast/enabled`.
- `getJson` returns a **JSON string** — `JSON.parse` it to get the value.

```ts
import { Config } from '@diskette/dialtone'

const config = Config.default()

config.insertJson5('mode', JSON.stringify('client'))     // quote the string value
config.insertJson5('scouting/multicast/enabled', 'false') // nested key, JSON5 literal

console.log(config.getJson('mode'))                       // "client"
console.log(JSON.parse(config.getJson('mode')))           // client
console.log(config.getJson('scouting/multicast/enabled')) // false
```

> **`remove` is narrow.** `config.remove(key)` only works for keys under `plugins/`;
> anything else throws *"Removal of values from Config is only supported for keys
> starting with `plugins/`."* To "unset" a normal key, rebuild the config or overwrite
> the key instead.

## Node modes

The `mode` key sets how a node participates in the network:

| Mode | Role |
| --- | --- |
| `peer` (default) | Connects directly to other peers, forming a mesh. |
| `client` | Connects to a single router or peer as its access point. |
| `router` | Routes traffic between other nodes — the infrastructure role (`zenohd`). |

```ts
import { Config } from '@diskette/dialtone'

const config = Config.default()
config.insertJson5('mode', JSON.stringify('client'))
```

## Connectivity: connect & listen endpoints

Endpoints are locator strings like `tcp/127.0.0.1:7447` (also `udp`, `quic`, `tls`, …).
`listen/endpoints` are the addresses a node accepts inbound connections on;
`connect/endpoints` are addresses it dials out to (e.g. a known router). Port `0` picks a
free port.

```ts
import { Config, Session } from '@diskette/dialtone'

const config = Config.fromJson5(`{
  mode: "peer",
  listen: { endpoints: ["tcp/127.0.0.1:0"] },
}`)

await using session = await Session.open(config)
console.log('listening on:', session.config().get('listen/endpoints'))
// → listening on: ["tcp/127.0.0.1:0"]
```

Both sections accept per-mode values too — `{ router: [...], peer: [...] }` — so one
config can behave differently depending on the mode it runs as.

## Discovery: scouting

Peers find each other by **scouting**: UDP multicast on the local network, and gossip
through already-connected nodes. Disable multicast when you want nodes to connect only
through explicit endpoints:

```ts
import { Config } from '@diskette/dialtone'

const config = Config.fromJson5(`{
  scouting: {
    multicast: { enabled: false },
    gossip: { enabled: true },
  },
}`)

console.log(config.getJson('scouting/multicast/enabled')) // false
```

## Timestamping

Timestamping stamps every published sample with an HLC timestamp. It's off by default
and must be enabled for the advanced pub/sub features that rely on sequence numbers — the
publisher **cache** and **sample-miss detection** (see the
[Pub/Sub Guide](./pub-sub.md)):

```ts
import { Config } from '@diskette/dialtone'

const config = Config.fromJson5('{ timestamping: { enabled: true } }')
console.log(config.getJson('timestamping/enabled')) // true
```

Without it, declaring a publisher with `cache` throws *"the 'timestamping' setting must
be enabled in the Zenoh configuration."*

## Inspecting a running session

Two accessors report on the live session:

- **`session.config()`** → a `SessionConfig` view: `get(key)` (JSON string),
  `toJson()` (the whole effective config), `queriesDefaultTimeoutMs()` (a `bigint`), and
  `getPluginConfig(name)`.
- **`session.info()`** → a `SessionInfo` handle whose methods are **async**: `zid()`,
  `routersZid()`, `peersZid()`, `transports()`, and `links()`.

```ts
import { Config, Session } from '@diskette/dialtone'

await using session = await Session.open(Config.fromJson5('{ mode: "peer" }'))

const cfg = session.config()
console.log('mode:', cfg.get('mode'))                        // "peer"
console.log('query timeout ms:', cfg.queriesDefaultTimeoutMs()) // 10000n

const info = session.info()
console.log('zid:', await info.zid())        // this node's id (hex)
console.log('routers:', await info.routersZid())
console.log('peers:', await info.peersZid())
```

## defineConfig(): typed configuration

Writing raw JSON5 keys is precise but unguided. `defineConfig()` gives you a
type-checked, autocompleted config object and returns a `Config` — it is exactly
`Config.fromJson5(JSON.stringify(config))`:

```ts
import { Session, defineConfig } from '@diskette/dialtone'

const config = defineConfig({
  mode: 'peer',
  scouting: { multicast: { enabled: false } },
  timestamping: { enabled: true },
})

await using session = await Session.open(config)
console.log('mode:', session.config().get('mode')) // "peer"
```

> **The types are a guide, not a guarantee.** Zenoh does not expose Rust-side types for
> its configuration schema, so Dialtone maintains `ZenohConfig` by hand. It can drift
> from what your installed Zenoh version actually accepts. Frequently-used sections
> (`mode`, `connect`, `listen`, `scouting`, `timestamping`) are typed precisely; deeper
> or rarely-tuned sections are intentionally left open (`Record<string, unknown>`).
> Always verify your config against the version you run — Zenoh validates at
> `Session.open` and throws on unknown or invalid keys, so if `open` succeeds your keys
> were accepted. When in doubt, drop to `Config.fromJson5` / `Config.fromFile` with raw
> JSON5; the underlying `Config` is the source of truth.

## Key reference

| Task | API |
| --- | --- |
| Start from defaults | `Config.default()` |
| Load from a file / string / env | `Config.fromFile` / `Config.fromJson5` / `Config.fromEnv` |
| Read a key | `config.getJson(key)` → JSON string |
| Set a key | `config.insertJson5(key, json5Value)` |
| Remove a plugin key | `config.remove('plugins/...')` |
| Typed config object | `defineConfig(zenohConfig)` |
| Open / close a session | `Session.open(config)` / `session.close()` |
| Live config view | `session.config()` → `get`, `toJson`, `queriesDefaultTimeoutMs` |
| Network info | `session.info()` → `zid`, `routersZid`, `peersZid`, `transports`, `links` |

## See also

- [Quick Start](./quick-start.md) — sessions, publishing, and receiving.
- [Key Expressions](./key-expressions.md) — the topic language for declaring entities.
- [Publish & Subscribe Guide](./pub-sub.md) — where `timestamping` and locality matter.
- [Querying Guide](./querying.md) — `get`, queryables, and queriers.
- [Serialization & Payloads](./serialization.md) — `Bytes`, `Encoding`, and the `zd` schema system.
- [Lifecycle & Cleanup](./lifecycle.md) — opening, closing, and owning sessions and entities.
