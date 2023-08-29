# sud

> **s**erialisieren **u**nd **d**eserialisieren

A data serialisation library.

Inspired by:

* [`serde`](https://serde.rs/)
* [`deser`](https://github.com/mitsuhiko/deser)
* [`SAX`](https://en.wikipedia.org/wiki/Simple_API_for_XML)

## Design

To quickly understand the design, let's compare to `serde`

### `serde::Deserializer`

This trait represents a scheme that decodes data from the serialised format, eg JSON, YAML, CBOR. `sud` has no single trait to define this
representation, instead we support any function that produces an event. Eg `Iterator::next`, `Stream::next` etc.

### `serde::Deserialize`

This trait represents a struct that can be filled with data. `sud` has no single trait to define this
representation, instead we support any function that consumes an event. Eg `Extend::extend_one`, `Sink::start_send` etc.

### `serde::Serializer`

This trait represents a scheme that encodes data to the serialised format, eg JSON, YAML, CBOR. `sud` has no single trait to define this
representation, instead we support any function that cosumes an event. Eg `Extend::extend_one`, `Sink::start_send` etc.

### `serde::Serialize`

This trait represents a struct that has data and can be serialised. `sud` has no single trait to define this
representation, instead we support any function that produces an event. Eg `Serializer::fold`, `Stream::next` etc.

---

The common theme here is events. There are event sources and event sinks. The user is responsible for driving the pipeline from event source to sink.

This makes modelling many different API designs much easier. What `sud` gives you that `serde` does not is the ability to produce values from an async `Stream` and write them to an async `TcpStream` with no re-implementation required. The `serde` visitor pattern is very limited to sync only code.

### `deser`

`deser` is an interesting library. It seems to be inspired by the same desire to have an event driven framework, but when I looked into the code it left some things to be desired
1. Length prefixed maps/arrays - Some binary encodings use length prefixing for faster deserialisations
2. Optimisation - performance wasn't a goal of deser. it's full of dynamic dispatch and allocations that I would like to avoid.

## Performance

Unknown.

## Compile times

Unknown.
