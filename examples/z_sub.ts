import { parseArgs } from 'node:util'
import { Session } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

async function main() {
  const { values } = parseArgs({
    options: {
      ...commonOptions,
      key: { type: 'string', short: 'k', default: 'demo/example/**' },
    },
  })

  console.log('Opening session...')
  await using session = await Session.open(configFromArgs(values))

  console.log(`Declaring Subscriber on '${values.key}'...`)
  await using subscriber = await session.declareSubscriber(values.key)

  console.log('Press CTRL-C to quit...')
  for await (const sample of subscriber.stream()) {
    let line = `>> [Subscriber] Received ${sample.kind} ('${sample.keyExpr}': '${sample.payload.tryToString() ?? ''}')`
    if (sample.attachment) line += ` (${sample.attachment.tryToString() ?? ''})`
    console.log(line)
  }
  // while (true) {
  //   const sample = await subscriber.recv()
  //   // Refer to z_bytes.ts to see how to deserialize different types of message.
  //   let line = `>> [Subscriber] Received ${sample.kind} ('${sample.keyExpr}': '${sample.payload.tryToString() ?? ''}')`
  //   if (sample.attachment) line += ` (${sample.attachment.tryToString() ?? ''})`
  //   console.log(line)
  // }
}

await main()
