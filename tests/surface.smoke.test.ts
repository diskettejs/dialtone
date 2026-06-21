// Concern: every option field marshals into its Zenoh builder.
//
// These declare with the *full* option bag set and assert only that it succeeds.
// That is deliberate: the runtime *effect* of most options (caching, detection,
// recovery, consolidation) is Zenoh's behavior and belongs nowhere in this suite —
// but "the field is threaded through `into_zenoh` without panicking or being
// dropped" is squarely the binding's job, and this is where it's covered. Each new
// option added to the surface should gain a line here.
import { expect, test } from 'vitest'
import { Config, Session } from '../index.js'
import { bounded, ke } from './helpers.ts'

test('Config factory methods construct instances', () => {
  expect(Config.default()).toBeInstanceOf(Config)
  expect(Config.fromJson5('{}')).toBeInstanceOf(Config)
})

test('Session.open accepts an explicit Config', async () => {
  await using session = await Session.open(Config.default())
  expect(session.isClosed).toBe(false)
})

test('declarePublisher accepts the full base + advanced option set', async () => {
  await using session = await Session.open()
  const k = ke('pub-smoke')
  const publisher = await session.declarePublisher(k, {
    encoding: 'text/plain',
    congestionControl: 'Block',
    priority: 'DataHigh',
    express: true,
    reliability: 'Reliable',
    allowedDestination: 'Any',
    cache: {
      maxSamples: 5,
      repliesConfig: { priority: 'DataHigh', congestionControl: 'Block', express: true },
    },
    sampleMissDetection: { heartbeat: { periodMs: 500 } },
    publisherDetection: true,
    publisherDetectionMetadata: 'meta/pub',
  })
  expect(publisher.keyExpr.toString()).toBe(k)
  publisher.undeclare()
})

test('declareSubscriber accepts the full advanced option set', async () => {
  await using session = await Session.open()
  const k = ke('sub-smoke')
  const subscriber = await session.declareSubscriber(k, {
    allowedOrigin: 'Any',
    handler: { kind: 'Fifo', capacity: 16 },
    history: { detectLatePublishers: true, maxSamples: 5, maxAgeSecs: 60 },
    recovery: { heartbeat: true },
    subscriberDetection: true,
    subscriberDetectionMetadata: 'meta/sub',
    queryTimeoutMs: 5_000,
  })
  expect(subscriber.keyExpr.toString()).toBe(k)
  subscriber.undeclare()
})

test('declareQueryable accepts its option set', async () => {
  await using session = await Session.open()
  const k = ke('queryable-smoke')
  const queryable = await session.declareQueryable(k, {
    complete: true,
    allowedOrigin: 'Any',
    handler: { kind: 'Ring', capacity: 8 },
  })
  expect(queryable.keyExpr.toString()).toBe(k)
  queryable.undeclare()
})

test('declareQuerier accepts the full option set', async () => {
  await using session = await Session.open()
  const k = ke('querier-smoke')
  const querier = await session.declareQuerier(k, {
    target: 'All',
    consolidation: 'None',
    congestionControl: 'Block',
    priority: 'DataHigh',
    express: true,
    allowedDestination: 'Any',
    timeout: 1_000,
    acceptReplies: 'Any',
  })
  expect(querier.keyExpr.toString()).toBe(k)
  querier.undeclare()
})

test('session.get accepts the full option set', async () => {
  await using session = await Session.open()
  const replies = await session.get(ke('get-smoke'), {
    ...bounded,
    payload: 'q',
    encoding: 'text/plain',
    attachment: 'a',
    congestionControl: 'Block',
    priority: 'DataHigh',
    express: true,
    target: 'All',
    allowedDestination: 'Any',
    acceptReplies: 'Any',
    sourceInfo: { sourceId: { zid: session.zid, eid: 0 }, sourceSn: 0 },
    handler: { kind: 'Fifo', capacity: 16 },
  })
  expect(replies).toBeDefined()
})
