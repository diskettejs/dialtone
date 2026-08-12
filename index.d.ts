export * from './binding.js'
export type * from './config.js'

import * as binding from './binding.js'
import type { ZenohConfig } from './config.js'

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
 * Builds a {@link binding.Config | Config} from a typed config object, ready to pass to
 * `Session.open`. Equivalent to `Config.fromJson5(JSON.stringify(config))`;
 * Zenoh validates the result eagerly and throws on invalid values or unknown keys
 * (except inside `connect.retry` and `listen.retry`, where unknown keys are ignored).
 *
 * @example
 * await Session.open(defineConfig({ mode: 'router' }))
 */
export declare function defineConfig(config?: ZenohConfig): binding.Config
