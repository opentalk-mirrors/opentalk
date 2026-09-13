// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Defines a new SQL enum
///
/// The first argument is the resulting type for use in rust code.
/// The second argument is the serialized version of the enum name.
/// The third argument is the sql type.
/// The 4th argument is the string identifier of the sql type.
///
/// After that follow the variants of the enum with the syntax of:
/// `RustVariant = byte-string`
///
/// # Example
///
/// ```rust,ignore
/// sql_enum!(
///     CustomSqlEnum,          // Name of the Rust enum name
///     "custom_sql_enum",      // Name of the type name in sql
///     CustomSqlEnumType,      // Name of the diesel enum type repr
///     {
///         Variant1 = b"variant1", // the variants with their respective sql string representation
///         Variant2 = b"variant2",
///     }
/// );
/// ```
#[macro_export]
macro_rules! sql_enum {
    (
        feature_gated:
        $(#[$enum_meta:meta])* $enum_ident:ident,
        $sql_type_lit:literal,
        $(#[$type_meta:meta])* $type_ident:ident,
        {$($variant_ident:ident = $variant_lit:literal),* $(,)?}
    ) => {
        $crate::sql_enum!(
            @impl__
            true
            $(#[$enum_meta])* $enum_ident,
            $sql_type_lit,
            $(#[$type_meta])* $type_ident,
            {$($variant_ident = $variant_lit),*}
        );
    };

    (
        $(#[$enum_meta:meta])* $enum_ident:ident,
        $sql_type_lit:literal,
        $(#[$type_meta:meta])* $type_ident:ident,
        {$($variant_ident:ident = $variant_lit:literal),* $(,)?}
    ) => {
        $crate::sql_enum!(
            @impl__
            false
            $(#[$enum_meta])* $enum_ident,
            $sql_type_lit,
            $(#[$type_meta])* $type_ident,
            {$($variant_ident = $variant_lit),*}
        );
    };

    (
        @impl__
        $do_feature_gate:ident
        $(#[$enum_meta:meta])* $enum_ident:ident,
        $sql_type_lit:literal,
        $(#[$type_meta:meta])* $type_ident:ident,
        {$($variant_ident:ident = $variant_lit:literal),* $(,)?}
    ) => {
        $crate::maybe_put_meta_behind_feature! {
            feature_gate_it = $do_feature_gate;
            feature = "diesel";
            meta =
                #[derive(::diesel::SqlType, ::diesel::QueryId)],
                #[diesel(postgres_type(name = $sql_type_lit))];

            item:
            $(#[$type_meta])*
            #[derive(Debug)]
            #[allow(missing_docs)]
            pub struct $type_ident;
        }

        $crate::maybe_put_meta_behind_feature! {
            feature_gate_it = $do_feature_gate;
            feature = "diesel";
            meta =
                #[derive(::diesel::deserialize::FromSqlRow, ::diesel::expression::AsExpression)],
                #[diesel(sql_type = $type_ident)];

            item:
            $(#[$enum_meta])*
            #[derive(Debug, Copy, Clone)]
            #[allow(missing_docs)]
            pub enum $enum_ident {
                $($variant_ident),*
            }
        }

        $crate::maybe_put_behind_feature! {
            feature_gate_it = $do_feature_gate;
            feature = "diesel";

            impl ::diesel::serialize::ToSql<$type_ident, ::diesel::pg::Pg> for $enum_ident {
                fn to_sql<'b>(
                    &'b self,
                    out: &mut ::diesel::serialize::Output<'b, '_, ::diesel::pg::Pg>
                ) -> ::diesel::serialize::Result {
                    match *self {
                        $(
                            Self::$variant_ident => std::io::Write::write_all(out, $variant_lit)?,
                        )*
                    }

                    Ok(::diesel::serialize::IsNull::No)
                }
            }

            impl ::diesel::deserialize::FromSql<$type_ident, ::diesel::pg::Pg> for $enum_ident {
                fn from_sql(
                    bytes: <::diesel::pg::Pg as ::diesel::backend::Backend>::RawValue<'_>
                ) -> ::diesel::deserialize::Result<Self> {
                    match bytes.as_bytes() {
                        $($variant_lit => Ok(Self::$variant_ident),)*
                        _ => Err("unknown enum variant".into()),
                    }
                }
            }
        }
    };
}
