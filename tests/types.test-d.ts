import { describe, expectTypeOf, test } from 'vitest'

import type {
  EntityGlobalId,
  Reply,
  ReplyError,
  ReplyErrored,
  ReplyResult,
  ReplySample,
  Sample,
} from '../index.js'

declare const result: ReplyResult
declare const reply: Reply

describe('ReplyResult is a two-arm discriminated union', () => {
  test('union shape', () => {
    expectTypeOf<ReplyResult>().toEqualTypeOf<ReplySample | ReplyErrored>()
  })

  test('ReplySample arm is tagged and carries the sample', () => {
    expectTypeOf<ReplySample['isSample']>().toEqualTypeOf<true>()
    expectTypeOf<ReplySample['isError']>().toEqualTypeOf<false>()
    expectTypeOf<ReplySample['sample']>().toEqualTypeOf<Sample>()
  })

  test('ReplyErrored arm is tagged and carries the error', () => {
    expectTypeOf<ReplyErrored['isSample']>().toEqualTypeOf<false>()
    expectTypeOf<ReplyErrored['isError']>().toEqualTypeOf<true>()
    expectTypeOf<ReplyErrored['error']>().toEqualTypeOf<ReplyError>()
  })
})

describe('ReplyResult narrows on either boolean tag without `!`', () => {
  test('checking `isSample` narrows both arms', () => {
    if (result.isSample) {
      expectTypeOf(result).toEqualTypeOf<ReplySample>()
      expectTypeOf(result.sample).toEqualTypeOf<Sample>()
    } else {
      expectTypeOf(result).toEqualTypeOf<ReplyErrored>()
      expectTypeOf(result.error).toEqualTypeOf<ReplyError>()
    }
  })

  test('checking `isError` narrows both arms', () => {
    if (result.isError) {
      expectTypeOf(result).toEqualTypeOf<ReplyErrored>()
      expectTypeOf(result.error).toEqualTypeOf<ReplyError>()
    } else {
      expectTypeOf(result).toEqualTypeOf<ReplySample>()
      expectTypeOf(result.sample).toEqualTypeOf<Sample>()
    }
  })
})

describe('Reply class exposes raw accessors plus the result union', () => {
  test('getters', () => {
    expectTypeOf(reply.sample).toEqualTypeOf<Sample | null>()
    expectTypeOf(reply.error).toEqualTypeOf<ReplyError | null>()
    expectTypeOf(reply.replierId).toEqualTypeOf<EntityGlobalId | null>()
    expectTypeOf(reply.isSample).toEqualTypeOf<boolean>()
    expectTypeOf(reply.isError).toEqualTypeOf<boolean>()
    expectTypeOf(reply.result).toEqualTypeOf<ReplyResult>()
  })

  test('reply.result narrows like ReplyResult', () => {
    const r = reply.result
    if (r.isSample) {
      expectTypeOf(r.sample).toEqualTypeOf<Sample>()
    } else {
      expectTypeOf(r.error).toEqualTypeOf<ReplyError>()
    }
  })
})
