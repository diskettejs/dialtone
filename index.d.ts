export * from './binding.js'

import * as binding from './binding.js'

declare module './binding.js' {
  interface Session {
    [Symbol.asyncDispose](): Promise<void>
    get(
      selector: binding.SelectorArg,
      options?: binding.GetOptions | undefined | null,
    ): Promise<TypedHandler<binding.Reply>>
  }
  interface Subscriber {
    [Symbol.dispose](): void
    recv(): Promise<binding.Sample>
    tryRecv(): binding.Sample | null
    stream(): TypedStream<binding.Sample>
  }
  interface Publisher {
    [Symbol.dispose](): void
  }
  interface MatchingListener {
    [Symbol.dispose](): void
    recv(): Promise<binding.MatchingStatus>
    tryRecv(): binding.MatchingStatus | null
    stream(): TypedStream<binding.MatchingStatus>
  }
  interface SampleMissListener {
    [Symbol.dispose](): void
    recv(): Promise<binding.Miss>
    tryRecv(): binding.Miss | null
    stream(): TypedStream<binding.Miss>
  }
  interface Scout {
    [Symbol.dispose](): void
    recv(): Promise<binding.Hello>
    tryRecv(): binding.Hello | null
    stream(): TypedStream<binding.Hello>
  }
  interface LivelinessToken {
    [Symbol.dispose](): void
  }
  interface Liveliness {
    get(
      keyExpr: binding.KeyExprArg,
      options?: binding.LivelinessGetOptions | undefined | null,
    ): Promise<TypedHandler<binding.Reply>>
  }
  interface LivelinessSubscriber {
    [Symbol.dispose](): void
    recv(): Promise<binding.Sample>
    tryRecv(): binding.Sample | null
    stream(): TypedStream<binding.Sample>
  }
  interface Querier {
    [Symbol.dispose](): void
    get(
      options?: binding.QuerierGetOptions | undefined | null,
    ): Promise<TypedHandler<binding.Reply>>
  }
  interface Queryable {
    [Symbol.dispose](): void
    recv(): Promise<binding.Query>
    tryRecv(): binding.Query | null
    stream(): TypedStream<binding.Query>
  }
  interface LinkEventsListener {
    [Symbol.dispose](): void
    recv(): Promise<binding.LinkEvent>
    tryRecv(): binding.LinkEvent | null
    stream(): TypedStream<binding.LinkEvent>
  }
  interface TransportEventsListener {
    [Symbol.dispose](): void
    recv(): Promise<binding.TransportEvent>
    tryRecv(): binding.TransportEvent | null
    stream(): TypedStream<binding.TransportEvent>
  }
}

export interface Handler<T> extends binding.Handler {
  recv(): Promise<T>
  tryRecv(): T | null
  stream(): Stream<T>
}

/**
 * File-scope aliases for use inside the `declare module './binding.js'` block
 * above: bare `Handler`/`Stream` there resolve to the augmented module's own
 * monomorphic classes (module-local names shadow this file's generics), and
 * `skipLibCheck` would silently turn the bogus instantiation into `any`.
 */
type TypedHandler<T> = Handler<T>
type TypedStream<T> = Stream<T>

/**
 * A native {@link binding.Stream} async-iterator cursor that carries its
 * payload type `T` at compile time. `T` is erased at runtime; every cursor is
 * the same monomorphic native `Stream`.
 */
export interface Stream<T> extends binding.Stream {
  [Symbol.asyncIterator](): AsyncGenerator<T, void, undefined>
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
