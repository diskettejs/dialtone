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
        priority: 'data_high',
        congestionControl: 'block',
      })

      expect(publisher.priority).toBe('data_high')
      expect(publisher.congestionControl).toBe('block')
    })
  })

  describe('put()', () => {
    test('delivers the payload to a matching subscriber', async () => {
      const key = 'test/pubsub/put'
      using publisher = await session.declarePublisher(key)
      using subscriber = await session.declareSubscriber(key)

      await publisher.put('hello')

      for await (const sample of subscriber.receive()) {
        expect(sample.payload.toString()).toBe('hello')
        expect(sample.keyExpr.toString()).toBe(key)
        expect(sample.kind).toBe('put')
        break
      }
    })

    test('applies encoding and attachment from PublisherPutOptions', async () => {
      const key = 'test/pubsub/put-options'
      using publisher = await session.declarePublisher(key)
      using subscriber = await session.declareSubscriber(key)

      await publisher.put('with options', { encoding: 'text/plain', attachment: 'meta' })

      for await (const sample of subscriber.receive()) {
        expect(sample.encoding.toString()).toBe('text/plain')
        expect(sample.attachment?.toString()).toBe('meta')
        break
      }
    })
  })

  describe('delete()', () => {
    test('delivers a Delete-kind sample', async () => {
      const key = 'test/pubsub/delete'
      using publisher = await session.declarePublisher(key)
      using subscriber = await session.declareSubscriber(key)

      await publisher.delete()

      for await (const sample of subscriber.receive()) {
        expect(sample.kind).toBe('delete')
        break
      }
    })

    test('applies attachment from PublisherDeleteOptions', async () => {
      const key = 'test/pubsub/delete-options'
      using publisher = await session.declarePublisher(key)
      using subscriber = await session.declareSubscriber(key)

      await publisher.delete({ attachment: 'meta' })

      for await (const sample of subscriber.receive()) {
        expect(sample.attachment?.toString()).toBe('meta')
        break
      }
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

      for await (const status of listener.receive()) {
        expect(status.matching).toBe(true)
        break
      }
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

      // Draining the buffer lets the blocked put through, in order.
      const seen: (string | null)[] = []
      for await (const sample of subscriber.receive()) {
        seen.push(sample.payload.toString())
        if (seen.length === 2) break
      }
      await second

      expect(seen).toEqual(['first', 'second'])
    })
  })

  describe('receive()', () => {
    test('drives a for await loop across successive samples', async () => {
      const key = 'test/pubsub/receive-loop'
      using publisher = await session.declarePublisher(key)
      using subscriber = await session.declareSubscriber(key)

      await publisher.put('one')
      await publisher.put('two')
      await publisher.put('three')

      const seen: (string | null)[] = []
      for await (const sample of subscriber.receive()) {
        seen.push(sample.payload.toString())
        if (seen.length === 3) break
      }

      expect(seen).toEqual(['one', 'two', 'three'])
    })

    test('completes the stream instead of throwing once the subscriber is undeclared', async () => {
      const key = 'test/pubsub/receive-after-undeclare'
      using publisher = await session.declarePublisher(key)
      const subscriber = await session.declareSubscriber(key)

      await publisher.put('last')

      // Undeclaring from inside the loop closes the channel, ending iteration.
      const seen: (string | null)[] = []
      for await (const sample of subscriber.receive()) {
        seen.push(sample.payload.toString())
        subscriber.undeclare()
      }

      expect(seen).toEqual(['last'])
    })

    test('throws on receive() after undeclare', async () => {
      const subscriber = await session.declareSubscriber('test/pubsub/receive-undeclared')
      subscriber.undeclare()

      expect(() => subscriber.receive()).toThrow(/has already been consumed/i)
    })
  })

  describe('detectPublishers()', () => {
    test("observes a publisher's liveliness token", async () => {
      const key = 'test/pubsub/detect-publishers'
      using subscriber = await session.declareSubscriber(key)
      using detector = await subscriber.detectPublishers()

      using _publisher = await session.declarePublisher(key, { publisherDetection: true })

      for await (const sample of detector.receive()) {
        expect(sample.kind).toBe('put')
        break
      }
    })
  })
})
