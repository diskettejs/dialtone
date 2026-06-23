import { describe, expect, test } from 'vitest'
import { Session } from '../index.js'
import { loopbackConfig } from './loopback.js'

describe('Publisher round-trip', () => {
  test('put() reaches a matching subscriber', async () => {
    await using session = await Session.open(loopbackConfig())
    await using sub = await session.declareSubscriber('dialtone/test/pub/put')
    await using pub = await session.declarePublisher('dialtone/test/pub/put')

    await pub.put('hello')

    const sample = await sub.handler.recvAsync()
    expect(sample.keyExpr.asStr).toBe('dialtone/test/pub/put')
    expect(sample.kind).toBe('Put')
    expect(sample.payload.toString()).toBe('hello')
  })

  test('put() carries a Uint8Array payload', async () => {
    await using session = await Session.open(loopbackConfig())
    await using sub = await session.declareSubscriber('dialtone/test/pub/bytes')
    await using pub = await session.declarePublisher('dialtone/test/pub/bytes')

    await pub.put(new Uint8Array([1, 2, 3]))

    const sample = await sub.handler.recvAsync()
    expect(Array.from(sample.payload.toBytes())).toEqual([1, 2, 3])
  })

  test('delete() sends a Delete sample', async () => {
    await using session = await Session.open(loopbackConfig())
    await using sub = await session.declareSubscriber('dialtone/test/pub/del')
    await using pub = await session.declarePublisher('dialtone/test/pub/del')

    await pub.delete()

    const sample = await sub.handler.recvAsync()
    expect(sample.kind).toBe('Delete')
  })

  test('exposes keyExpr, id, and the fixed QoS', async () => {
    await using session = await Session.open(loopbackConfig())
    await using pub = await session.declarePublisher('dialtone/test/pub/meta', {
      priority: 'DataHigh',
      congestionControl: 'Block',
    })

    expect(pub.keyExpr.asStr).toBe('dialtone/test/pub/meta')
    expect(pub.priority).toBe('DataHigh')
    expect(pub.congestionControl).toBe('Block')
    expect(typeof pub.id.zid).toBe('string')
    expect(pub.id.zid.length).toBeGreaterThan(0)
  })
})
