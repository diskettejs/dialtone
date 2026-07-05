import {
  Config,
  LinkEventsListener,
  LivelinessSubscriber,
  LivelinessToken,
  MatchingListener,
  Publisher,
  Querier,
  Queryable,
  SampleMissListener,
  Scout,
  Session,
  Subscriber,
  TransportEventsListener,
} from './binding.js'

Session.prototype[Symbol.asyncDispose] = async function () {
  return await this.close()
}

Subscriber.prototype[Symbol.dispose] = function () {
  return this.undeclare()
}

Publisher.prototype[Symbol.dispose] = function () {
  return this.undeclare()
}

MatchingListener.prototype[Symbol.dispose] = function () {
  return this.undeclare()
}

SampleMissListener.prototype[Symbol.dispose] = function () {
  return this.undeclare()
}

LivelinessToken.prototype[Symbol.dispose] = function () {
  return this.undeclare()
}

LivelinessSubscriber.prototype[Symbol.dispose] = function () {
  return this.undeclare()
}

Querier.prototype[Symbol.dispose] = function () {
  return this.undeclare()
}

Queryable.prototype[Symbol.dispose] = function () {
  return this.undeclare()
}

Scout.prototype[Symbol.dispose] = function () {
  return this.stop()
}

LinkEventsListener.prototype[Symbol.dispose] = function () {
  return this.undeclare()
}

TransportEventsListener.prototype[Symbol.dispose] = function () {
  return this.undeclare()
}

export function defineConfig(config = {}) {
  return Config.fromJson5(JSON.stringify(config))
}

export * from './binding.js'
