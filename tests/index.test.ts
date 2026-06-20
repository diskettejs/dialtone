import { expect, test } from 'vitest'
import {
  Config,
  CongestionControl,
  Locality,
  Priority,
  Reliability,
  Session,
} from '../index.js'

test('Config factory methods construct instances', () => {
  expect(Config.default()).toBeInstanceOf(Config)
  expect(Config.fromJson5('{}')).toBeInstanceOf(Config)
})

test('Session opens, exposes a zid, and closes', async () => {
  const session = await Session.open()

  expect(session.isClosed).toBe(false)
  expect(typeof session.zid).toBe('string')
  expect(session.zid.length).toBeGreaterThan(0)

  await session.close()
  expect(session.isClosed).toBe(true)
}, 15_000)

test('Session.open accepts an explicit Config', async () => {
  const session = await Session.open(Config.default())
  await session.close()
}, 15_000)

test('Session.newTimestamp returns an NTP64 time and id', async () => {
  const session = await Session.open()
  const timestamp = session.newTimestamp()

  expect(typeof timestamp.time).toBe('bigint')
  expect(typeof timestamp.id).toBe('string')

  await session.close()
}, 15_000)

test('Session.put and delete resolve, with the full option set', async () => {
  const session = await Session.open()
  const timestamp = session.newTimestamp()

  await session.put('demo/zenoh-ts/value', 'hello')
  await session.put('demo/zenoh-ts/value', Buffer.from([1, 2, 3]), {
    encoding: 'application/octet-stream',
    attachment: 'metadata',
    congestionControl: CongestionControl.Block,
    priority: Priority.DataHigh,
    express: true,
    reliability: Reliability.Reliable,
    allowedDestination: Locality.Any,
    timestamp,
    sourceInfo: { sourceId: { zid: session.zid, eid: 0 }, sourceSn: 0 },
  })
  await session.delete('demo/zenoh-ts/value', { timestamp })

  await session.close()
}, 15_000)

test('Session.info reports the session and peer ids', async () => {
  const session = await Session.open()
  const info = session.info()

  expect(await info.zid()).toBe(session.zid)
  expect(Array.isArray(await info.routersZid())).toBe(true)
  expect(Array.isArray(await info.peersZid())).toBe(true)

  await session.close()
}, 15_000)
