import { describe, expect, test } from 'vitest'

import { Bytes, Deserializer, Serializer } from '../index.js'

type Write<T> = (s: Serializer, value: T) => void
type Read<T> = (d: Deserializer) => T
type RoundtripCase<T> = [name: string, value: T, write: Write<T>, read: Read<T>]

function roundtrip<T>(write: (s: Serializer) => void, read: Read<T>): T {
  const s = new Serializer()
  write(s)
  const d = new Deserializer(s.finish())
  const value = read(d)
  expect(d.done()).toBe(true)
  return value
}

function bytesOf(write: (s: Serializer) => void): number[] {
  const s = new Serializer()
  write(s)
  return Array.from(s.finish().toBytes())
}

function roundtripEach<T>(cases: RoundtripCase<T>[]): void {
  test.each(cases)('%s', (_name, value, write, read) => {
    expect(roundtrip((s) => write(s, value), read)).toEqual(value)
  })
}

describe('scalars (number)', () => {
  roundtripEach<number>([
    ['i8 / min', -128, (s, v) => s.i8(v), (d) => d.i8()],
    ['i8 / max', 127, (s, v) => s.i8(v), (d) => d.i8()],
    ['i16 / min', -32768, (s, v) => s.i16(v), (d) => d.i16()],
    ['i16 / max', 32767, (s, v) => s.i16(v), (d) => d.i16()],
    ['i32 / min', -2147483648, (s, v) => s.i32(v), (d) => d.i32()],
    ['i32 / max', 2147483647, (s, v) => s.i32(v), (d) => d.i32()],
    ['u8 / min', 0, (s, v) => s.u8(v), (d) => d.u8()],
    ['u8 / max', 255, (s, v) => s.u8(v), (d) => d.u8()],
    ['u16 / max', 65535, (s, v) => s.u16(v), (d) => d.u16()],
    ['u32 / max', 4294967295, (s, v) => s.u32(v), (d) => d.u32()],
    ['f32 / 0.5', 0.5, (s, v) => s.f32(v), (d) => d.f32()],
    ['f32 / -2.5', -2.5, (s, v) => s.f32(v), (d) => d.f32()],
    ['f32 / integer', 1234, (s, v) => s.f32(v), (d) => d.f32()],
    ['f64 / pi', 3.141592653589793, (s, v) => s.f64(v), (d) => d.f64()],
    ['f64 / -1.5', -1.5, (s, v) => s.f64(v), (d) => d.f64()],
    ['f64 / MAX_SAFE_INTEGER', Number.MAX_SAFE_INTEGER, (s, v) => s.f64(v), (d) => d.f64()],
  ])
})

describe('scalars (bigint)', () => {
  roundtripEach<bigint>([
    ['i64 / min', -(2n ** 63n), (s, v) => s.i64(v), (d) => d.i64()],
    ['i64 / max', 2n ** 63n - 1n, (s, v) => s.i64(v), (d) => d.i64()],
    ['i128 / min', -(2n ** 127n), (s, v) => s.i128(v), (d) => d.i128()],
    ['i128 / max', 2n ** 127n - 1n, (s, v) => s.i128(v), (d) => d.i128()],
    ['u64 / min', 0n, (s, v) => s.u64(v), (d) => d.u64()],
    ['u64 / max', 2n ** 64n - 1n, (s, v) => s.u64(v), (d) => d.u64()],
    ['u128 / min', 0n, (s, v) => s.u128(v), (d) => d.u128()],
    ['u128 / max', 2n ** 128n - 1n, (s, v) => s.u128(v), (d) => d.u128()],
    ['varInt / min', 0n, (s, v) => s.varInt(v), (d) => d.varInt()],
    ['varInt / max', 2n ** 64n - 1n, (s, v) => s.varInt(v), (d) => d.varInt()],
  ])
})

describe('bool / string', () => {
  roundtripEach<boolean>([
    ['bool / true', true, (s, v) => s.bool(v), (d) => d.bool()],
    ['bool / false', false, (s, v) => s.bool(v), (d) => d.bool()],
  ])

  roundtripEach<string>([
    ['string / empty', '', (s, v) => s.string(v), (d) => d.string()],
    ['string / ascii', 'serialization', (s, v) => s.string(v), (d) => d.string()],
    ['string / multibyte', 'héllo 🌐', (s, v) => s.string(v), (d) => d.string()],
  ])
})

describe('typed arrays', () => {
  roundtripEach<Uint8Array>([
    [
      'uint8Array',
      new Uint8Array([0, 1, 128, 255]),
      (s, v) => s.uint8Array(v),
      (d) => d.uint8Array(),
    ],
  ])
  roundtripEach<Int8Array>([
    [
      'int8Array',
      new Int8Array([-128, -1, 0, 127]),
      (s, v) => s.int8Array(v),
      (d) => d.int8Array(),
    ],
  ])
  roundtripEach<Uint16Array>([
    [
      'uint16Array',
      new Uint16Array([0, 256, 65535]),
      (s, v) => s.uint16Array(v),
      (d) => d.uint16Array(),
    ],
  ])
  roundtripEach<Int16Array>([
    [
      'int16Array',
      new Int16Array([-32768, 0, 32767]),
      (s, v) => s.int16Array(v),
      (d) => d.int16Array(),
    ],
  ])
  roundtripEach<Uint32Array>([
    [
      'uint32Array',
      new Uint32Array([0, 65536, 4294967295]),
      (s, v) => s.uint32Array(v),
      (d) => d.uint32Array(),
    ],
  ])
  roundtripEach<Int32Array>([
    [
      'int32Array',
      new Int32Array([-2147483648, 0, 2147483647]),
      (s, v) => s.int32Array(v),
      (d) => d.int32Array(),
    ],
  ])
  roundtripEach<Float32Array>([
    [
      'float32Array',
      new Float32Array([0.5, -2.5, 1234]),
      (s, v) => s.float32Array(v),
      (d) => d.float32Array(),
    ],
  ])
  roundtripEach<Float64Array>([
    [
      'float64Array',
      new Float64Array([1.5, -2.5, 3.141592653589793]),
      (s, v) => s.float64Array(v),
      (d) => d.float64Array(),
    ],
    [
      'float64Array / empty',
      new Float64Array([]),
      (s, v) => s.float64Array(v),
      (d) => d.float64Array(),
    ],
  ])
  roundtripEach<BigInt64Array>([
    [
      'bigInt64Array',
      new BigInt64Array([-(2n ** 63n), 0n, 2n ** 63n - 1n]),
      (s, v) => s.bigInt64Array(v),
      (d) => d.bigInt64Array(),
    ],
  ])
  roundtripEach<BigUint64Array>([
    [
      'bigUint64Array',
      new BigUint64Array([0n, 42n, 2n ** 64n - 1n]),
      (s, v) => s.bigUint64Array(v),
      (d) => d.bigUint64Array(),
    ],
  ])
})

describe('string[] / boolean[]', () => {
  roundtripEach<string[]>([
    ['stringArray', ['', 'a', 'hello', '🌐'], (s, v) => s.stringArray(v), (d) => d.stringArray()],
  ])
  roundtripEach<boolean[]>([
    ['boolArray', [true, false, true, true, false], (s, v) => s.boolArray(v), (d) => d.boolArray()],
  ])
})

describe('nested / composition', () => {
  test('bytes (nested ZBytes)', () => {
    const input = Bytes.fromBytes(new Uint8Array([1, 2, 3, 4, 5]))
    const out = roundtrip(
      (s) => s.bytes(input),
      (d) => d.bytes(),
    )
    expect(Array.from(out.toBytes())).toEqual([1, 2, 3, 4, 5])
  })

  test('mixed sequence is read back in order (tuple-equivalent)', () => {
    const s = new Serializer()
    s.u32(42)
    s.string('answer')
    s.f64(3.5)
    s.bool(true)

    const d = new Deserializer(s.finish())
    expect(d.u32()).toBe(42)
    expect(d.string()).toBe('answer')
    expect(d.f64()).toBe(3.5)
    expect(d.bool()).toBe(true)
    expect(d.done()).toBe(true)
  })

  test('Map<string, u32> composed by hand', () => {
    const map = new Map<string, number>([
      ['a', 1],
      ['b', 2],
      ['c', 3],
    ])

    const s = new Serializer()
    s.varInt(BigInt(map.size))
    for (const [k, v] of map) {
      s.string(k)
      s.u32(v)
    }

    const d = new Deserializer(s.finish())
    const out = new Map<string, number>()
    const len = d.varInt()
    for (let i = 0n; i < len; i++) {
      out.set(d.string(), d.u32())
    }

    expect(out).toEqual(map)
    expect(d.done()).toBe(true)
  })
})

describe('wire format (cross-checked against zenoh-ext Rust tests)', () => {
  test.each<[name: string, write: (s: Serializer) => void, expected: number[]]>([
    ['i32 = 1234566', (s) => s.i32(1234566), [134, 214, 18, 0]],
    ['i32 = -49245', (s) => s.i32(-49245), [163, 63, 255, 255]],
    ['string = "test"', (s) => s.string('test'), [4, 116, 101, 115, 116]],
    [
      'tuple (u16, f32, string)',
      (s) => {
        s.u16(500)
        s.f32(1234)
        s.string('test')
      },
      [244, 1, 0, 64, 154, 68, 4, 116, 101, 115, 116],
    ],
    [
      'Vec<i64>',
      (s) => s.bigInt64Array(new BigInt64Array([-100n, 500n, 100000n, -20000000n])),
      [
        4, 156, 255, 255, 255, 255, 255, 255, 255, 244, 1, 0, 0, 0, 0, 0, 0, 160, 134, 1, 0, 0, 0,
        0, 0, 0, 211, 206, 254, 255, 255, 255, 255,
      ],
    ],
    [
      'Vec<(string, i16)> composed by hand',
      (s) => {
        s.varInt(2n)
        s.string('s1')
        s.i16(10)
        s.string('s2')
        s.i16(-10000)
      },
      [2, 2, 115, 49, 10, 0, 2, 115, 50, 240, 216],
    ],
  ])('%s', (_name, write, expected) => {
    expect(bytesOf(write)).toEqual(expected)
  })
})

describe('lifecycle & errors', () => {
  test('done() reflects remaining data', () => {
    const s = new Serializer()
    s.u8(1)
    s.u8(2)

    const d = new Deserializer(s.finish())
    expect(d.done()).toBe(false)
    d.u8()
    expect(d.done()).toBe(false)
    d.u8()
    expect(d.done()).toBe(true)
  })

  test.each<[name: string, op: () => void]>([
    [
      'finish() called twice',
      () => {
        const s = new Serializer()
        s.u32(1)
        s.finish()
        s.finish()
      },
    ],
    [
      'serialize after finish()',
      () => {
        const s = new Serializer()
        s.finish()
        s.u32(1)
      },
    ],
    [
      'reading past end of buffer',
      () => {
        const s = new Serializer()
        s.u8(7)
        const d = new Deserializer(s.finish())
        d.u8()
        d.u8()
      },
    ],
    [
      'reading more bytes than available',
      () => {
        const s = new Serializer()
        s.u8(1)
        const d = new Deserializer(s.finish())
        d.u32()
      },
    ],
  ])('throws: %s', (_name, op) => {
    expect(op).toThrow()
  })
})
