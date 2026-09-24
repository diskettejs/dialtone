import { describe, expect, test } from 'vitest'
import { defineConfig, type ZenohConfig } from '../index.js'

describe('defineConfig — valid configs', () => {
  test('no config / empty config falls back to Zenoh defaults', () => {
    expect(() => defineConfig()).not.toThrow()
    expect(() => defineConfig({})).not.toThrow()
  })

  test('comprehensive object exercising every precisely-typed section', () => {
    const config: ZenohConfig = {
      id: '1a2b3c4d', // ZenohId: lowercase hex, <= 16 bytes
      mode: 'peer',
      metadata: { name: 'demo-node', location: 'lab' },
      namespace: 'demo/site-a', // non-wildcard key expression
      queries_default_timeout: 10000,
      connect: {
        timeout_ms: 5000,
        endpoints: ['tcp/127.0.0.1:7447', 'tcp/127.0.0.1:7448'],
        exit_on_failure: false,
      },
      listen: {
        // per-mode (ModeDependent) form: applies different values by node mode
        timeout_ms: { peer: 3000, router: 6000 },
        endpoints: ['tcp/0.0.0.0:0'],
        exit_on_failure: { peer: false },
      },
      open: {
        return_conditions: { connect_scouted: true, declares: true },
      },
      scouting: {
        timeout: 3000,
        delay: 200,
        multicast: {
          enabled: true,
          address: '224.0.0.224:7446',
          interface: 'auto',
          ttl: 1,
          autoconnect: ['router', 'peer'],
          listen: true,
        },
        gossip: {
          enabled: true,
          multihop: false,
          target: { router: ['router', 'peer'], peer: ['router'] },
          autoconnect: ['router', 'peer'],
        },
      },
      timestamping: {
        enabled: { router: true, peer: false },
        drop_future_timestamp: false,
      },
      aggregation: {
        subscribers: ['demo/**'],
        publishers: ['demo/**'],
      },
    }

    expect(() => defineConfig(config)).not.toThrow()
  })

  test('common: client connecting to a router', () => {
    const config: ZenohConfig = {
      mode: 'client',
      connect: { endpoints: ['tcp/192.168.1.10:7447'] },
    }
    expect(() => defineConfig(config)).not.toThrow()
  })

  test('common: peer listening + multicast scouting', () => {
    const config: ZenohConfig = {
      mode: 'peer',
      listen: { endpoints: ['tcp/0.0.0.0:7447', 'udp/0.0.0.0:7447'] },
      scouting: { multicast: { enabled: true } },
    }
    expect(() => defineConfig(config)).not.toThrow()
  })

  test('mode-dependent values: unique vs per-mode forms are interchangeable', () => {
    const unique: ZenohConfig = {
      connect: { endpoints: ['tcp/127.0.0.1:7447'] },
    }
    const perMode: ZenohConfig = {
      connect: { endpoints: { router: ['tcp/127.0.0.1:7447'], peer: [] } },
    }
    expect(() => defineConfig(unique)).not.toThrow()
    expect(() => defineConfig(perMode)).not.toThrow()
  })
})

describe('defineConfig — invalid configs throw', () => {
  test('unknown top-level key (deny_unknown_fields)', () => {
    expect(() => defineConfig({ verbosity: 'high' } as unknown as ZenohConfig)).toThrow()
  })

  test('unknown nested key', () => {
    expect(() =>
      defineConfig({ scouting: { multicast: { bogus: true } } } as unknown as ZenohConfig),
    ).toThrow()
  })

  test('invalid mode (not router | peer | client)', () => {
    expect(() => defineConfig({ mode: 'supernode' } as unknown as ZenohConfig)).toThrow()
  })

  test('wrong value type (queries_default_timeout must be a number)', () => {
    expect(() =>
      defineConfig({ queries_default_timeout: 'soon' } as unknown as ZenohConfig),
    ).toThrow()
  })
})
