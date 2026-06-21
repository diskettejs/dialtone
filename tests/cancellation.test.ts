// Concern: the CancellationToken binding seam — its own state, and the three get
// sites that accept it.
//
// What's squarely the binding's job here: that `cancel()` resolves and flips
// `isCancelled`; that cancelling disconnects a bound get's reply channel (surfaced
// as `recv()` → null); that a get bound to an already-cancelled token rejects
// (Zenoh's error mapped across the boundary); and that one shared token threads
// into `session.get`, `querier.get`, and `liveliness().get`. Zenoh owns the
// interruption mechanics themselves — we only assert the seam.
import { expect, test } from 'vitest'
import { CancellationToken, Session } from '../index.js'
import { bounded, ke, recvWithin } from './helpers.ts'

test('a fresh token is not cancelled, and cancel() flips that', async () => {
  const token = new CancellationToken()
  expect(token.isCancelled).toBe(false)
  await token.cancel()
  expect(token.isCancelled).toBe(true)
})

test('cancel() is idempotent', async () => {
  const token = new CancellationToken()
  await token.cancel()
  await expect(token.cancel()).resolves.toBeUndefined()
  expect(token.isCancelled).toBe(true)
})

test('cancel interrupts an in-flight get and closes its reply channel', async () => {
  await using session = await Session.open()
  const token = new CancellationToken()
  // A long timeout so the channel would NOT close on its own within the assertion
  // window — only cancellation can close it that fast.
  const replies = await session.get(ke('cancel-get'), { consolidation: 'None', timeout: 30_000 }, token)
  expect(token.isCancelled).toBe(false)

  await token.cancel()
  expect(token.isCancelled).toBe(true)

  // Disconnected channel: recv resolves to null promptly, well before the 30s
  // query timeout would have closed it.
  const reply = await recvWithin(() => replies.recv(), 2_000)
  expect(reply).toBeNull()
})

test('a get bound to an already-cancelled token rejects', async () => {
  await using session = await Session.open()
  const token = new CancellationToken()
  await token.cancel()

  await expect(session.get(ke('cancelled-get'), bounded, token)).rejects.toThrow()
})

test('one shared token threads into session, querier, and liveliness gets', async () => {
  await using session = await Session.open()
  const token = new CancellationToken()

  const sReplies = await session.get(ke('cancel-session'), { ...bounded }, token)
  expect(sReplies).toBeDefined()

  // Consolidation/timeout are fixed at querier-declare time, not per-get.
  const querier = await session.declareQuerier(ke('cancel-querier'), { consolidation: 'None', timeout: 500 })
  const qReplies = await querier.get(undefined, token)
  expect(qReplies).toBeDefined()

  const lReplies = await session.liveliness().get(ke('cancel-liveliness'), { timeout: 500 }, token)
  expect(lReplies).toBeDefined()

  // Cancelling the one shared token interrupts every get it was passed to.
  await token.cancel()
  querier.undeclare()
})
