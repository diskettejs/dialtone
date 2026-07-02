import { parseArgs } from 'node:util'
import { KeyExpr, Session, type Sample } from '../index.js'
import { bytesToString, commonOptions, configFromArgs } from './common.ts'

async function main() {
  const { values } = parseArgs({
    options: {
      ...commonOptions,
      key: { type: 'string', short: 'k', default: 'demo/example/**' },
      complete: { type: 'boolean' },
    },
  })

  const stored = new Map<string, Sample>()

  console.log('Opening session...')
  await using session = await Session.open(configFromArgs(values))

  console.log(`Declaring Subscriber on '${values.key}'...`)
  await using subscriber = await session.declareSubscriber(values.key)

  console.log(`Declaring Queryable on '${values.key}'...`)
  await using queryable = await session.declareQueryable(values.key, { complete: values.complete })

  console.log('Press CTRL-C to quit...')

  // Rust drives both receivers with `select!`; here each async iterator runs
  // its own loop concurrently, sharing the `stored` map.
  const store = (async () => {
    // @ts-expect-error
    for await (const sample of subscriber.stream()) {
      const key = sample.keyExpr.toString()
      console.log(
        `>> [Subscriber] Received ${sample.kind} ('${key}': '${bytesToString(sample.payload)}')`,
      )
      if (sample.kind === 'Delete') stored.delete(key)
      else stored.set(key, sample)
    }
  })()

  const serve = (async () => {
    // @ts-expect-error
    for await (const query of queryable.stream()) {
      console.log(`>> [Queryable] Received Query '${query.keyExpr}'`)
      for (const [key, sample] of stored) {
        if (query.keyExpr.intersects(new KeyExpr(key))) {
          await query.reply(sample.keyExpr, sample.payload.toBytes())
        }
      }
    }
  })()

  await Promise.all([store, serve])
}

await main()
