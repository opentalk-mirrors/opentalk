# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

speaker = Sprecher { $number }
unknown = Unbekannt
protocol = Protokoll
protocol_start = Die Protokollierung wurde gestartet
protocol_pause = Die Protokollierung wurde pausiert
protocol_resume = Die Protokollierung wurde fortgesetzt
protocol_end = Die Protokollierung wurde beendet
automatic_meeting_protocol = Automatisches Protokoll
date = Datum
time = Zeit
moderated_by = Moderation
participants = Teilnehmende
language = Sprache
pseudonymization = Pseudonymisierung
enabled = Aktiviert
disabled = Deaktiviert
language_name = { $language ->
    [de] Deutsch
    [en] Englisch
    *[other] { $language }
}
automatic = Automatisch
disclaimer_errors = Dieses Protokoll wurde automatisch erstellt und kann Fehler enthalten.
page_number = Seite { $current } von { $total }
