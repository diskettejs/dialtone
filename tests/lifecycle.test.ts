// Concern: lifecycle state and disposal — including the one thing that lives in
// hand-written JS (`index.js`).
//
// NAPI-RS has no codegen for the dispose symbols, so `index.js` attaches
// `Symbol.dispose`/`Symbol.asyncDispose` onto the prototypes to make `using` /
// `await using` drive `undeclare()` / `close()`. These tests exercise that wiring
// directly, plus the wrapper's own state: `isClosed`, idempotent undeclare, and
// the "use after undeclare throws" guard.
import { expect, test } from 'vitest'
import { Session } from '../index.js'
import type { Session as SessionType, Subscriber } from '../index.js'
import { ke, recvWithin } from './helpers.ts'

test('a session reports isClosed across its lifecycle and exposes a zid', async () => {
  const session = await Session.open()
  expect(session.isClosed).toBe(false)
  expect(typeof session.zid).toBe('string')
  expect(session.zid.length).toBeGreaterThan(0)

  await session.close()
  expect(session.isClosed).toBe(true)
})

test('await using drives Session close at scope exit', async () => {
  let captured: SessionType | undefined
  {
    await using session = await Session.open()
    captured = session
    expect(session.isClosed).toBe(false)
  }
  // Leaving the block invokes Symbol.asyncDispose → close(), and awaits it.
  expect(captured!.isClosed).toBe(true)
})

test('using + await using compose: entities undeclare before the session closes', async () => {
  const k = ke('compose')
  let session: SessionType | undefined
  let subscriber: Subscriber | undefined
  {
    await using s = await Session.open()
    using sub = await s.declareSubscriber(k)
    session = s
    subscriber = sub

    await s.put(k, 'x')
    const sample = await recvWithin(() => sub.recv())
    expect(sample!.payload.toString()).toBe('x')
  }
  // Disposal is LIFO: sub.undeclare() runs first — tryRecv now reports closed —
  // then session.close() is awaited.
  expect(() => subscriber!.tryRecv()).toThrow()
  expect(session!.isClosed).toBe(true)
})

test('undeclare is idempotent', async () => {
  await using session = await Session.open()
  const subscriber = await session.declareSubscriber(ke('idempotent'))

  subscriber.undeclare()
  expect(() => subscriber.undeclare()).not.toThrow()
})

test('getters throw after the entity is undeclared', async () => {
  await using session = await Session.open()
  const subscriber = await session.declareSubscriber(ke('use-after-undeclare'))

  subscriber.undeclare()
  expect(() => subscriber.keyExpr).toThrow('subscriber has been undeclared')
})
