export * from './binding.js'
export * from './config.js'

import * as binding from './binding.js'

declare module './binding.js' {
  interface Session {
    [Symbol.asyncDispose](): Promise<void>
  }
  interface Subscriber {
    [Symbol.asyncDispose](): Promise<void>
  }
  interface Publisher {
    [Symbol.asyncDispose](): Promise<void>
  }
  interface MatchingListener {
    [Symbol.asyncDispose](): Promise<void>
  }
  interface SampleMissListener {
    [Symbol.asyncDispose](): Promise<void>
  }
  interface Scout {
    [Symbol.dispose](): void
  }
  interface LivelinessToken {
    [Symbol.asyncDispose](): Promise<void>
  }
  interface LivelinessSubscriber {
    [Symbol.asyncDispose](): Promise<void>
  }
  interface Querier {
    [Symbol.asyncDispose](): Promise<void>
  }
  interface Queryable {
    [Symbol.asyncDispose](): Promise<void>
  }
}
