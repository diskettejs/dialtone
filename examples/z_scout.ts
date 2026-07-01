import { Config, Scout, WhatAmIMatcher } from '../index.js'

async function main() {
  console.log('Scouting...')
  const what = WhatAmIMatcher.empty().router().peer()
  const scout = await Scout.scout(what, Config.default())

  // Scout for one second, then stop — which ends the stream below.
  const timer = setTimeout(() => scout.stop(), 1000)
  try {
    for await (const hello of scout.stream()) {
      const locators = hello.locators().map((l) => l.toString()).join(', ')
      console.log(`Hello { zid: ${hello.zid}, whatami: ${hello.whatami}, locators: [${locators}] }`)
    }
  } finally {
    clearTimeout(timer)
  }
}

await main()
