import { describe, expect, test } from 'vitest'

import { Bytes } from '../index.js'

const utf8 = (value: string): Uint8Array => new TextEncoder().encode(value)

describe('Bytes', () => {
  describe('from()', () => {
    test('accepts a string and round-trips through toString', () => {
      expect(Bytes.from('hello').toString()).toBe('hello')
      expect(Bytes.from('héllo 🌍').toString()).toBe('héllo 🌍')
    })

    test('stores a string as its UTF-8 encoding, readable via toBytes', () => {
      expect(Bytes.from('héllo 🌍').toBytes()).toEqual(utf8('héllo 🌍'))
    })

    test('accepts raw bytes and round-trips them through toBytes', () => {
      const raw = new Uint8Array([0, 1, 2, 254, 255])
      expect(Bytes.from(raw).toBytes()).toEqual(raw)
    })

    test('decodes valid UTF-8 bytes via toString', () => {
      expect(Bytes.from(utf8('héllo 🌍')).toString()).toBe('héllo 🌍')
    })
  })

  describe('toBytes()', () => {
    test('is non-consuming — repeatable', () => {
      const bytes = Bytes.from(new Uint8Array([1, 2, 3]))
      expect(bytes.toBytes()).toEqual(new Uint8Array([1, 2, 3]))
      expect(bytes.toBytes()).toEqual(new Uint8Array([1, 2, 3]))
    })
  })

  describe('toString()', () => {
    test('replaces a lone 0xFF with U+FFFD', () => {
      expect(Bytes.from(new Uint8Array([0xff])).toString()).toBe('�')
    })

    test('keeps the valid prefix and replaces a bare continuation byte', () => {
      // "hi" then 0x80, a continuation byte with no leading byte
      expect(Bytes.from(new Uint8Array([0x68, 0x69, 0x80])).toString()).toBe('hi�')
    })

    test('emits one U+FFFD per maximal invalid subsequence', () => {
      const raw = new Uint8Array([0xff, 0xfe, 0xfd])
      expect(Bytes.from(raw).toString()).toBe('���')
      expect(Bytes.from(raw).toString()).toBe(new TextDecoder().decode(raw))
    })

    test('never throws regardless of contents', () => {
      expect(() => Bytes.from(new Uint8Array([0xff, 0xfe, 0xfd])).toString()).not.toThrow()
    })

    test('decodes an empty payload to the empty string', () => {
      expect(new Bytes().toString()).toBe('')
      expect(Bytes.from('').toString()).toBe('')
      expect(Bytes.from(new Uint8Array([])).toString()).toBe('')
    })

    test('is used by string coercion', () => {
      expect(`${Bytes.from('hello')}`).toBe('hello')
    })
  })

  describe('len', () => {
    test('reports byte length, not code-point count', () => {
      expect(Bytes.from('abc').len).toBe(3)
      expect(Bytes.from('é').len).toBe(2) // one code point, two UTF-8 bytes
      expect(Bytes.from('🌍').len).toBe(4) // one code point, four UTF-8 bytes
    })
  })

  describe('isEmpty', () => {
    test('is true for a freshly constructed Bytes', () => {
      const bytes = new Bytes()
      expect(bytes.isEmpty).toBe(true)
      expect(bytes.len).toBe(0)
      expect(bytes.toBytes()).toEqual(new Uint8Array([]))
    })

    test('is false for non-empty payloads', () => {
      expect(Bytes.from('x').isEmpty).toBe(false)
      expect(Bytes.from(new Uint8Array([0])).isEmpty).toBe(false)
    })
  })
})
