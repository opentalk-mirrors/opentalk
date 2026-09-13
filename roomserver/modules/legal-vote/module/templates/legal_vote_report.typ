// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

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
#let datetime_format = "[year]-[month]-[day] [hour]:[minute]:[second]"
#let vote_option = (
  yes: linguify("approval"),
  no: linguify("disapproval"),
  abstain: linguify("abstention"),
)

= #linguify("opentalk_vote_report")

#let metadata_table_content = (
  (
    linguify("title"),
    wrappable(data.summary.title),
  ),
)

#if "subtitle" in data.summary {
  metadata_table_content.push((
    linguify("subtitle"),
    wrappable(data.summary.subtitle),
  ))
}

#if "topic" in data.summary {
  metadata_table_content.push((
    linguify("topic"),
    wrappable(data.summary.topic),
  ))
}

#metadata_table_content.push((
  linguify("pseudonymous"),
  [#if data.summary.pseudonymous { linguify("yes") } else { linguify("no") }],
))

#metadata_table_content.push((
  linguify("referendum_leader"),
  wrappable(data.summary.creator),
))

#metadata_table_content.push((
  linguify("vote_id"),
  data.summary.id,
))

#metadata_table_content.push((
  linguify("start"),
  [ #parse_datetime(data.summary.start_time).display(datetime_format) ],
))

#if "end_time" in data.summary {
  metadata_table_content.push((
    linguify("end"),
    [ #parse_datetime(data.summary.end_time).display(datetime_format) ],
  ))
}

#metadata_table_content.push((
  linguify("report_timezone"),
  data.summary.report_timezone,
))

#metadata_table_content.push((
  linguify("participant_count"),
  data.summary.participant_count,
))

#metadata_table_content.push((
  linguify("scheduled_duration"),
  if "duration" in data.summary {
    [#data.summary.duration s]
  } else {
    linguify("unlimited")
  },
))

#metadata_table_content.push((
  linguify("abstention"),
  if data.summary.enable_abstain {
    linguify("allowed")
  } else {
    linguify("disallowed")
  },
))

#metadata_table_content.push((
  linguify("automatic_close"),
  if data.summary.auto_close {
    linguify("enabled")
  } else {
    linguify("disabled")
  },
))

#metadata_table_content.push((
  linguify("vote_ended_due_to"),
  if data.summary.stop_reason.kind == "by_user" {
    linguify("user_ended_the_vote", args: (user: wrappable(data.summary.stop_reason.user)))
  } else if data.summary.stop_reason.kind == "auto" {
    linguify("all_users_voted")
  } else if data.summary.stop_reason.kind == "expired" {
    linguify("expired")
  } else if data.summary.stop_reason.kind == "canceled" {
    if data.summary.stop_reason.reason == "room_destroyed" {
      linguify("aborted_by_room_close")
    } else if data.summary.stop_reason.reason == "initiator_left" {
      linguify("aborted_by_vote_initiator_leaving")
    } else if data.summary.stop_reason.reason == "custom" {
      linguify("aborted_for_custom_reason", args: (reason: wrappable(data.summary.stop_reason.custom)))
    } else {
      linguify("aborted_for_unknown_reason")
    }
  } else {
    linguify("unknown_reason")
  },
))

#metadata_table_content.push((
  linguify("number_of_votes"),
  data.summary.vote_count,
))

#table(
  stroke: none,
  columns: (auto, 1fr),
  ..for (name, content) in metadata_table_content {
    ([*#name*:], [#content])
  }
)

#set table.hline(stroke: 0.5pt + rgb("bfbfbf"))

#if "final_results" in data.summary and data.summary.final_results.results == "valid" [

  == #linguify("results")

  #let results_table_content = (
    (
      linguify("approval"),
      data.summary.final_results.yes,
    ),
    (
      linguify("disapproval"),
      data.summary.final_results.no,
    ),
  )

  #if "abstain" in data.summary.final_results {
    results_table_content.push((
      linguify("abstention"),
      data.summary.final_results.abstain,
    ))
  }

  #set table.hline(stroke: 0.5pt + rgb("bfbfbf"))
  #table(
    stroke: none,
    columns: (auto, 1fr),
    table.header([*#linguify("vote")*], [*#linguify("count")*]),
    table.hline(y: 0),
    table.hline(y: 1),
    ..for (vote, count) in results_table_content {
      ([#vote], [#count])
    },
  )

]

== Recorded votes

#set table.hline(stroke: 0.5pt + rgb("bfbfbf"))
#table(
  stroke: none,
  columns: (1fr, auto, auto, auto),
  table.header([*#linguify("name")*], [*#linguify("token")*], [*#linguify("vote")*], [*#linguify("timestamp")*]),
  table.hline(y: 0),
  table.hline(y: 1),
  ..for vote in data.votes {
    (
      if "name" in vote [
        #wrappable(vote.name)
      ] else {
        linguify("hidden")
      },
      [#vote.token],
      [#vote_option.at(vote.option)],
      if "time" in vote [
        #parse_datetime(vote.time).display(datetime_format)
      ] else [
        —
      ],
    )
  },
)

== #linguify("event_log")

#set table.hline(stroke: 0.5pt + rgb("bfbfbf"))
#table(
  stroke: none,
  columns: (1fr, auto, 1fr),
  table.header([*#linguify("name")*], [*#linguify("timestamp")*], [*#linguify("event")*]),
  table.hline(y: 0),
  table.hline(y: 1),
  ..for event in data.events {
    (
      if "name" in event.event_details [
        #wrappable(event.event_details.name)
      ] else {
        linguify("anonymous")
      },
      if "time" in event [
        #parse_datetime(event.time).display(datetime_format)
      ] else [
        —
      ],
      {
        if event.kind == "issue" {
          let issue = if "kind" not in event.event_details {
            linguify("reports_a_problem")
          } else if event.event_details.kind == "audio" {
            linguify("reports_an_audio_issue")
          } else if event.event_details.kind == "video" {
            linguify("reports_a_video_issue")
          } else if event.event_details.kind == "screenshare" {
            linguify("reports_a_screenshare_issue")
          }

          if "description" in event.event_details [
            #issue: #wrappable(event.event_details.description)
          ] else [
            #issue
          ]
        } else if event.kind == "user_joined" {
          linguify("user_joined")
        } else if event.kind == "user_left" {
          linguify("user_left")
        }
      },
    )
  },
)
