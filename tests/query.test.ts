import { afterAll, beforeAll, describe, expect, test } from 'vitest'

import { defineConfig, Parameters, Selector, Session } from '../index.js'

let session: Session

beforeAll(async () => {
  session = await Session.open(defineConfig())
})

afterAll(async () => {
  await session.close()
})

// `consolidation: 'None'` is a test fixture, not an assertion target: it lets a single
// reply be observed synchronously. The default Auto consolidation buffers replies until
// the query is finalized, which would couple seam tests to drop()/GC timing.

describe('Queryable', () => {
  describe('declaration', () => {
    test('marshals id and keyExpr', async () => {
      using queryable = await session.declareQueryable('test/queryable')

      expect(queryable.keyExpr.toString()).toBe('test/queryable')
      expect(typeof queryable.id.zid).toBe('string')
      expect(typeof queryable.id.eid).toBe('number')
    })
  })

  describe('undeclare()', () => {
    test('rejects recv() after undeclare', async () => {
      const queryable = await session.declareQueryable('test/queryable/undeclare')
      queryable.undeclare()

      await expect(queryable.recv()).rejects.toThrow(/has already been consumed./i)
    })
  })
})

describe('Query', () => {
  describe('request getters', () => {
    test('marshal payload, encoding, attachment, and parameters', async () => {
      const key = 'test/query/request'
      using queryable = await session.declareQueryable(key)
      await session.get(`${key}?unit=celsius`, {
        consolidation: 'None',
        payload: 'req',
        encoding: 'text/plain',
        attachment: 'meta',
      })

      const query = await queryable.recv()
      expect(query.payload?.tryToString()).toBe('req')
      expect(query.encoding?.toString()).toBe('text/plain')
      expect(query.attachment?.tryToString()).toBe('meta')
      expect(query.parameters.get('unit')).toBe('celsius')
    })

    test('marshal an absent payload as null', async () => {
      const key = 'test/query/no-payload'
      using queryable = await session.declareQueryable(key)
      await session.get(key, { consolidation: 'None' })

      const query = await queryable.recv()
      expect(query.payload).toBeNull()
    })
  })

  describe('reply()', () => {
    test('yields the sample arm and applies ReplyOptions', async () => {
      const key = 'test/query/reply'
      using queryable = await session.declareQueryable(key)
      const replies = await session.get(key, { consolidation: 'None' })

      const query = await queryable.recv()
      await query.reply(key, 'pong', { encoding: 'text/plain', attachment: 'meta' })

      const reply = await replies.recv()
      expect(reply.replierId).not.toBeNull()
      expect(reply.result.error).toBeNull()
      expect(reply.result.sample?.payload.tryToString()).toBe('pong')
      expect(reply.result.sample?.encoding.toString()).toBe('text/plain')
      expect(reply.result.sample?.attachment?.tryToString()).toBe('meta')
    })
  })

  describe('replyErr()', () => {
    test('yields the error arm and applies ReplyErrOptions', async () => {
      const key = 'test/query/reply-err'
      using queryable = await session.declareQueryable(key)
      const replies = await session.get(key, { consolidation: 'None' })

      const query = await queryable.recv()
      await query.replyErr('boom', { encoding: 'text/plain' })

      const reply = await replies.recv()
      expect(reply.result.sample).toBeNull()
      expect(reply.result.error?.payload.tryToString()).toBe('boom')
      expect(reply.result.error?.encoding.toString()).toBe('text/plain')
    })
  })

  describe('replyDel()', () => {
    test('delivers a Delete-kind reply', async () => {
      const key = 'test/query/reply-del'
      using queryable = await session.declareQueryable(key)
      const replies = await session.get(key, { consolidation: 'None' })

      const query = await queryable.recv()
      await query.replyDel(key)

      const reply = await replies.recv()
      expect(reply.result.sample?.kind).toBe('Delete')
    })
  })

  describe('drop()', () => {
    test('rejects reply() after drop', async () => {
      const key = 'test/query/drop'
      using queryable = await session.declareQueryable(key)
      await session.get(key, { consolidation: 'None' })

      const query = await queryable.recv()
      query.drop()

      await expect(query.reply(key, 'late')).rejects.toThrow(/has already been consumed/i)
    })
  })
})

describe('Querier', () => {
  describe('declaration', () => {
    test('applies priority, congestionControl, and acceptReplies from QuerierOptions', async () => {
      const key = 'test/querier'
      using querier = await session.declareQuerier(key, {
        priority: 'DataHigh',
        congestionControl: 'Block',
        acceptReplies: 'Any',
      })

      expect(querier.keyExpr.toString()).toBe(key)
      expect(typeof querier.id.zid).toBe('string')
      expect(querier.priority).toBe('DataHigh')
      expect(querier.congestionControl).toBe('Block')
      expect(querier.acceptReplies).toBe('Any')
    })
  })

  describe('get()', () => {
    test('returns a ReplyHandler and plumbs QuerierGetOptions to the query', async () => {
      const key = 'test/querier/get'
      using queryable = await session.declareQueryable(key)
      using querier = await session.declareQuerier(key)

      const replies = await querier.get({ payload: 'ping', parameters: { n: '1' } })
      expect(typeof replies.recv).toBe('function')

      const query = await queryable.recv()
      expect(query.payload?.tryToString()).toBe('ping')
      expect(query.parameters.get('n')).toBe('1')
    })
  })

  describe('undeclare()', () => {
    test('rejects get() after undeclare', async () => {
      const querier = await session.declareQuerier('test/querier/undeclare')
      querier.undeclare()

      await expect(querier.get()).rejects.toThrow(/has already been consumed/i)
    })
  })
})

describe('Session.get()', () => {
  describe('selector resolution', () => {
    test('accepts a Selector instance', async () => {
      const key = 'test/select/instance'
      using queryable = await session.declareQueryable(key)
      await session.get(new Selector(key, 'mode=fast'), { consolidation: 'None' })

      const query = await queryable.recv()
      expect(query.selector.keyExpr.toString()).toBe(key)
      expect(query.parameters.get('mode')).toBe('fast')
    })

    test('overrides selector parameters with the parameters option', async () => {
      const key = 'test/select/override'
      using queryable = await session.declareQueryable(key)
      await session.get(`${key}?ignored=1`, {
        consolidation: 'None',
        parameters: { used: '1' },
      })

      const query = await queryable.recv()
      expect(query.parameters.get('used')).toBe('1')
      expect(query.parameters.containsKey('ignored')).toBe(false)
    })
  })
})
