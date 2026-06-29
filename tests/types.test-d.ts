import { describe, expectTypeOf, test } from 'vitest'

import type {
  Bytes,
  Encoding,
  EntityGlobalId,
  Reliability,
  Replies,
  Reply,
  ReplyError,
  ReplyErrored,
  ReplyResult,
  ReplySample,
  Sample,
} from '../index.js'

declare const replies: Replies
declare const reply: Reply
declare const result: ReplyResult

describe('Replies channel methods yield ReplyResult', () => {
  test('recv / tryRecv / drain / stream advertise the union, not the Reply class', () => {
    expectTypeOf(replies.recv()).toEqualTypeOf<Promise<ReplyResult>>()
    expectTypeOf(replies.tryRecv()).toEqualTypeOf<ReplyResult | null>()
    expectTypeOf(replies.drain()).toEqualTypeOf<ReplyResult[]>()
    expectTypeOf(replies.stream()).toEqualTypeOf<ReadableStream<ReplyResult>>()
  })
})

describe('ReplyResult is a two-arm discriminated union', () => {
  test('union shape', () => {
    expectTypeOf<ReplyResult>().toEqualTypeOf<ReplySample | ReplyErrored>()
  })

  test('ReplySample arm carries the sample and an absent error', () => {
    expectTypeOf<ReplySample['sample']>().toEqualTypeOf<Sample>()
    expectTypeOf<ReplySample['error']>().toEqualTypeOf<null | undefined>()
    expectTypeOf<ReplySample['replierId']>().toEqualTypeOf<EntityGlobalId | null>()
  })

  test('ReplyErrored arm carries the error and an absent sample', () => {
    expectTypeOf<ReplyErrored['error']>().toEqualTypeOf<ReplyError>()
    expectTypeOf<ReplyErrored['sample']>().toEqualTypeOf<null | undefined>()
    expectTypeOf<ReplyErrored['replierId']>().toEqualTypeOf<EntityGlobalId | null>()
  })
})

describe('ReplyResult narrows on presence without `!`', () => {
  test('checking `sample` narrows both arms', () => {
    if (result.sample) {
      expectTypeOf(result).toEqualTypeOf<ReplySample>()
      expectTypeOf(result.sample).toEqualTypeOf<Sample>()
      expectTypeOf(result.sample.attachment).toEqualTypeOf<Bytes | null>()
    } else {
      expectTypeOf(result).toEqualTypeOf<ReplyErrored>()
      expectTypeOf(result.error).toEqualTypeOf<ReplyError>()
    }
  })

  test('checking `error` narrows both arms', () => {
    if (result.error) {
      expectTypeOf(result).toEqualTypeOf<ReplyErrored>()
      expectTypeOf(result.error).toEqualTypeOf<ReplyError>()
      expectTypeOf(result.error.encoding).toEqualTypeOf<Encoding>()
    } else {
      expectTypeOf(result).toEqualTypeOf<ReplySample>()
      expectTypeOf(result.sample).toEqualTypeOf<Sample>()
      expectTypeOf(result.sample.reliability).toEqualTypeOf<Reliability>()
    }
  })
})

describe('Reply class exposes nullable accessors', () => {
  test('getters', () => {
    expectTypeOf(reply.sample).toEqualTypeOf<Sample | null>()
    expectTypeOf(reply.error).toEqualTypeOf<ReplyError | null>()
    expectTypeOf(reply.replierId).toEqualTypeOf<EntityGlobalId | null>()
  })
})
