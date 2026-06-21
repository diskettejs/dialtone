// Concern: the FFI plumbing is connected end-to-end for surfaces with no
// marshalling round-trip to cover them — the liveliness subsystem and advanced
// publisher detection.
//
// These DELIBERATELY lean on Zenoh's own behavior (a token surfacing, a detection
// token appearing) because that is the only signal that the binding method is
// wired at all. Each test is the *sole* positive proof for its entry point; keep
// the assertions to "a thing arrived, shaped correctly" and resist asserting
// Zenoh's semantics (appear-then-vanish lifecycles, ordering, completeness).
import { expect, test } from 'vitest'
import { Session } from '../index.js'
import { ke, recvWithin } from './helpers.ts'

test('a liveliness token surfaces through a liveliness subscriber', async () => {
  await using session = await Session.open()
  const liveliness = session.liveliness()
  const root = ke('liveliness')

  using subscriber = await liveliness.declareSubscriber(`${root}/**`)
  const token = await liveliness.declareToken(`${root}/token`)

  // Wiring check: the token's appearance is delivered as a sample for its key.
  // (That appearance is a `Put` is Zenoh's semantics — we assert the wiring.)
  const appeared = await recvWithin(() => subscriber.recv())
  expect(appeared).not.toBeNull()
  expect(appeared!.keyExpr.toString()).toBe(`${root}/token`)
  expect(appeared!.kind).toBe('Put')

  token.undeclare()
})

test('detectPublishers observes a detection-enabled publisher', async () => {
  await using session = await Session.open()
  const k = ke('detect')

  using subscriber = await session.declareSubscriber(k)
  const detected = await subscriber.detectPublishers()

  // Declared after detectPublishers, so it is seen as a live change.
  const publisher = await session.declarePublisher(k, { publisherDetection: true })

  const appeared = await recvWithin(() => detected.recv())
  expect(appeared).not.toBeNull()
  expect(appeared!.kind).toBe('Put')

  publisher.undeclare()
  detected.undeclare()
})
