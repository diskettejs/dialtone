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
