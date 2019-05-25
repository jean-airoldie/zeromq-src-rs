[![Apache 2.0 licensed](https://img.shields.io/badge/license-Apache2.0-blue.svg)](./LICENSE-APACHE)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE-MIT)

> zeromq-src-rs - Source code and logic to build `libzmq` from source

This crate is intended to be consumed by a `sys` crate.

See [`testcrate-static`](testcrate-static) for a usage example.

# Dependencies
* [CMake 2.8.12+ (or 3.0.2+ on Darwin)](https://github.com/zeromq/libzmq/blob/de4d69f59788fed86bcb0f610723c5acd486a7da/CMakeLists.txt#L7)

# Env Vars
* `DEP_ZMQ_INCLUDE` is path to the include directory.
* `DEP_ZMQ_LIB` is the path to the lib directory.
* `DEP_ZMQ_OUT` is the path to the out directory (root).

# Versioning
* The `master` branch uses the [`libzmq`] master and is considered a developper preview. When a preview is publish,
   the version will take the form of `VERSION-preview+BUILD_METADATA`.
* The `lastest_release` branch uses the [`libzmq`] `latest_release` branch and is considered a stable branch. When a stable release is published, the version will take the form of `VERSION+BUILD_METADATA`.
* In both cases, `BUILD_METADATA` specifies the version of `libzmq` used.

# License
While [`libzmq`] is license under `LGPL`, is has a linking exception, which means that this crate does not need to conform to the usual `LGPL` conditions. Indeed this crate does not modify the source code in any way and simply allows linking to `libzmq`. To quote from the [`zeromq website`]:
> ZeroMQ is safe for use in close-source applications. The LGPL share-alike terms do not apply to applications built on top of ZeroMQ.
> You do not need a commercial license. The LGPL applies to ZeroMQ's own source code, not your applications. Many commercial applications use ZeroMQ.

Thus this project is effectively licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Acknowledgments
* Based on [`openssl-src`]

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `zeromq-src-rs` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[`openssl-src`]: https://github.com/alexcrichton/openssl-src-rs
[`libzmq`]: https://github.com/zeromq/libzmq
[`zeromq website`]: http://zeromq.org/area:licensing
