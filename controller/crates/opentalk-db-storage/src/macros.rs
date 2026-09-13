// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Creates a diesel expression that checks if the passed $v is empty
///
/// Originally from [casbin-rs/diesel-adapter][origin]
///
/// If it is not empty the $field must match $v
/// Basically: `($v.is_empty() OR !$v.is_empty() AND $field = $v)`
/// Example:
/// ```ignore,rust
/// diesel::delete(
///   table.filter(
///     ptype
///     .eq("p")
///     .and(db_storage::eq_empty!("", v4))
///     .and(db_storage::eq_empty!("", v5)),
///   ),
/// );
/// ```
///
/// Result:
/// ```ignore,sql
/// DELETE FROM "table" WHERE "table"."ptype" = $1 AND ($2 OR $3 AND "table"."v4" = $4) AND ($5 OR $6 AND "table"."v5" = $7) -- binds: ["p", true, false, "", true, false, ""]
/// ```
///
/// [origin]: https://github.com/casbin-rs/diesel-adapter/blob/c23a53ae84cb92c67ae2f2c3733a3cc07b2aaf0b/src/macros.rs
#[macro_export]
macro_rules! eq_empty {
    ($v:expr,$field:expr) => {{
        || {
            use diesel::BoolExpressionMethods;

            ::diesel::dsl::sql("")
                .bind::<diesel::sql_types::Bool, _>($v.is_empty())
                .or(diesel::dsl::sql("")
                    .bind::<diesel::sql_types::Bool, _>(!$v.is_empty())
                    .and($field.eq($v)))
        }
    }
    ()};
}
