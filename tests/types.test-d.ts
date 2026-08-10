import { describe, expectTypeOf, test } from 'vitest'

import type {
  EntityGlobalId,
  Reply,
  ReplyError,
  ReplyIter,
  ReplyResult,
  ReplyResultError,
  ReplyResultSample,
  Sample,
  SampleIter,
} from '../index.js'

declare const result: ReplyResult
declare const reply: Reply
declare const samples: SampleIter
declare const replies: ReplyIter

describe('ReplyResult union discriminated by nullability', () => {
  test('union shape', () => {
    expectTypeOf<ReplyResult>().toEqualTypeOf<ReplyResultSample | ReplyResultError>()
    expectTypeOf(reply.id).toEqualTypeOf<EntityGlobalId | null>()
    expectTypeOf(reply.result).toEqualTypeOf<ReplyResult>()
  })

  test('ReplyResult narrows on `sample`/`error` nullability', () => {
    if (result.sample) {
      expectTypeOf(result).toEqualTypeOf<ReplyResultSample>()
      expectTypeOf(result.sample).toEqualTypeOf<Sample>()
      expectTypeOf(result.error).toEqualTypeOf<null>()
    } else {
      expectTypeOf(result).toEqualTypeOf<ReplyResultError>()
      expectTypeOf(result.error).toEqualTypeOf<ReplyError>()
      expectTypeOf(result.sample).toEqualTypeOf<null>()
    }
  })
})

describe('generated iterators satisfy the async iterable protocol', () => {
  test('iterator classes are assignable to AsyncIterable of their yield type', () => {
    expectTypeOf(samples).toExtend<AsyncIterable<Sample>>()
    expectTypeOf(replies).toExtend<AsyncIterable<Reply>>()
  })

  test('for await binds the yield type', async () => {
    for await (const sample of samples) {
      expectTypeOf(sample).toEqualTypeOf<Sample>()
    }
  })
})
