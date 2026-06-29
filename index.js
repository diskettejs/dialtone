import {
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
} from './binding.js'

Session.prototype[Symbol.asyncDispose] = async function () {
  return await this.close()
}

Subscriber.prototype[Symbol.asyncDispose] = async function () {
  return await this.undeclare()
}

Publisher.prototype[Symbol.asyncDispose] = async function () {
  return await this.undeclare()
}

MatchingListener.prototype[Symbol.asyncDispose] = async function () {
  return await this.undeclare()
}

SampleMissListener.prototype[Symbol.asyncDispose] = async function () {
  return await this.undeclare()
}

LivelinessToken.prototype[Symbol.asyncDispose] = async function () {
  return await this.undeclare()
}

LivelinessSubscriber.prototype[Symbol.asyncDispose] = async function () {
  return await this.undeclare()
}

Querier.prototype[Symbol.asyncDispose] = async function () {
  return await this.undeclare()
}

Queryable.prototype[Symbol.asyncDispose] = async function () {
  return await this.undeclare()
}

Scout.prototype[Symbol.dispose] = function () {
  return this.stop()
}

export * from './binding.js'
