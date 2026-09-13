// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

#import "@preview/linguify:0.5.0": *

#set page(
  paper: "a4",
)
#set text(
  size: 10pt,
  hyphenate: true,
)

// Helper that allows long mixed-alphanumeric tokens (e.g. "Room42Guest", user
// names or titles containing digits) to wrap. Typst's hyphenator only runs on
// tokens made entirely of letters from the active language; as soon as a
// digit (or other non-letter) appears, the whole token becomes unbreakable
// and overflows narrow table cells.
//
// The helper performs a string-level `str.replace`: it matches a run of
// digits together with an optional adjacent letter on each side, and
// re-emits the characters joined by a zero-width space (U+200B). The ZWSP
// is invisible but provides a legal line-break opportunity, and it also
// splits the token into pure-letter substrings that the hyphenator can then
// process normally.
//
// Matching the digit run as a whole (rather than just a letter/digit pair)
// is important: regex matches are non-overlapping, so for a token like
// "Foo1Bar" a pair-based regex would only fire on "o1" and miss the "1B"
// boundary. We can't use look-around either, because Typst's regex engine
// (Rust's `regex` crate) doesn't support it.
//
// We use `str.replace` rather than a `show regex(...)` rule for two
// reasons: the helper returns a `str`, which is auto-promoted to content
// in markup contexts, and crucially can still be passed as a scalar to
// plugins (`linguify(args: ...)` forwards values to Fluent, which rejects
// content). The transformation is scoped to this helper (rather than
// applied document-wide) so that fixed strings used by tests are not
// modified; only the user-supplied fields that we explicitly wrap below
// are affected.
#let wrappable(s) = if s == none { none } else {
  s.replace(
    regex("\p{L}?\d+\p{L}?"),
    m => m.text.clusters().join("\u{200B}"),
  )
}

#let data = json("data.json")

#set-database(eval(load-ftl-data("./l10n", data.available_languages)))
#set text(lang: data.report_language)

#let parse_datetime(s) = toml(bytes("date = " + s)).date
#let datetime_format = "[year]-[month]-[day] [hour]:[minute]"
#let role_label = (
  moderator: linguify("moderator"),
  user: linguify("user"),
  guest: linguify("guest"),
)
#let role_order = (
  moderator: 0,
  user: 1,
  guest: 2,
)

= #linguify("attendance_report")

#let metadata_table_content = (
  (
    linguify("meeting"),
    wrappable(data.title),
  ),
)

#if data.description.len() > 0 {
  metadata_table_content.push((
    linguify("details"),
    wrappable(data.description),
  ))
}

#if "starts_at" in data {
  metadata_table_content.push((
    linguify("planned_start"),
    parse_datetime(data.starts_at).display(datetime_format),
  ))
}

#if "ends_at" in data {
  metadata_table_content.push((
    linguify("planned_end"),
    parse_datetime(data.ends_at).display(datetime_format),
  ))
}

#metadata_table_content.push((
  linguify("report_created_at"),
  parse_datetime(data.report_created_at).display(datetime_format),
))

#metadata_table_content.push((
  linguify("report_timezone"),
  data.report_timezone,
))


#table(
  stroke: none,
  columns: (auto, 1fr),
  ..for (name, content) in metadata_table_content {
    ([*#name*:], [#content])
  }
)

== #linguify("participants")

#set table.hline(stroke: 0.5pt + rgb("bfbfbf"))

#table(
  stroke: none,
  columns: (auto, 1fr, auto),
  table.header([*#linguify("nr")*], [*#linguify("name")*], [*#linguify("role")*]),
  table.hline(y: 0),
  table.hline(y: 1),
  ..for (i, participant) in data.participants.sorted(key: p => role_order.at(p.role)).enumerate(start: 1) {
    (
      [#i],
      [#wrappable(participant.name)],
      [#role_label.at(participant.role)],
    )
  },
)
