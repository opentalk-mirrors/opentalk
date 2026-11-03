# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

speaker = Sprecher { $number }
unknown = Unbekannt
transcript = Protokoll
transcript_start = Die Protokollierung wurde gestartet
transcript_pause = Die Protokollierung wurde pausiert
transcript_resume = Die Protokollierung wurde fortgesetzt
transcript_end = Die Protokollierung wurde beendet
automatic_transcript = Automatisches Protokoll
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
