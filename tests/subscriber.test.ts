import { describe, expect, test } from 'vitest'
import { Session } from '../index.js'
import { loopbackConfig } from './loopback.js'

describe('Subscriber', () => {
  describe('fifo', () => {
    test('stream() yields the put sample, then ends when undeclared', async () => {
      await using session = await Session.open(loopbackConfig())
      const sub = await session.declareSubscriber('dialtone/test/fifo/stream')

      await session.put('dialtone/test/fifo/stream', 'hello')

      // Consume without `break`: after the sample, undeclare the subscription —
      // dropping the channel's sender disconnects it, so the next iteration ends
      // the stream. Reaching the assertion below is itself the proof the loop
      // terminated rather than blocking on a next sample that never arrives.
      const received: string[] = []
      for await (const sample of sub.handler.stream()) {
        expect(sample.keyExpr.asStr).toBe('dialtone/test/fifo/stream')
        received.push(sample.payload.toString())
        await sub.undeclare()
      }

      expect(received).toEqual(['hello'])
    })

    test('exposes channel introspection', async () => {
      await using session = await Session.open(loopbackConfig())
      await using sub = await session.declareSubscriber('dialtone/test/fifo/introspect')

      // Default channel is a FIFO of 256.
      expect(sub.handler.capacity).toBe(256)
      expect(sub.handler.isEmpty).toBe(true)
      expect(sub.handler.isDisconnected).toBe(false)
    })

    test('recv() resolves the put sample', async () => {
      await using session = await Session.open(loopbackConfig())
      await using sub = await session.declareSubscriber('dialtone/test/fifo/recv')

      await session.put('dialtone/test/fifo/recv', 'hello')

      const sample = await sub.handler.recv()
      expect(sample.payload.toString()).toBe('hello')
    })

    test('recvTimeout() resolves null when no sample arrives', async () => {
      await using session = await Session.open(loopbackConfig())
      await using sub = await session.declareSubscriber('dialtone/test/fifo/recvTimeout/empty')

      // Nothing is published, so the wait expires and yields null.
      expect(await sub.handler.recvTimeout(50)).toBeNull()
    })

    test('recvTimeout() resolves a sample when one is available', async () => {
      await using session = await Session.open(loopbackConfig())
      await using sub = await session.declareSubscriber('dialtone/test/fifo/recvTimeout/value')

      await session.put('dialtone/test/fifo/recvTimeout/value', 'hi')

      const sample = await sub.handler.recvTimeout(1000)
      expect(sample?.payload.toString()).toBe('hi')
    })

    test('recvDeadline() resolves null once the deadline has passed', async () => {
      await using session = await Session.open(loopbackConfig())
      await using sub = await session.declareSubscriber('dialtone/test/fifo/recvDeadline')

      // A deadline ~50ms out with nothing published: returns null when reached.
      expect(await sub.handler.recvDeadline(Date.now() + 50)).toBeNull()
    })

    test('drain() returns all queued samples in order, without blocking', async () => {
      await using session = await Session.open(loopbackConfig())
      await using sub = await session.declareSubscriber('dialtone/test/fifo/drain')

      await session.put('dialtone/test/fifo/drain', 'a')
      await session.put('dialtone/test/fifo/drain', 'b')
      await session.put('dialtone/test/fifo/drain', 'c')

      const drained = sub.handler.drain().map((s) => s.payload.toString())
      expect(drained).toEqual(['a', 'b', 'c'])
      expect(sub.handler.isEmpty).toBe(true)
    })
  })

  describe('ring', () => {
    test('recvAsync() resolves a sample', async () => {
      await using session = await Session.open(loopbackConfig())
      await using sub = await session.declareSubscriber('dialtone/test/ring/recv', {
        handler: { kind: 'Ring', capacity: 3 },
      })

      await session.put('dialtone/test/ring/recv', 'latest')

      const sample = await sub.handler.recvAsync()
      expect(sample.payload.toString()).toBe('latest')
    })

    test('recv() resolves the put sample', async () => {
      await using session = await Session.open(loopbackConfig())
      await using sub = await session.declareSubscriber('dialtone/test/ring/recvSync', {
        handler: { kind: 'Ring', capacity: 3 },
      })

      await session.put('dialtone/test/ring/recvSync', 'latest')

      const sample = await sub.handler.recv()
      expect(sample.payload.toString()).toBe('latest')
    })

    test('recvTimeout() resolves null when the ring stays empty', async () => {
      await using session = await Session.open(loopbackConfig())
      await using sub = await session.declareSubscriber('dialtone/test/ring/recvTimeout', {
        handler: { kind: 'Ring', capacity: 3 },
      })

      expect(await sub.handler.recvTimeout(50)).toBeNull()
    })
  })
})
