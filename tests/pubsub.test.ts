import { afterAll, beforeAll, describe, expect, test } from 'vitest'

import { defineConfig, Session } from '../index.js'

let session: Session

beforeAll(async () => {
  session = await Session.open(defineConfig())
})

afterAll(async () => {
  await session.close()
})

describe('Publisher', () => {
  describe('declaration', () => {
    test('applies priority and congestionControl from PublisherOptions', async () => {
      const key = 'test/pubsub/declare-options'
      using publisher = await session.declarePublisher(key, {
        priority: 'DataHigh',
        congestionControl: 'Block',
      })

      expect(publisher.priority).toBe('DataHigh')
      expect(publisher.congestionControl).toBe('Block')
    })
  })

  describe('put()', () => {
    test('delivers the payload to a matching subscriber', async () => {
      const key = 'test/pubsub/put'
      using publisher = await session.declarePublisher(key)
      using subscriber = await session.declareSubscriber(key)

      await publisher.put('hello')
      const sample = await subscriber.recv()

      expect(sample.payload.tryToString()).toBe('hello')
      expect(sample.keyExpr.toString()).toBe(key)
      expect(sample.kind).toBe('Put')
    })

    test('applies encoding and attachment from PublisherPutOptions', async () => {
      const key = 'test/pubsub/put-options'
      using publisher = await session.declarePublisher(key)
      using subscriber = await session.declareSubscriber(key)

      await publisher.put('with options', { encoding: 'text/plain', attachment: 'meta' })
      const sample = await subscriber.recv()

      expect(sample.encoding.toString()).toBe('text/plain')
      expect(sample.attachment?.tryToString()).toBe('meta')
    })
  })

  describe('delete()', () => {
    test('delivers a Delete-kind sample', async () => {
      const key = 'test/pubsub/delete'
      using publisher = await session.declarePublisher(key)
      using subscriber = await session.declareSubscriber(key)

      await publisher.delete()
      const sample = await subscriber.recv()

      expect(sample.kind).toBe('Delete')
    })

    test('applies attachment from PublisherDeleteOptions', async () => {
      const key = 'test/pubsub/delete-options'
      using publisher = await session.declarePublisher(key)
      using subscriber = await session.declareSubscriber(key)

      await publisher.delete({ attachment: 'meta' })
      const sample = await subscriber.recv()

      expect(sample.attachment?.tryToString()).toBe('meta')
    })
  })

  describe('matchingStatus()', () => {
    test('reports no match when no subscriber is present', async () => {
      const key = 'test/pubsub/matching-status'
      using publisher = await session.declarePublisher(key)

      expect((await publisher.matchingStatus()).matching).toBe(false)
    })
  })

  describe('matchingListener()', () => {
    test('delivers the current status immediately on declare', async () => {
      const key = 'test/pubsub/matching-listener'
      using _subscriber = await session.declareSubscriber(key)
      using publisher = await session.declarePublisher(key)

      using listener = await publisher.matchingListener()
      const status = await listener.recv()

      expect(status.matching).toBe(true)
      expect(listener.tryRecv()).toBeNull()
    })
  })

  describe('undeclare()', () => {
    test('rejects put() after undeclare', async () => {
      const key = 'test/pubsub/undeclare'
      const publisher = await session.declarePublisher(key)
      publisher.undeclare()
      await expect(publisher.put('x')).rejects.toThrow(/has already been consumed/i)
    })
  })
})

describe('Subscriber', () => {
  describe('declaration', () => {
    test('plumbs channelCapacity to the receiving channel', async () => {
      const key = 'test/pubsub/channel-capacity'
      using publisher = await session.declarePublisher(key)
      using subscriber = await session.declareSubscriber(key, { channelCapacity: 1 })

      await publisher.put('first')

      // The channel is full, so this put cannot settle until the buffer drains.
      let settled = false
      const second = publisher.put('second').then(() => {
        settled = true
      })
      await new Promise((resolve) => setTimeout(resolve, 100))
      expect(settled).toBe(false)

      expect((await subscriber.recv()).payload.tryToString()).toBe('first')
      await second
      expect((await subscriber.recv()).payload.tryToString()).toBe('second')
    })
  })

  describe('recv()', () => {
    test('returns the next sample', async () => {
      const key = 'test/pubsub/recv'
      using publisher = await session.declarePublisher(key)
      using subscriber = await session.declareSubscriber(key)

      await publisher.put('hello')
      const sample = await subscriber.recv()

      expect(sample.payload.tryToString()).toBe('hello')
    })

    test('tryRecv() returns null when the channel is empty', async () => {
      const key = 'test/pubsub/try-recv'
      using publisher = await session.declarePublisher(key)
      using subscriber = await session.declareSubscriber(key)

      await publisher.put('only')
      await subscriber.recv()

      expect(subscriber.tryRecv()).toBeNull()
    })
  })

  describe('detectPublishers()', () => {
    test("observes a publisher's liveliness token", async () => {
      const key = 'test/pubsub/detect-publishers'
      using subscriber = await session.declareSubscriber(key)
      using detector = await subscriber.detectPublishers()

      using _publisher = await session.declarePublisher(key, { publisherDetection: true })
      expect((await detector.recv()).kind).toBe('Put')
    })
  })
})
