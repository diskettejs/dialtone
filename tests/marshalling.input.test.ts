// Concern: inputs crossing JS → Rust/Zenoh.
//
// Every value a caller passes in — payloads, key expressions, QoS enums, the
// nested Timestamp/SourceInfo objects, the whole option bag — must arrive on the
// other side of the FFI boundary intact. We observe that by putting with the
// values set and reading them back off the delivered Sample: a same-session
// round-trip is just the vehicle, the *marshalling* is what's asserted.
import { expect, test } from 'vitest'
import { KeyExpr, Session } from '../index.js'
import { ke, recvWithin } from './helpers.ts'

test('put marshals every option onto the delivered sample', async () => {
  await using session = await Session.open()
  const k = ke('put-opts')
  using sub = await session.declareSubscriber(k)
  const ts = session.newTimestamp()

  await session.put(k, Buffer.from([1, 2, 3]), {
    encoding: 'application/octet-stream',
    attachment: 'meta',
    congestionControl: 'Block',
    priority: 'DataHigh',
    express: true,
    allowedDestination: 'Any',
    timestamp: ts,
    sourceInfo: { sourceId: { zid: session.zid, eid: 0 }, sourceSn: 7 },
  })

  const sample = await recvWithin(() => sub.recv())
  expect(sample).not.toBeNull()

  // Payload: a Buffer is copied across byte-for-byte.
  expect([...sample!.payload]).toEqual([1, 2, 3])
  // QoS / encoding / attachment: set on the put, read back off the sample.
  expect(sample!.encoding).toBe('application/octet-stream')
  expect(sample!.attachment?.toString()).toBe('meta')
  expect(sample!.congestionControl).toBe('Block')
  expect(sample!.priority).toBe('DataHigh')
  expect(sample!.express).toBe(true)
  expect(sample!.kind).toBe('Put')
  // Timestamp round-trips through Rust (BigInt time + hex id → NTP64/ZenohId and
  // back) — exercises both Timestamp::to_zenoh and Timestamp::from_zenoh.
  expect(typeof ts.time).toBe('bigint')
  expect(sample!.timestamp?.time).toBe(ts.time)
  expect(sample!.timestamp?.id).toBe(ts.id)
  // SourceInfo: a nested object marshals out and back, sequence number intact.
  expect(sample!.sourceInfo?.sourceSn).toBe(7)
  expect(sample!.sourceInfo?.sourceId.zid).toBe(session.zid)
})

test('a string payload is marshalled as UTF-8 bytes', async () => {
  await using session = await Session.open()
  const k = ke('utf8')
  using sub = await session.declareSubscriber(k)

  await session.put(k, 'héllo') // multibyte: 6 bytes, 5 chars

  const sample = await recvWithin(() => sub.recv())
  expect(sample!.payload.toString('utf8')).toBe('héllo')
  expect(sample!.payload.length).toBe(6)
})

test('delete marshals its options and arrives as a Delete sample', async () => {
  await using session = await Session.open()
  const k = ke('delete')
  using sub = await session.declareSubscriber(k)
  const ts = session.newTimestamp()

  await session.delete(k, {
    attachment: 'bye',
    priority: 'DataHigh',
    timestamp: ts,
  })

  const sample = await recvWithin(() => sub.recv())
  expect(sample!.kind).toBe('Delete')
  expect(sample!.attachment?.toString()).toBe('bye')
  expect(sample!.priority).toBe('DataHigh')
  expect(sample!.timestamp?.time).toBe(ts.time)
})

test('a KeyExpr instance is accepted wherever a string key expression is', async () => {
  await using session = await Session.open()
  const k = ke('keyexpr-arg')
  // Both the subscribe and the put take a KeyExpr instance rather than a string;
  // KeyExprArg converts either form at the boundary.
  using sub = await session.declareSubscriber(new KeyExpr(k))
  await session.put(new KeyExpr(k), 'hi')

  const sample = await recvWithin(() => sub.recv())
  expect(sample!.payload.toString()).toBe('hi')
})

test('KeyExpr comparison methods accept both string and KeyExpr arguments', () => {
  const ex = new KeyExpr('demo/**')
  // The binding concern is the `string | KeyExpr` argument polymorphism and the
  // boolean return marshalling — not the matching algebra itself, which is Zenoh's.
  expect(typeof ex.intersects('demo/a')).toBe('boolean')
  expect(typeof ex.intersects(new KeyExpr('demo/a'))).toBe('boolean')
  expect(typeof ex.includes('demo/a')).toBe('boolean')
  expect(ex.equals(new KeyExpr('demo/**'))).toBe(true)
  // join returns a KeyExpr instance, not a string.
  expect(ex.join('child')).toBeInstanceOf(KeyExpr)
})
