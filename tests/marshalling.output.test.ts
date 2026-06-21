// Concern: outputs crossing Rust/Zenoh → JS.
//
// The wrapper's getters and return types must hand JS the right *shapes*: typed
// class instances (a KeyExpr, not a string; a Buffer, not a Uint8Array), the
// discriminated reply union, the nested id objects, and config getters that echo
// what was declared. Values are pinned in marshalling.input; here we pin types,
// the union discriminant, and the declaration→getter round-trip.
import { expect, test } from 'vitest'
import { KeyExpr, Session } from '../index.js'
import type { ReplyError, ReplySample } from '../index.js'
import { bounded, collectReplies, ke, recvWithin } from './helpers.ts'

test('a delivered Sample exposes typed instances, not raw values', async () => {
  await using session = await Session.open()
  const k = ke('sample-types')
  using sub = await session.declareSubscriber(k)
  await session.put(k, 'x')

  const sample = await recvWithin(() => sub.recv())
  // keyExpr is a KeyExpr instance (so `.intersects`/`.join` are available), and
  // the payload is a Node Buffer (so `.toString`/indexing work).
  expect(sample!.keyExpr).toBeInstanceOf(KeyExpr)
  expect(sample!.keyExpr.toString()).toBe(k)
  expect(Buffer.isBuffer(sample!.payload)).toBe(true)
  // Absent optionals come back as null, not as a throwing getter.
  expect(sample!.attachment).toBeNull()
  expect(sample!.sourceInfo).toBeNull()
})

test('the get reply union discriminates sample and error arms', async () => {
  await using session = await Session.open()
  const k = ke('reply-union')
  using queryable = await session.declareQueryable(k)

  const serve = (async () => {
    for await (const query of queryable) {
      await query.reply(k, 'answer')
      await query.replyErr('boom', { encoding: 'text/plain' })
      break
    }
  })()

  const replies = await session.get(k, { ...bounded })
  const arms = await collectReplies(replies, 2)
  await serve

  // `reply.sample` is the discriminant the wrapper exposes: a truthy Sample on
  // the sample arm, exactly `null` (via the `Null` getter) on the error arm.
  const sampleArm = arms.find((r): r is ReplySample => r.sample !== null)
  const errorArm = arms.find((r): r is ReplyError => r.sample === null)
  expect(sampleArm).toBeDefined()
  expect(errorArm).toBeDefined()

  // Sample arm: payload marshalled, replier id present.
  expect(sampleArm!.sample.payload.toString()).toBe('answer')
  expect(sampleArm!.replierId).not.toBeNull()
  // Error arm: its own payload + encoding getters.
  expect(errorArm!.payload.toString()).toBe('boom')
  expect(errorArm!.encoding).toBe('text/plain')
})

test('a received Query exposes typed metadata getters', async () => {
  await using session = await Session.open()
  const k = ke('query-getters')
  using queryable = await session.declareQueryable(k)

  const replies = await session.get(`${k}?arg=1`, { ...bounded, payload: 'q-payload' })

  const query = await recvWithin(() => queryable.recv())
  expect(query).not.toBeNull()
  expect(query!.keyExpr).toBeInstanceOf(KeyExpr)
  expect(query!.keyExpr.toString()).toBe(k)
  expect(query!.selector).toContain('arg=1')
  expect(query!.parameters).toContain('arg=1')
  expect(query!.payload?.toString()).toBe('q-payload')
  await query!.reply(k, 'ok')

  // Drain so the query channel closes promptly rather than at the get timeout.
  await collectReplies(replies, 1)
})

test('entity ids marshal as { zid, eid }', async () => {
  await using session = await Session.open()
  using publisher = await session.declarePublisher(ke('entity-id'))

  const id = publisher.id
  expect(typeof id.zid).toBe('string')
  expect(id.zid.length).toBeGreaterThan(0)
  expect(typeof id.eid).toBe('number')
})

test('Publisher config getters echo the declared options', async () => {
  await using session = await Session.open()
  const k = ke('pub-config')
  using publisher = await session.declarePublisher(k, {
    encoding: 'text/plain',
    congestionControl: 'Block',
    priority: 'DataHigh',
    reliability: 'Reliable',
  })

  expect(publisher.keyExpr).toBeInstanceOf(KeyExpr)
  expect(publisher.keyExpr.toString()).toBe(k)
  expect(publisher.encoding).toBe('text/plain')
  expect(publisher.congestionControl).toBe('Block')
  expect(publisher.priority).toBe('DataHigh')
  expect(publisher.reliability).toBe('Reliable')
})

test('Querier config getters echo the declared options', async () => {
  await using session = await Session.open()
  const k = ke('querier-config')
  using querier = await session.declareQuerier(k, {
    congestionControl: 'Block',
    priority: 'DataHigh',
  })

  expect(querier.keyExpr.toString()).toBe(k)
  expect(querier.congestionControl).toBe('Block')
  expect(querier.priority).toBe('DataHigh')
  expect(typeof querier.id.zid).toBe('string')
})

test('SessionInfo getters marshal zid and peer/router id arrays', async () => {
  await using session = await Session.open()
  const info = session.info()

  expect(await info.zid()).toBe(session.zid)
  expect(Array.isArray(await info.routersZid())).toBe(true)
  expect(Array.isArray(await info.peersZid())).toBe(true)
})
