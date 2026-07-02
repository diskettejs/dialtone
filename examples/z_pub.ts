import { setTimeout as sleep } from 'node:timers/promises'
import { parseArgs } from 'node:util'
import { Session, type PublisherPutOptions } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

async function main() {
  const { values } = parseArgs({
    options: {
      ...commonOptions,
      key: { type: 'string', short: 'k', default: 'demo/example/zenoh-rs-pub' },
      payload: { type: 'string', short: 'p', default: 'Pub from Rust!' },
      attach: { type: 'string', short: 'a' },
      'add-matching-listener': { type: 'boolean' },
    },
  })

  console.log('Opening session...')
  await using session = await Session.open(configFromArgs(values))

  console.log(`Declaring Publisher on '${values.key}'...`)
  await using publisher = await session.declarePublisher(values.key)

  if (values['add-matching-listener']) {
    const listener = await publisher.matchingListener()
    // No callbacks in Dialtone: consume the listener as an async iterator.
    void (async () => {
      // @ts-expect-error
      for await (const status of listener.stream()) {
        console.log(
          status.matching
            ? 'Publisher has matching subscribers.'
            : 'Publisher has NO MORE matching subscribers.',
        )
      }
    })()
  }

  console.log('Press CTRL-C to quit...')
  for (let idx = 0; ; idx++) {
    await sleep(1000)
    const buf = `[${idx.toString().padStart(4)}] ${values.payload}`
    console.log(`Putting Data ('${values.key}': '${buf}')...`)

    // Optionally set the encoding metadata and add an attachment.
    // Refer to z_bytes.ts to see how to serialize different types of message.
    const options: PublisherPutOptions = { encoding: 'text/plain' }
    if (values.attach !== undefined) options.attachment = values.attach
    await publisher.put(buf, options)
  }
}

await main()
