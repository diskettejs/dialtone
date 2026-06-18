import { describe, expect, it } from 'vitest'
import { Config, Session } from '../index.js'

const settle = (ms = 200) => new Promise((r) => setTimeout(r, ms))

describe('config', () => {
  it('open() with a Config instance and with a JSON5 string', async () => {
    const fromInstance = await Session.open(Config.default())
    await fromInstance.close()

    const fromString = await Session.open('{ mode: "peer" }')
    await fromString.close()
  }, 10_000)

  it('insertJson5 mutates the config and toString reflects it', () => {
    const config = Config.default()
    config.insertJson5('scouting/multicast/enabled', 'false')

    const dumped = JSON.parse(config.toString())
    expect(dumped.scouting.multicast.enabled).toBe(false)
  })

  it('two sessions link over explicit TCP endpoints with multicast off', async () => {
    const endpoint = 'tcp/127.0.0.1:17447'

    const listener = Config.default()
    listener.insertJson5('scouting/multicast/enabled', 'false')
    listener.insertJson5('listen/endpoints', `["${endpoint}"]`)

    const connector = Config.default()
    connector.insertJson5('scouting/multicast/enabled', 'false')
    connector.insertJson5('connect/endpoints', `["${endpoint}"]`)

    // Two sessions in this process stand in for two processes: they can only find
    // each other through the explicit TCP link, since multicast discovery is off.
    const a = await Session.open(listener)
    const b = await Session.open(connector)

    const sub = await a.declareSubscriber('scan/events')
    await settle(500) // let the TCP session establish + subscription propagate

    await b.put('scan/events', 'job-done')
    const sample = await sub.receive()

    expect(sample).not.toBeNull()
    expect(new TextDecoder().decode(sample!.payload)).toBe('job-done')

    await sub.undeclare()
    await b.close()
    await a.close()
  }, 15_000)
})
