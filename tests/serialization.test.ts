import { describe, expect, test } from 'vitest'

import { zd, type ZSchema } from '../index.js'

const bytesOf = <T>(schema: ZSchema<T>, value: T): number[] =>
  Array.from(schema.serialize(value).toBytes())

const roundtrip = <T>(schema: ZSchema<T>, value: T): T =>
  schema.deserialize(schema.serialize(value))

describe('byte-identity with zenoh-ext', () => {
  test('i32', () => {
    expect(bytesOf(zd.i32(), 1234566)).toEqual([134, 214, 18, 0])
    expect(bytesOf(zd.i32(), -49245)).toEqual([163, 63, 255, 255])
  })

  test('string', () => {
    expect(bytesOf(zd.string(), 'test')).toEqual([4, 116, 101, 115, 116])
  })

  test('tuple (u16, f32, string)', () => {
    const schema = zd.tuple([zd.u16(), zd.f32(), zd.string()])
    expect(bytesOf(schema, [500, 1234.0, 'test'])).toEqual([
      244, 1, 0, 64, 154, 68, 4, 116, 101, 115, 116,
    ])
  })

  test('array of (string, i16) tuples', () => {
    const schema = zd.array(zd.tuple([zd.string(), zd.i16()]))
    expect(
      bytesOf(schema, [
        ['s1', 10],
        ['s2', -10000],
      ]),
    ).toEqual([2, 2, 115, 49, 10, 0, 2, 115, 50, 240, 216])
  })
})

describe('leaf round-trips', () => {
  test('bool', () => {
    expect(roundtrip(zd.bool(), true)).toBe(true)
    expect(roundtrip(zd.bool(), false)).toBe(false)
  })

  test('string / bytes', () => {
    expect(roundtrip(zd.string(), 'héllo 🌍')).toBe('héllo 🌍')
    const bytes = new Uint8Array([0, 1, 2, 254, 255])
    expect(roundtrip(zd.bytes(), bytes)).toEqual(bytes)
  })

  test('fixed-width integers at their bounds', () => {
    expect(roundtrip(zd.i8(), -128)).toBe(-128)
    expect(roundtrip(zd.i8(), 127)).toBe(127)
    expect(roundtrip(zd.u8(), 255)).toBe(255)
    expect(roundtrip(zd.i16(), -32768)).toBe(-32768)
    expect(roundtrip(zd.u16(), 65535)).toBe(65535)
    expect(roundtrip(zd.i32(), -2147483648)).toBe(-2147483648)
    expect(roundtrip(zd.u32(), 4294967295)).toBe(4294967295)
  })

  test('64-bit integers are bigint both ways', () => {
    expect(roundtrip(zd.i64(), -9223372036854775808n)).toBe(-9223372036854775808n)
    expect(roundtrip(zd.u64(), 18446744073709551615n)).toBe(18446744073709551615n)
  })

  test('floats — f32 exact when representable, f64 identity', () => {
    expect(roundtrip(zd.f32(), 1.5)).toBe(1.5)
    expect(roundtrip(zd.f32(), -0.25)).toBe(-0.25)
    expect(roundtrip(zd.f64(), 3.141592653589793)).toBe(3.141592653589793)
  })
})

describe('composite round-trips', () => {
  test('array', () => {
    expect(roundtrip(zd.array(zd.i32()), [1, -2, 3])).toEqual([1, -2, 3])
    expect(roundtrip(zd.array(zd.string()), [])).toEqual([])
  })

  test('tuple (heterogeneous, fixed arity)', () => {
    const schema = zd.tuple([zd.string(), zd.i32(), zd.bool()])
    expect(roundtrip(schema, ['x', 7, true])).toEqual(['x', 7, true])
  })

  test('object (positional, declaration order)', () => {
    const point = zd.object({ x: zd.f64(), y: zd.f64() })
    expect(roundtrip(point, { x: 1.5, y: -2 })).toEqual({ x: 1.5, y: -2 })
  })

  test('object ignores extra keys (width subtyping)', () => {
    const point = zd.object({ x: zd.f64(), y: zd.f64() })
    const extra = { x: 1, y: 2, z: 3 } as { x: number; y: number }
    expect(roundtrip(point, extra)).toEqual({ x: 1, y: 2 })
  })

  test('map (real JS Map, arbitrary key schema)', () => {
    const schema = zd.map(zd.string(), zd.i32())
    const value = new Map([
      ['a', 1],
      ['b', 2],
    ])
    expect(roundtrip(schema, value)).toEqual(value)
  })

  test('set (real JS Set)', () => {
    const schema = zd.set(zd.i32())
    const value = new Set([1, 2, 3])
    expect(roundtrip(schema, value)).toEqual(value)
  })

  test('nested — array of objects containing a map', () => {
    const schema = zd.array(zd.object({ id: zd.u32(), tags: zd.map(zd.string(), zd.bool()) }))
    const value = [
      { id: 1, tags: new Map([['live', true]]) },
      { id: 2, tags: new Map<string, boolean>() },
    ]
    expect(roundtrip(schema, value)).toEqual(value)
  })
})

describe('errors surface, not silently coerced', () => {
  test('out-of-range value for a narrow width throws (checked narrow)', () => {
    expect(() => zd.u8().serialize(300)).toThrow()
    expect(() => zd.i8().serialize(200)).toThrow()
  })

  test('64-bit leaves require bigint, not number', () => {
    expect(() => zd.i64().serialize(5 as unknown as bigint)).toThrow()
  })

  test('missing object field throws via the leaf’s coercion of undefined', () => {
    const schema = zd.object({ x: zd.f64() })
    expect(() => schema.serialize({} as { x: number })).toThrow()
  })

  test('trailing bytes after the schema is satisfied throws', () => {
    const payload = zd.tuple([zd.i32(), zd.i32()]).serialize([1, 2])
    expect(() => zd.i32().deserialize(payload)).toThrow()
  })

  test('truncated payload throws', () => {
    const payload = zd.i32().serialize(1)
    expect(() => zd.tuple([zd.i32(), zd.i32()]).deserialize(payload)).toThrow()
  })
})
