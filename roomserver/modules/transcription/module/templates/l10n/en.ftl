# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

speaker = Speaker { $number }
unknown = Unknown
transcript = Transcript
protocol_start = Transcription was started
protocol_pause = Transcription was paused
protocol_resume = Transcription was resumed
protocol_end = Transcription was stopped
automatic_transcript = Automatic Transcript
date = Date
time = Time
moderated_by = Moderated by
participants = Participants
language = Language
pseudonymization = Pseudonymization
enabled = Enabled
disabled = Disabled
language_name = { $language ->
    [de] German
    [en] English
    *[other] { $language }
}
automatic = Automatic
disclaimer_errors = This transcript was generated automatically and may contain errors.
page_number = Page { $current } of { $total }
