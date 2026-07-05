import { describe, expect, test } from 'vitest'

import { Bytes } from '../index.js'

const utf8 = (value: string): Uint8Array => new TextEncoder().encode(value)

describe('Bytes', () => {
  describe('fromString()', () => {
    test('round-trips through tryToString', () => {
      expect(Bytes.fromString('hello').tryToString()).toBe('hello')
      expect(Bytes.fromString('héllo 🌍').tryToString()).toBe('héllo 🌍')
    })

    test('stores the UTF-8 encoding, readable via toBytes', () => {
      expect(Bytes.fromString('héllo 🌍').toBytes()).toEqual(utf8('héllo 🌍'))
    })
  })

  describe('fromBytes()', () => {
    test('round-trips arbitrary bytes through toBytes', () => {
      const raw = new Uint8Array([0, 1, 2, 254, 255])
      expect(Bytes.fromBytes(raw).toBytes()).toEqual(raw)
    })

    test('decodes valid UTF-8 via tryToString', () => {
      expect(Bytes.fromBytes(utf8('héllo 🌍')).tryToString()).toBe('héllo 🌍')
    })
  })

  describe('toBytes()', () => {
    test('is non-consuming — repeatable', () => {
      const bytes = Bytes.fromBytes(new Uint8Array([1, 2, 3]))
      expect(bytes.toBytes()).toEqual(new Uint8Array([1, 2, 3]))
      expect(bytes.toBytes()).toEqual(new Uint8Array([1, 2, 3]))
    })
  })

  describe('tryToString()', () => {
    test('returns null for a lone 0xFF (invalid UTF-8)', () => {
      expect(Bytes.fromBytes(new Uint8Array([0xff])).tryToString()).toBeNull()
    })

    test('returns null for a valid prefix followed by a bare continuation byte', () => {
      // "hi" then 0x80, a continuation byte with no leading byte
      expect(Bytes.fromBytes(new Uint8Array([0x68, 0x69, 0x80])).tryToString()).toBeNull()
    })

    test('never throws regardless of contents', () => {
      expect(() => Bytes.fromBytes(new Uint8Array([0xff, 0xfe, 0xfd])).tryToString()).not.toThrow()
    })

    test('decodes an empty payload to the empty string', () => {
      expect(new Bytes().tryToString()).toBe('')
      expect(Bytes.fromString('').tryToString()).toBe('')
      expect(Bytes.fromBytes(new Uint8Array([])).tryToString()).toBe('')
    })
  })

  describe('len', () => {
    test('reports byte length, not code-point count', () => {
      expect(Bytes.fromString('abc').len).toBe(3)
      expect(Bytes.fromString('é').len).toBe(2) // one code point, two UTF-8 bytes
      expect(Bytes.fromString('🌍').len).toBe(4) // one code point, four UTF-8 bytes
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
      expect(Bytes.fromString('x').isEmpty).toBe(false)
      expect(Bytes.fromBytes(new Uint8Array([0])).isEmpty).toBe(false)
    })
  })
})
