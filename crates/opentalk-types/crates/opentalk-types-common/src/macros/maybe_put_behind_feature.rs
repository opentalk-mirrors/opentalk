// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Conditionally put items behind a feature flag, used inside macros
///
/// # Example:
///
/// ```rust
/// opentalk_types_common::maybe_put_behind_feature!{
///     feature_gate_it = true; // <- decides if the feature gate is applied or not
///     feature = "my-feature";
///
///     pub struct MyStruct {}
///
///     impl MyStruct {}
/// }
/// ```
///
/// Depending if `feature_gate_it` is `true` or `false` the output looks like:
///
/// ```rust
/// #[cfg(feature = "my-feature")]
/// pub struct MyStruct {}
///
/// #[cfg(feature = "my-feature")]
/// impl MyStruct {}
/// ```
///
/// or
///
/// ```rust
/// pub struct MyStruct {}
///
/// impl MyStruct {}
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! maybe_put_behind_feature {
    (
        feature_gate_it = false;
        feature = $feature:literal;
        $($item:item)+
    ) => {
        $($item)+
    };
    (
        feature_gate_it = true;
        feature = $feature:literal;
        $($item:item)+
    ) => {
        $(
        #[cfg(feature = $feature)]
        $item
        )+
    };
}

/// Conditionally put meta attributes behind a feature flag
///
/// # Example:
///
/// ```rust
/// use serde::{Serialize, Deserialize};
///
/// opentalk_types_common::maybe_put_meta_behind_feature!{
///     feature_gate_it = true; // <- decides if the feature gate is applied or not
///     feature = "serde";
///     meta = #[derive(Serialize, Deserialize)]; // <- This is the meta to maybe put behind a cfg_attr
///
///     item: // <- Declare that the item to put the meta on begins
///
///     pub struct MyStruct {}
/// }
/// ```
///
/// Depending if `feature_gate_it` is `true` or `false` the output looks like:
///
/// ```rust
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// pub struct MyStruct {}
/// ```
///
/// or
///
/// ```rust
/// use serde::{Serialize, Deserialize};
///
/// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// pub struct MyStruct {}
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! maybe_put_meta_behind_feature {
    (
        feature_gate_it = false;

        $(
        feature = $feature:literal;
        meta = $(#[$meta:meta]),*;
        )+

        item:
        $item:item
    ) => {
        $($(#[$meta])*)+
        $item
    };
    (
        feature_gate_it = true;

        $(
        feature = $feature:literal;
        meta = $(#[$meta:meta]),*;
        )+

        item:
        $item:item
    ) => {
        $($(#[cfg_attr(feature = $feature, $meta)])*)+
        $item
    };
}
