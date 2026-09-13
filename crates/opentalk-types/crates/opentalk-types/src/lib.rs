// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/*!
Types and traits for the OpenTalk API and signaling protocols.

This crate re-exports items from the crates where they are defined.
All of these types can be used directly by importing the crate where they
are defined directly. If many types are needed in a project, this crate
could be the easier starting point though, because it serves as an index of
which crates are available.

# Crate features

In order to allow efficient usage of the referenced items, this crate
defines a flag for each of them. In addition some meta features are
available that pull in a set of dependencies.

## Meta features

### Types meta features

* **types-all** -
  Enables all features that depend on specific datatype crates, indirectly by
  enabling the **api** and the **signaling-all** features.
* **api** -
  Enables the **api-v1** feature.
* **signaling-all** -
  Enables all the signaling datatypes feature for all signaling modules.

### Functionality meta features

* **backend** -
  Should be enabled when implementing the server side of either a signaling module
  or an the API. Enables:
  * the **backend** feature of each types crate
  * **diesel**
  * **rand**
  * **redis**
  * **serde**
* **frontend** -
  Should be enabled when implemting the client side of either a signaling module
  or the API. Enables:
  * the **frontend** feature of each types crate
  * **serde**

## Type features

* **api-v1** -
  Re-exports [`opentalk-types-api-v1`](https://docs.rs/opentalk-types-api-v1) crate as [`api::v1`].
* **signaling** -
  Re-exports all content of the [`opentalk-types-signaling`](https://docs.rs/opentalk-types-signaling) crate in the [`signaling`] module.

## Functionality features

* **clap** -
  Should be enabled when implementing command-line tooling that exposes the
  types using [`clap`](https://docs.rs/clap). This allows listing the possible
  values of enumeration types using the builtin `--help` functionality of `clap`.
* **diesel** -
  Enabling this feature makes some newtypes storable in a database through the
  [`diesel`](https://docs.rs/diesel) crate. Used for implementing the server
  side with a `diesel` database backend.
* **rand** -
  Adds random generation of some datatypes such as `uuid`, so that they can
  be generated on the server side in the API endpoint implementation or inside
  signaling modules.
* **redis** -
  Adds annotations to some signaling data types so that they can be stored
  inside redis by the server side of a signaling module.
* **serde** -
  Adds [`serde`](https://docs.rs/serde/) `Serialize` and `Deserialize`
  implementations for each datatype that is sent over the network.
* **utoipa** -
  Adds [`utoipa`](https://docs.rs/utoipa/) `ToSchema` and `IntoParams`
  implementations to all types that are exposed in the OpenTalk Web API, so that
  they can be used to generate an OpenAPI specification of the Web API.
*/

#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

#[cfg(feature = "common")]
pub use opentalk_types_common as common;

#[cfg(feature = "api-v1")]
pub mod api {
    /*!
    Re-exports all known API versions under the corresponding name.
    */
    pub use opentalk_types_api_v1 as v1;
}

#[cfg(feature = "signaling")]
pub mod signaling {
    /*!
    Re-exports all known signaling modules, each gated by a feature
    called `signaling-<modulename>`, e.g. `signaling-control` or
    `signaling-subroom-audio`.
    */
    pub use opentalk_types_signaling::*;
}
