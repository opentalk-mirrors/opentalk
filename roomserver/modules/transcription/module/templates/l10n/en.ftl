# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

speaker = Speaker { $number }
unknown = Unknown
protocol = Protocol
protocol_start = Protocol was started
protocol_pause = Protocol was paused
protocol_resume = Protocol was resumed
protocol_end = Protocol was stopped
automatic_meeting_protocol = Automatic Meeting Protocol
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
disclaimer_errors = This meeting protocol was generated automatically and may contain errors.
page_number = Page { $current } of { $total }
