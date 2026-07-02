import { parseArgs } from 'node:util'
import { Session } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

async function main() {
  const { values } = parseArgs({
    options: {
      ...commonOptions,
      key: { type: 'string', short: 'k', default: 'demo/example/**' },
      forward: { type: 'string', short: 'f', default: 'demo/forward' },
    },
  })

  console.log('Opening session...')
  await using session = await Session.open(configFromArgs(values))

  console.log(`Declaring Subscriber on '${values.key}'...`)
  await using subscriber = await session.declareSubscriber(values.key)

  console.log(`Declaring Publisher on '${values.forward}'...`)
  await using publisher = await session.declarePublisher(values.forward)

  console.log(`Forwarding data from '${values.key}' to '${values.forward}'...`)
  // Dialtone has no SubscriberForward helper; forward via the async iterator.
  // @ts-expect-error
  for await (const sample of subscriber.stream()) {
    if (sample.kind === 'Delete') {
      await publisher.delete()
    } else {
      await publisher.put(sample.payload.toBytes(), { encoding: sample.encoding.toString() })
    }
  }
}

await main()
