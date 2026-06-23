import { expect, test } from 'vitest'
import { Session } from '../index.js'
import { loopbackConfig } from './loopback.js'

test('session exposes zid and isClosed', async () => {
  const session = await Session.open(loopbackConfig())
  expect(typeof session.zid).toBe('string')
  expect(session.zid.length).toBeGreaterThan(0)
  expect(session.isClosed).toBe(false)
  await session.close()
  expect(session.isClosed).toBe(true)
})
