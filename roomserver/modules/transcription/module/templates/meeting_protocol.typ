// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

#import "@preview/linguify:0.5.0": *

#set text(
  size: 10pt,
)

#set par(
  leading: 1em,
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

#let database = eval(load-ftl-data("./l10n", data.available_languages))
#set page(
  paper: "a4",
  footer: context align(center, text(luma(45%), linguify(
    "page_number",
    from: database,
    args: (
      current: counter(page).get().first(),
      total: counter(page).final().first(),
    ),
  ))),
)


#set-database(database)
#set text(lang: data.report_language)

#let parse_datetime(s) = toml(bytes("date = " + s)).date
#let day_format = "[day].[month].[year]"
#let exact_time_format = "[hour]:[minute]:[second]"
#let loose_time_format = "[hour]:[minute]"

#let dim(value) = text(luma(45%), value)
#let speaker_kinds = (
  name: speaker => wrappable(speaker.name),
  pseudonym: speaker => linguify("speaker", args: (number: speaker.number)),
  unknown: speaker => linguify("unknown"),
)
#let display_speaker(speaker) = text(weight: "bold", speaker_kinds.at(speaker.kind)(speaker))

#let display_marker(key) = dim(emph(linguify(key)))
#let display_event = (
  start: event => display_marker("protocol_start"),
  pause: event => display_marker("protocol_pause"),
  resume: event => display_marker("protocol_resume"),
  end: event => display_marker("protocol_end"),
  text: event => [#display_speaker(event.speaker) \ #wrappable(event.text)],
)
#let title = data.at("title", default: none)

#let started_at = parse_datetime(data.started_at)
#let transcription_language = data.at("transcription_language", default: none)
#let ended_at = parse_datetime(data.ended_at)

// We need to use linguify-raw here because the regular `linguify()` returns a `context` expression, which Typst only
// resolved when placed somewhere in the document, which is not the case for the document title.
#let transcript_label = linguify-raw("transcript", from: database, lang: data.report_language)
#set document(title: (title, transcript_label, started_at.display(day_format)).filter(it => it != none).join(" - "))

#if title != none [= #wrappable(title) ]
#smallcaps[#dim(linguify("automatic_transcript"))]
#v(1em)

#let separator = line(length: 100%, stroke: luma(70%))
#separator

#let summary_row(key, value) = (dim(linguify(key)), text(weight: "bold", value))
#grid(
  columns: (auto, auto),
  column-gutter: 5em,
  row-gutter: 1em,
  ..summary_row("date", started_at.display(day_format)),
  ..summary_row("time", [#started_at.display(loose_time_format) - #ended_at.display(loose_time_format)]),
  ..summary_row("language", if transcription_language != none {
    linguify("language_name", args: (language: transcription_language))
  } else { linguify("automatic") }),
  ..summary_row("participants", [#data.present_participant_count]),
  ..summary_row("pseudonymization", if data.pseudonymous { linguify("enabled") } else { linguify("disabled") }),
  ..summary_row("moderated_by", data.moderators.map(display_speaker).join(", ")),
)

#separator

#block(radius: 4pt, inset: 8pt, fill: luma(230), width: 100%)[
  #linguify("disclaimer_errors")
]

#for entry in data.protocol {
  block(below: 1.5em)[
    #dim(parse_datetime(entry.timestamp).display(exact_time_format))
    #display_event.at(entry.event.kind)(entry.event)
  ]
}
