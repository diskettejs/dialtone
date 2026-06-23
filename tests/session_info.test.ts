import { expect, test } from 'vitest'
import { Config, Session } from '../index.js'
import { loopbackConfig } from './loopback.js'

const TCP_ENDPOINT = 'tcp/127.0.0.1:17453'

// A peer that listens on a fixed TCP endpoint and does not scout.
function listenConfig(): Config {
  const config = Config.default()
  config.insertJson5('mode', '"peer"')
  config.insertJson5('scouting/multicast/enabled', 'false')
  config.insertJson5('listen/endpoints', `["${TCP_ENDPOINT}"]`)
  config.insertJson5('connect/endpoints', '[]')
  return config
}

// A peer that connects to the fixed TCP endpoint and does not scout.
function connectConfig(): Config {
  const config = Config.default()
  config.insertJson5('mode', '"peer"')
  config.insertJson5('scouting/multicast/enabled', 'false')
  config.insertJson5('listen/endpoints', '[]')
  config.insertJson5('connect/endpoints', `["${TCP_ENDPOINT}"]`)
  return config
}

async function waitFor<T>(
  poll: () => Promise<T[]>,
  timeoutMs = 5000,
  intervalMs = 50,
): Promise<T[]> {
  const deadline = Date.now() + timeoutMs
  for (;;) {
    const items = await poll()
    if (items.length > 0 || Date.now() > deadline) return items
    await new Promise((resolve) => setTimeout(resolve, intervalMs))
  }
}

test('session.info() exposes the session zid', async () => {
  const session = await Session.open(loopbackConfig())
  const info = session.info()
  expect(await info.zid()).toBe(session.zid)
  await session.close()
})

test('an isolated session has no transports or links', async () => {
  const session = await Session.open(loopbackConfig())
  const info = session.info()
  expect(await info.transports()).toEqual([])
  expect(await info.links()).toEqual([])
  expect(await info.locators()).toEqual([])
  await session.close()
})

test('event listeners declare, expose a handler, and undeclare', async () => {
  const session = await Session.open(loopbackConfig())
  const info = session.info()

  const transportEvents = await info.transportEventsListener()
  expect(transportEvents.handler).toBeDefined()
  await transportEvents.undeclare()
  await transportEvents.undeclare() // second call is a no-op

  const linkEvents = await info.linkEventsListener()
  expect(linkEvents.handler).toBeDefined()
  await linkEvents.undeclare()

  await session.close()
})

test('event listeners are disposable via `await using`', async () => {
  await using session = await Session.open(loopbackConfig())
  const info = session.info()

  // Both `SessionInfo` listeners patch `Symbol.asyncDispose`, so `await using`
  // releases them at scope exit — parity with the other declared listeners.
  {
    await using transportEvents = await info.transportEventsListener()
    await using linkEvents = await info.linkEventsListener()
    expect(Symbol.asyncDispose in transportEvents).toBe(true)
    expect(Symbol.asyncDispose in linkEvents).toBe(true)
  }
})

test('connected peers see each other as a transport with links', async () => {
  const listener = await Session.open(listenConfig())
  const connector = await Session.open(connectConfig())

  try {
    const transports = await waitFor(() => connector.info().transports())
    expect(transports.length).toBeGreaterThan(0)

    const transport = transports[0]!
    expect(transport.zid).toBe(listener.zid)
    expect(['Peer', 'Router', 'Client']).toContain(transport.whatami)
    expect(typeof transport.isQos).toBe('boolean')
    expect(typeof transport.isMulticast).toBe('boolean')

    const links = await waitFor(() => connector.info().links())
    expect(links.length).toBeGreaterThan(0)

    const link = links[0]!
    expect(link.zid).toBe(listener.zid)
    expect(typeof link.src.asStr).toBe('string')
    expect(typeof link.dst.asStr).toBe('string')
    expect(typeof link.mtu).toBe('number')
    expect(typeof link.isStreamed).toBe('boolean')
    expect(Array.isArray(link.interfaces)).toBe(true)
  } finally {
    await connector.close()
    await listener.close()
  }
})

test('transportEventsListener with history replays the existing transport', async () => {
  const listener = await Session.open(listenConfig())
  const connector = await Session.open(connectConfig())

  try {
    // Wait until the transport is established before subscribing with history.
    await waitFor(() => connector.info().transports())

    const events = await connector.info().transportEventsListener({
      history: true,
    })
    if ('recvTimeout' in events.handler) {
      const event = await events.handler.recvTimeout(3000)
      expect(event).not.toBeNull()
      expect(event!.kind).toBe('Put')
      expect(event!.transport.zid).toBe(listener.zid)
    }
    await events.undeclare()
  } finally {
    await connector.close()
    await listener.close()
  }
})
