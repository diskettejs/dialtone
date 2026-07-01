import { parseArgs } from 'node:util'
import { Session, type Priority, type PublisherOptions } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

async function main() {
  const { values, positionals } = parseArgs({
    allowPositionals: true,
    options: {
      ...commonOptions,
      express: { type: 'boolean' },
      priority: { type: 'string', short: 'p' },
      print: { type: 'boolean', short: 't' },
      number: { type: 'string', short: 'n', default: '100000' },
    },
  })

  const size = Number(positionals[0])
  if (!Number.isInteger(size) || size <= 0) {
    throw new Error('usage: node z_pub_thr.ts <payload_size> [options]')
  }
  const number = Number(values.number)
  const data = Uint8Array.from({ length: size }, (_, i) => i % 10)

  await using session = await Session.open(configFromArgs(values))

  const options: PublisherOptions = { congestionControl: 'Block', express: values.express }
  if (values.priority) options.priority = values.priority as Priority
  await using publisher = await session.declarePublisher('test/thr', options)

  console.log('Press CTRL-C to quit...')
  let count = 0
  let start = performance.now()
  for (;;) {
    await publisher.put(data)
    if (values.print) {
      if (count < number) {
        count++
      } else {
        console.log(`${count / ((performance.now() - start) / 1000)} msg/s`)
        count = 0
        start = performance.now()
      }
    }
  }
}

await main()
