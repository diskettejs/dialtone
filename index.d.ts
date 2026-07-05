export * from './binding.js'

import * as binding from './binding.js'

declare module './binding.js' {
  interface Session {
    [Symbol.asyncDispose](): Promise<void>
  }
  interface Subscriber {
    [Symbol.dispose](): void
  }
  interface Publisher {
    [Symbol.dispose](): void
  }
  interface MatchingListener {
    [Symbol.dispose](): void
  }
  interface SampleMissListener {
    [Symbol.dispose](): void
  }
  interface Scout {
    [Symbol.dispose](): void
  }
  interface LivelinessToken {
    [Symbol.dispose](): void
  }
  interface LivelinessSubscriber {
    [Symbol.dispose](): void
  }
  interface Querier {
    [Symbol.dispose](): void
  }
  interface Queryable {
    [Symbol.dispose](): void
  }
  interface LinkEventsListener {
    [Symbol.dispose](): void
  }
  interface TransportEventsListener {
    [Symbol.dispose](): void
  }
}

/**
 * A native {@link binding.zd.Schema} that carries its decoded type `T` at
 * compile time. `T` is erased at runtime — every `zd.*()` builder returns the
 * same monomorphic native `Schema`; this interface only narrows
 * `serialize`/`deserialize` for inference.
 */
export interface ZSchema<T> extends binding.zd.Schema {
  serialize(data: T): binding.Bytes
  deserialize(bytes: binding.Bytes): T
}

/**
 * Schema builders and the free `serialize`/`deserialize` functions. A generic
 * facade over the native `zd` namespace, which napi-rs generates
 * non-generically; the runtime object is re-exported unchanged from
 * `./binding.js`.
 *
 * @example
 * const point = zd.object({ x: zd.f64(), y: zd.f64() })
 * const bytes = point.serialize({ x: 1.5, y: -2 })
 * const value = point.deserialize(bytes) // { x: number, y: number }
 */
export declare const zd: {
  bool(): ZSchema<boolean>
  string(): ZSchema<string>
  bytes(): ZSchema<Uint8Array>
  i8(): ZSchema<number>
  i16(): ZSchema<number>
  i32(): ZSchema<number>
  u8(): ZSchema<number>
  u16(): ZSchema<number>
  u32(): ZSchema<number>
  i64(): ZSchema<bigint>
  u64(): ZSchema<bigint>
  f32(): ZSchema<number>
  f64(): ZSchema<number>
  array<T>(item: ZSchema<T>): ZSchema<T[]>
  set<T>(item: ZSchema<T>): ZSchema<Set<T>>
  map<K, V>(key: ZSchema<K>, value: ZSchema<V>): ZSchema<Map<K, V>>
  tuple<T extends unknown[]>(items: { [K in keyof T]: ZSchema<T[K]> }): ZSchema<T>
  object<T extends Record<string, unknown>>(fields: {
    [K in keyof T]: ZSchema<T[K]>
  }): ZSchema<T>
  serialize<T>(schema: ZSchema<T>, data: T): binding.Bytes
  deserialize<T>(schema: ZSchema<T>, bytes: binding.Bytes): T
}

/** Node operating mode. Maps to Zenoh's `mode` key. */
export type WhatAmI = 'router' | 'peer' | 'client'

/**
 * A value that is either applied uniformly, or per-mode.
 *
 * e.g. `["tcp/10.0.0.1:7447"]` (unique) or
 * `{ router: ["tcp/10.0.0.1:7447"], peer: [] }` (mode-dependent).
 */
export type ModeDependent<T> = T | { router?: T; peer?: T; client?: T }

/**
 * A locator string, e.g. `"tcp/127.0.0.1:7447"`. Mirrors Zenoh's `EndPoint`.
 */
export type EndPoint = string

export interface ConnectConfig {
  timeout_ms?: ModeDependent<number>
  endpoints?: ModeDependent<EndPoint[]>
  exit_on_failure?: ModeDependent<boolean>
  retry?: Record<string, unknown>
}

export interface ListenConfig {
  timeout_ms?: ModeDependent<number>
  endpoints?: ModeDependent<EndPoint[]>
  exit_on_failure?: ModeDependent<boolean>
  retry?: Record<string, unknown>
}

export interface OpenConfig {
  return_conditions?: {
    connect_scouted?: boolean
    declares?: boolean
  }
}

export interface ScoutingMulticastConfig {
  enabled?: boolean
  address?: string
  interface?: string
  ttl?: number
  autoconnect?: ModeDependent<WhatAmI[]>
  autoconnect_strategy?: unknown
  listen?: ModeDependent<boolean>
}

export interface ScoutingGossipConfig {
  enabled?: boolean
  multihop?: boolean
  target?: ModeDependent<WhatAmI[]>
  autoconnect?: ModeDependent<WhatAmI[]>
  autoconnect_strategy?: unknown
}

export interface ScoutingConfig {
  timeout?: number
  delay?: number
  multicast?: ScoutingMulticastConfig
  gossip?: ScoutingGossipConfig
}

export interface TimestampingConfig {
  enabled?: ModeDependent<boolean>
  drop_future_timestamp?: boolean
}

export interface AggregationConfig {
  subscribers?: string[]
  publishers?: string[]
}

/**
 * A typed view of Zenoh's session configuration. Keys mirror Zenoh's
 * configuration (`snake_case`); every field is optional and falls back to
 * Zenoh's defaults. Frequently-used sections are typed precisely; deep or
 * rarely-tuned sections are left open and validated by Zenoh at parse time.
 */
export interface ZenohConfig {
  id?: string
  metadata?: Record<string, unknown>
  mode?: WhatAmI
  region_name?: string
  namespace?: string
  queries_default_timeout?: number
  connect?: ConnectConfig
  listen?: ListenConfig
  open?: OpenConfig
  scouting?: ScoutingConfig
  timestamping?: TimestampingConfig
  aggregation?: AggregationConfig
  routing?: Record<string, unknown>
  qos?: Record<string, unknown>
  transport?: Record<string, unknown>
  adminspace?: Record<string, unknown>
  access_control?: Record<string, unknown>
  low_pass_filter?: Array<Record<string, unknown>>
  downsampling?: Array<Record<string, unknown>>
  stats?: Record<string, unknown>
  gateway?: Record<string, unknown>
  plugins_loading?: Record<string, unknown>
  plugins?: Record<string, unknown>
}

/**
 * Builds a {@link binding.Config} from a typed config object, ready to pass to
 * `Session.open`. Equivalent to `Config.fromJson5(JSON.stringify(config))`;
 * Zenoh validates the result and throws on invalid or unknown keys.
 *
 * @example
 * await Session.open(defineConfig({ mode: 'router' }))
 */
export declare function defineConfig(config?: ZenohConfig): binding.Config
