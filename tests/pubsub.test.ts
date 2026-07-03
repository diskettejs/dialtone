import { afterAll, beforeAll, describe, expect, test } from 'vitest'

import { defineConfig, Session } from '../index.js'

// A publisher and subscriber declared on the same session exchange samples
// via session-local delivery (Locality.Any, the default, includes
// SessionLocal) — no router, no network, no discovery race. That's enough to
// prove the JS <-> Rust seam is wired correctly; Zenoh's own distributed
// routing is already covered by Zenoh's test suite.

let session: Session
let counter = 0
const nextKey = () => `test/pubsub/${counter++}`

beforeAll(async () => {
  session = await Session.open(defineConfig())
})

afterAll(async () => {
  await session.close()
})

describe('publish -> subscribe wiring', () => {
  test('put is received via recv()', async () => {
    const key = nextKey()
    const publisher = await session.declarePublisher(key)
    const subscriber = await session.declareSubscriber(key)

    await publisher.put('hello')
    const sample = await subscriber.recv()

    expect(sample.payload.tryToString()).toBe('hello')
    expect(sample.keyExpr.toString()).toBe(key)
    expect(sample.kind).toBe('Put')
    expect(subscriber.tryRecv()).toBeNull()

    subscriber.undeclare()
    publisher.undeclare()
  })

  test('put is received via stream() async iterator', async () => {
    const key = nextKey()
    const publisher = await session.declarePublisher(key)
    const subscriber = await session.declareSubscriber(key)

    await publisher.put('streamed')

    for await (const sample of subscriber.stream()) {
      expect(sample.payload.tryToString()).toBe('streamed')
      break
    }

    subscriber.undeclare()
    publisher.undeclare()
  })

  test('delete produces a Delete-kind sample', async () => {
    const key = nextKey()
    const publisher = await session.declarePublisher(key)
    const subscriber = await session.declareSubscriber(key)

    await publisher.delete()
    const sample = await subscriber.recv()

    expect(sample.kind).toBe('Delete')

    subscriber.undeclare()
    publisher.undeclare()
  })

  test('PublisherPutOptions (encoding, attachment) round-trip onto the sample', async () => {
    const key = nextKey()
    const publisher = await session.declarePublisher(key)
    const subscriber = await session.declareSubscriber(key)

    await publisher.put('with options', { encoding: 'text/plain', attachment: 'meta' })
    const sample = await subscriber.recv()

    expect(sample.encoding.toString()).toBe('text/plain')
    expect(sample.attachment?.tryToString()).toBe('meta')

    subscriber.undeclare()
    publisher.undeclare()
  })

  test('declare-time PublisherOptions (priority, congestionControl) set the publisher', async () => {
    const key = nextKey()
    const publisher = await session.declarePublisher(key, {
      priority: 'DataHigh',
      congestionControl: 'Block',
    })

    expect(publisher.priority).toBe('DataHigh')
    expect(publisher.congestionControl).toBe('Block')

    publisher.undeclare()
  })
})

describe('matching status and listener', () => {
  test('matchingStatus() reflects whether a subscriber currently exists', async () => {
    const key = nextKey()
    const publisher = await session.declarePublisher(key)

    expect((await publisher.matchingStatus()).matching).toBe(false)

    const subscriber = await session.declareSubscriber(key)
    expect((await publisher.matchingStatus()).matching).toBe(true)

    subscriber.undeclare()
    publisher.undeclare()
  })

  // A MatchingListener delivers the current status immediately on declare
  // when a match already exists (zenoh/api/session.rs,
  // declare_matches_listener_inner) — declaring the subscriber first makes
  // that first delivery deterministic, no sleep required.
  test('matchingListener() delivers the current status immediately on declare', async () => {
    const key = nextKey()
    const subscriber = await session.declareSubscriber(key)
    const publisher = await session.declarePublisher(key)

    const listener = await publisher.matchingListener()
    const status = await listener.recv()

    expect(status.matching).toBe(true)
    expect(listener.tryRecv()).toBeNull()

    listener.undeclare()
    subscriber.undeclare()
    publisher.undeclare()
  })
})

describe('detect publishers via liveliness', () => {
  test("detectPublishers() observes a publisher's liveliness token appear and disappear", async () => {
    const key = nextKey()
    const subscriber = await session.declareSubscriber(key)
    const detector = await subscriber.detectPublishers()

    const publisher = await session.declarePublisher(key, { publisherDetection: true })
    expect((await detector.recv()).kind).toBe('Put')

    publisher.undeclare()
    expect((await detector.recv()).kind).toBe('Delete')

    detector.undeclare()
    subscriber.undeclare()
  })
})

// Missed samples are only detectable from a real sequence-number gap between
// an AdvancedPublisher (sampleMissDetection enabled) and its subscriber.
// Zenoh's own test for this kills and restarts a router mid-test to force
// that gap (zenoh-ext/tests/advanced.rs::test_advanced_sample_miss) — that's
// Zenoh's retransmission/detection logic, not reproducible (or worth
// reproducing) in a session-local unit test. This only proves the listener
// declares and hands back a working handle.
describe('sample miss listener', () => {
  test('sampleMissListener() declares and returns an idle handle', async () => {
    const key = nextKey()
    const subscriber = await session.declareSubscriber(key)
    const missListener = await subscriber.sampleMissListener()

    expect(missListener.tryRecv()).toBeNull()

    missListener.undeclare()
    subscriber.undeclare()
  })
})

describe('undeclare guards further use', () => {
  test('put after publisher.undeclare() rejects', async () => {
    const key = nextKey()
    const publisher = await session.declarePublisher(key)
    publisher.undeclare()

    await expect(publisher.put('x')).rejects.toThrow(/undeclared/i)
  })

  test('recv after subscriber.undeclare() rejects', async () => {
    const key = nextKey()
    const subscriber = await session.declareSubscriber(key)
    subscriber.undeclare()

    await expect(subscriber.recv()).rejects.toThrow(/undeclared/i)
  })
})
