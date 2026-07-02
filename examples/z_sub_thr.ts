import { parseArgs } from 'node:util'
import { Session } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

class Stats {
  roundSize: number
  roundCount = 0
  finishedRounds = 0
  roundStart = performance.now()
  globalStart: number | undefined

  constructor(roundSize: number) {
    this.roundSize = roundSize
  }

  increment() {
    if (this.roundCount === 0) {
      this.roundStart = performance.now()
      this.globalStart ??= this.roundStart
      this.roundCount++
    } else if (this.roundCount < this.roundSize) {
      this.roundCount++
    } else {
      this.printRound()
      this.finishedRounds++
      this.roundCount = 0
    }
  }

  printRound() {
    const elapsed = (performance.now() - this.roundStart) / 1000
    console.log(`${this.roundSize / elapsed} msg/s`)
  }

  printFinal() {
    if (this.globalStart === undefined) return
    const elapsed = (performance.now() - this.globalStart) / 1000
    const total = this.roundSize * this.finishedRounds + this.roundCount
    console.log(`Received ${total} messages over ${elapsed.toFixed(2)}s: ${total / elapsed}msg/s`)
  }
}

async function main() {
  const { values } = parseArgs({
    options: {
      ...commonOptions,
      samples: { type: 'string', short: 's', default: '10' },
      number: { type: 'string', short: 'n', default: '100000' },
    },
  })
  const samples = Number(values.samples)
  const stats = new Stats(Number(values.number))

  await using session = await Session.open(configFromArgs(values))
  await using subscriber = await session.declareSubscriber('test/thr')

  console.log('Press CTRL-C to quit...')
  // @ts-expect-error
  for await (const _sample of subscriber.stream()) {
    stats.increment()
    if (stats.finishedRounds >= samples) break
  }
  stats.printFinal()
}

await main()
