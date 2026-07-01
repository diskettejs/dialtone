import { describe, expectTypeOf, test } from 'vitest'

import { zd, type Bytes, type ZSchema } from '../index.js'

describe('leaf builders carry their scalar type', () => {
  test('numbers, bigints, and the special leaves', () => {
    expectTypeOf(zd.bool()).toEqualTypeOf<ZSchema<boolean>>()
    expectTypeOf(zd.string()).toEqualTypeOf<ZSchema<string>>()
    expectTypeOf(zd.bytes()).toEqualTypeOf<ZSchema<Uint8Array>>()
    expectTypeOf(zd.i32()).toEqualTypeOf<ZSchema<number>>()
    expectTypeOf(zd.f64()).toEqualTypeOf<ZSchema<number>>()
    expectTypeOf(zd.i64()).toEqualTypeOf<ZSchema<bigint>>()
    expectTypeOf(zd.u64()).toEqualTypeOf<ZSchema<bigint>>()
  })
})

describe('composite builders infer element/field types', () => {
  test('array and set', () => {
    expectTypeOf(zd.array(zd.string())).toEqualTypeOf<ZSchema<string[]>>()
    expectTypeOf(zd.set(zd.i32())).toEqualTypeOf<ZSchema<Set<number>>>()
  })

  test('map', () => {
    expectTypeOf(zd.map(zd.string(), zd.i32())).toEqualTypeOf<ZSchema<Map<string, number>>>()
  })

  test('tuple preserves positional heterogeneity', () => {
    expectTypeOf(zd.tuple([zd.string(), zd.i32(), zd.bool()])).toEqualTypeOf<
      ZSchema<[string, number, boolean]>
    >()
  })

  test('object maps field schemas to a record', () => {
    const point = zd.object({ x: zd.f64(), y: zd.f64() })
    expectTypeOf(point).toEqualTypeOf<ZSchema<{ x: number; y: number }>>()
  })
})

describe('serialize / deserialize narrow to the schema’s type', () => {
  test('methods on the schema', () => {
    const point = zd.object({ x: zd.f64(), y: zd.f64() })
    expectTypeOf(point.serialize).parameter(0).toEqualTypeOf<{ x: number; y: number }>()
    expectTypeOf(point.serialize({ x: 1, y: 2 })).toEqualTypeOf<Bytes>()
    expectTypeOf(point.deserialize).returns.toEqualTypeOf<{ x: number; y: number }>()
  })

  test('free functions', () => {
    const schema = zd.array(zd.i32())
    expectTypeOf(zd.serialize(schema, [1, 2, 3])).toEqualTypeOf<Bytes>()
    expectTypeOf(zd.deserialize(schema, {} as Bytes)).toEqualTypeOf<number[]>()
  })
})
