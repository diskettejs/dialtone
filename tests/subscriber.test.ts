import { describe, expect, it } from 'vitest'
import { Session } from '../index.js'

const settle = () => new Promise((r) => setTimeout(r, 200))
const decode = (b: Uint8Array) => new TextDecoder().decode(b)

describe('subscriber handler option', () => {
  it('defaults to a fifo channel that delivers every sample in order', async () => {
    const session = await Session.open()
    const sub = await session.declareSubscriber('demo/fifo', {
      handler: { type: 'fifo', capacity: 8 },
    })
    await settle()

    await session.put('demo/fifo', 'a')
    await session.put('demo/fifo', 'b')

    expect(decode((await sub.receive())!.payload)).toBe('a')
    expect(decode((await sub.receive())!.payload)).toBe('b')

    await sub.undeclare()
    await session.close()
  }, 10_000)

  it('ring channel keeps the most recent sample and drops the oldest when full', async () => {
    const session = await Session.open()
    const sub = await session.declareSubscriber('demo/ring', {
      handler: { type: 'ring', capacity: 1 },
    })
    await settle()

    // Nothing is consuming yet, so the capacity-1 ring keeps overwriting.
    await session.put('demo/ring', 'a')
    await session.put('demo/ring', 'b')
    await session.put('demo/ring', 'c')
    await settle()

    const sample = await sub.receive()
    expect(decode(sample!.payload)).toBe('c')

    await sub.undeclare()
    await session.close()
  }, 10_000)
})
