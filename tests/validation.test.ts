// Concern: validation the wrapper performs itself, and error translation.
//
// Two flavours of "this must throw":
//   - Wrapper-owned rules — the recovery mutual-exclusion (re-checked in Rust
//     because zenoh-ext enforces it in the type system, which TS can't express)
//     and the advanced-only-method guards on plain liveliness subscribers. These
//     messages are *ours*, so we assert them verbatim.
//   - Error translation — a Zenoh `Result::Err` (bad key expr, bad timestamp id,
//     malformed config) must surface as a thrown JS Error. The wording is Zenoh's
//     and version-dependent, so we assert only that it throws.
import { expect, test } from 'vitest'
import { Config, KeyExpr, Session } from '../index.js'
import { ke } from './helpers.ts'

test('recovery accepts exactly one of heartbeat / periodicQueriesMs', async () => {
  await using session = await Session.open()

  // The two valid arms declare a subscriber.
  using heartbeatSub = await session.declareSubscriber(ke('rec-hb'), {
    recovery: { heartbeat: true },
  })
  expect(heartbeatSub.keyExpr.toString()).toContain('rec-hb')

  using periodicSub = await session.declareSubscriber(ke('rec-periodic'), {
    recovery: { periodicQueriesMs: 1_000 },
  })
  expect(periodicSub.keyExpr.toString()).toContain('rec-periodic')

  // Neither arm and both arms are rejected — with the wrapper's own wording.
  await expect(session.declareSubscriber(ke('rec-neither'), { recovery: {} })).rejects.toThrow(
    'recovery requires exactly one of `heartbeat: true` or `periodicQueriesMs`',
  )
  await expect(
    session.declareSubscriber(ke('rec-both'), {
      recovery: { heartbeat: true, periodicQueriesMs: 1_000 },
    }),
  ).rejects.toThrow('recovery options `heartbeat` and `periodicQueriesMs` are mutually exclusive')
})

test('advanced-only methods reject on plain liveliness subscribers', async () => {
  await using session = await Session.open()
  // Liveliness subscribers are plain, not advanced — the advanced-only methods
  // must surface a clear error rather than misbehave.
  const subscriber = await session.liveliness().declareSubscriber(ke('live/**'))

  await expect(subscriber.sampleMissListener()).rejects.toThrow(
    'sample miss detection is not available on liveliness subscribers',
  )
  await expect(subscriber.detectPublishers()).rejects.toThrow(
    'publisher detection is not available on liveliness subscribers',
  )

  subscriber.undeclare()
})

test('the KeyExpr constructor rejects non-canon input; autocanonize repairs it', () => {
  // The constructor translates Zenoh's canon-form rejection into a throw...
  expect(() => new KeyExpr('demo/**/**/x')).toThrow()
  // ...while autocanonize returns a valid KeyExpr instance (the specific canon
  // string is Zenoh's algorithm, not asserted here).
  expect(KeyExpr.autocanonize('demo/**/**/x')).toBeInstanceOf(KeyExpr)
})

test('a key-expression argument rejects a value that is not a string or KeyExpr', async () => {
  await using session = await Session.open()
  // Passing some other object where a key expression is expected is rejected at
  // the FFI boundary (during argument conversion, so it throws synchronously)
  // rather than reinterpreting its pointer as a KeyExpr.
  expect(() => session.put({} as unknown as KeyExpr, 'x')).toThrow(/KeyExpr/)
})

test('Config.fromJson5 rejects malformed input', () => {
  expect(() => Config.fromJson5('definitely not json5 ::')).toThrow()
})

test('put rejects a timestamp whose id is not a valid Zenoh id', async () => {
  await using session = await Session.open()
  await expect(
    session.put(ke('bad-ts'), 'x', { timestamp: { time: 1n, id: 'nothex' } }),
  ).rejects.toThrow()
})
