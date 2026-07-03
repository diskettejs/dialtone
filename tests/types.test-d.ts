import { describe, expectTypeOf, test } from 'vitest'

import type {
  EntityGlobalId,
  Reply,
  ReplyError,
  ReplyResult,
  ReplyResultError,
  ReplyResultSample,
  Sample,
} from '../index.js'

declare const result: ReplyResult
declare const reply: Reply

describe('ReplyResult union discriminated by nullability', () => {
  test('union shape', () => {
    expectTypeOf<ReplyResult>().toEqualTypeOf<ReplyResultSample | ReplyResultError>()
    expectTypeOf(reply.replierId).toEqualTypeOf<EntityGlobalId | null>()
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
