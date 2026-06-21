// Concern: the channel/receiver machinery the wrapper owns on top of Zenoh's
// handlers.
//
// This is the wrapper's own translation layer: Zenoh's `Ok(Some)/Ok(None)/Err`
// receiver outcomes become JS `value`/`null`/`throw`; the async-iterator protocol
// is wired by hand; the FIFO/Ring choice routes to the right handler; and undeclare
// releases the receiver. We assert the *protocol*, never Zenoh's buffering policy
// (e.g. that a Ring drops the oldest — that's Zenoh's, covered nowhere here).
import { expect, test } from 'vitest'
import { Config, Session, scout } from '../index.js'
import { ke, recvWithin } from './helpers.ts'

test('tryRecv distinguishes empty (null) from closed (throws)', async () => {
  await using session = await Session.open()
  const subscriber = await session.declareSubscriber(ke('tryrecv'))

  // Connected, nothing published yet: empty but open → null.
  expect(subscriber.tryRecv()).toBeNull()

  // Undeclaring disconnects the channel; tryRecv must surface that as distinct
  // from "empty" so a polling loop can terminate instead of spinning.
  subscriber.undeclare()
  expect(() => subscriber.tryRecv()).toThrow('subscriber has been undeclared')
})

test('recv resolves null once the subscriber is undeclared', async () => {
  await using session = await Session.open()
  const k = ke('recv-null')
  const subscriber = await session.declareSubscriber(k)

  await session.put(k, 'a')
  const first = await recvWithin(() => subscriber.recv())
  expect(first!.payload.toString()).toBe('a')

  // Undeclare releases the receiver with the declaration (the wrapper's
  // "drop, don't drain" choice), so a subsequent recv ends rather than blocking.
  subscriber.undeclare()
  expect(await subscriber.recv()).toBeNull()
})

test('a subscriber is an async iterator that terminates on break', async () => {
  await using session = await Session.open()
  const k = ke('iterator')
  using subscriber = await session.declareSubscriber(k)

  await session.put(k, 'a')
  await session.put(k, 'b')

  const received: string[] = []
  for await (const sample of subscriber) {
    received.push(sample.payload.toString())
    if (received.length === 2) break
  }
  // The iterator protocol is the point — it yields samples and stops cleanly on
  // break. (Delivery order is Zenoh's FIFO guarantee, so assert the set, not the
  // sequence.)
  expect(new Set(received)).toEqual(new Set(['a', 'b']))
})

test('the Ring handler is wired and delivers through recv', async () => {
  await using session = await Session.open()
  const k = ke('ring')
  using subscriber = await session.declareSubscriber(k, {
    handler: { kind: 'Ring', capacity: 4 },
  })

  await session.put(k, 'x')

  // Only assertion: selecting Ring routes to a working RingChannel-backed
  // receiver. The drop-oldest *policy* of a full ring is Zenoh's, not ours.
  const sample = await recvWithin(() => subscriber.recv())
  expect(sample!.payload.toString()).toBe('x')
})

test('a matching listener polls non-blocking and undeclares', async () => {
  await using session = await Session.open()
  using publisher = await session.declarePublisher(ke('matching-listener'))

  const listener = await publisher.matchingListener()
  // No matching change observed yet: a non-blocking poll is null, never throws.
  expect(listener.tryRecv()).toBeNull()
  listener.undeclare()
  expect(() => listener.tryRecv()).toThrow()
})

test('scout polls non-blocking and throws once stopped', async () => {
  // An empty matcher (scout everything) and a Ring handler both marshal in.
  const handle = await scout([], Config.default(), { kind: 'Ring', capacity: 4 })

  // Before stopping: a non-blocking poll returns (a buffered Hello or null) and
  // never throws — "open but empty" must stay distinct from "closed".
  expect(() => handle.tryRecv()).not.toThrow()

  // Stopping closes the channel: tryRecv is now the "closed" outcome.
  handle.stop()
  expect(() => handle.tryRecv()).toThrow()
})
