import { parseArgs } from 'node:util'
import { Session } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

async function main() {
  const { values, positionals } = parseArgs({
    allowPositionals: true,
    options: {
      ...commonOptions,
      'no-express': { type: 'boolean' },
      warmup: { type: 'string', short: 'w', default: '1' },
      samples: { type: 'string', short: 'n', default: '100' },
    },
  })

  const size = Number(positionals[0])
  if (!Number.isInteger(size) || size <= 0) {
    throw new Error('usage: node z_ping.ts <payload_size> [options]')
  }
  const n = Number(values.samples)
  const warmupMs = Number(values.warmup) * 1000
  const express = !values['no-express']

  await using session = await Session.open(configFromArgs(values))
  await using sub = await session.declareSubscriber('test/pong')
  await using publisher = await session.declarePublisher('test/ping', {
    congestionControl: 'Block',
    express,
  })

  const data = Uint8Array.from({ length: size }, (_, i) => i % 10)

  console.log(`Warming up for ${warmupMs / 1000}s...`)
  const warmupEnd = performance.now() + warmupMs
  while (performance.now() < warmupEnd) {
    await publisher.put(data)
    await sub.recv()
  }

  const samples: number[] = []
  for (let i = 0; i < n; i++) {
    const writeTime = performance.now()
    await publisher.put(data)
    await sub.recv()
    samples.push((performance.now() - writeTime) * 1000) // microseconds
  }

  samples.forEach((rtt, i) => {
    console.log(`${size} bytes: seq=${i} rtt=${rtt.toFixed(0)}µs lat=${(rtt / 2).toFixed(0)}µs`)
  })
}

await main()
