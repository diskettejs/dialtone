import { parseArgs } from 'node:util'
import { Session } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

async function main() {
  const { values } = parseArgs({
    options: { ...commonOptions, 'no-express': { type: 'boolean' } },
  })
  const express = !values['no-express']

  await using session = await Session.open(configFromArgs(values))
  await using publisher = await session.declarePublisher('test/pong', {
    congestionControl: 'Block',
    express,
  })
  await using subscriber = await session.declareSubscriber('test/ping')

  // Echo every ping payload straight back on test/pong (no callbacks in Dialtone).
  // @ts-expect-error
  for await (const sample of subscriber.stream()) {
    await publisher.put(sample.payload.toBytes())
  }
}

await main()
