# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

speaker = Sprecher { $number }
unknown = Unbekannt
transcript = Transkript
protocol_start = Die Transkription wurde gestartet
protocol_pause = Die Transkription wurde pausiert
protocol_resume = Die Transkription wurde fortgesetzt
protocol_end = Die Transkription wurde beendet
automatic_transcript = Automatisches Transkript
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
