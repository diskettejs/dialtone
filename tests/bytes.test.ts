import { describe, expect, test } from 'vitest'

import { Bytes } from '../index.js'

const utf8 = (value: string): Uint8Array => new TextEncoder().encode(value)

describe('Bytes', () => {
  describe('from()', () => {
    test('accepts a string and round-trips through tryToString', () => {
      expect(Bytes.from('hello').tryToString()).toBe('hello')
      expect(Bytes.from('héllo 🌍').tryToString()).toBe('héllo 🌍')
    })

    test('stores a string as its UTF-8 encoding, readable via toBytes', () => {
      expect(Bytes.from('héllo 🌍').toBytes()).toEqual(utf8('héllo 🌍'))
    })

    test('accepts raw bytes and round-trips them through toBytes', () => {
      const raw = new Uint8Array([0, 1, 2, 254, 255])
      expect(Bytes.from(raw).toBytes()).toEqual(raw)
    })

    test('decodes valid UTF-8 bytes via tryToString', () => {
      expect(Bytes.from(utf8('héllo 🌍')).tryToString()).toBe('héllo 🌍')
    })
  })

  describe('toBytes()', () => {
    test('is non-consuming — repeatable', () => {
      const bytes = Bytes.from(new Uint8Array([1, 2, 3]))
      expect(bytes.toBytes()).toEqual(new Uint8Array([1, 2, 3]))
      expect(bytes.toBytes()).toEqual(new Uint8Array([1, 2, 3]))
    })
  })

  describe('tryToString()', () => {
    test('returns null for a lone 0xFF (invalid UTF-8)', () => {
      expect(Bytes.from(new Uint8Array([0xff])).tryToString()).toBeNull()
    })

    test('returns null for a valid prefix followed by a bare continuation byte', () => {
      // "hi" then 0x80, a continuation byte with no leading byte
      expect(Bytes.from(new Uint8Array([0x68, 0x69, 0x80])).tryToString()).toBeNull()
    })

    test('never throws regardless of contents', () => {
      expect(() => Bytes.from(new Uint8Array([0xff, 0xfe, 0xfd])).tryToString()).not.toThrow()
    })

    test('decodes an empty payload to the empty string', () => {
      expect(new Bytes().tryToString()).toBe('')
      expect(Bytes.from('').tryToString()).toBe('')
      expect(Bytes.from(new Uint8Array([])).tryToString()).toBe('')
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
