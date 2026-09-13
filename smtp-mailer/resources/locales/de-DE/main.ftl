registered-event-invite-subject = Einladung zum OpenTalk-Meeting - {$event-name}
unregistered-event-invite-subject = Einladung zum OpenTalk-Meeting - {$event-name}
event-update-subject = OpenTalk-Meeting wurde geändert - {$event-name}
event-cancellation-subject = OpenTalk-Meeting wurde abgesagt - {$event-name}
event-uninvite-subject = Sie wurden von einem OpenTalk-Meeting entfernt - {$event-name}

# Arguments name
invite = Einladung zum OpenTalk-Meeting - { $name }
        .p = {$firstname} {$lastname} lädt Sie zu einem OpenTalk-Meeting ein. Sie können diese Einladung bei OpenTalk ansehen.

view-in-dashboard-link = Einladung ansehen
        .alt = Klicken um im Dashboard zu öffnen

invite-unregistered = Einladung zum OpenTalk-Meeting - { $name }
        .p = {$firstname} {$lastname} lädt Sie zu einem OpenTalk-Meeting ein. Sie können sich einmalig bei OpenTalk einloggen sind, um diese Einladung dort ansehen zu können.

view-in-dashboard-link-unregistered = Anmelden und Einladung ansehen
        .alt = Klicken um im Dashboard zu öffnen

invite-external = Einladung zum OpenTalk-Meeting - { $name }
        .p = {$firstname} {$lastname} lädt Sie zu einem OpenTalk-Meeting ein.

event_update = OpenTalk-Meeting wurde geändert - { $name }
        .p = {$firstname} {$lastname} hat ein OpenTalk-Meeting geändert. Sie können diese Einladung bei OpenTalk ansehen.

event_cancellation = OpenTalk-Meeting wurde abgesagt - { $name }
        .p = {$firstname} {$lastname} hat ein OpenTalk-Meeting abgesagt.

event_uninvite = Einladung zum OpenTalk-Meeting wurde zurückgezogen - { $name }
        .p = {$firstname} {$lastname} hat sie von einem OpenTalk-Meeting entfernt.

meeting-information = Meeting-Information
meeting-information-title = Titel
meeting-information-details = Details
meeting-information-when = Uhrzeit
meeting-information-link = Link zum Meeting
join-directly-link = Öffne den Raum in einem neuen Tab/Fenster
        .alt = Klicken um zu öffnen

meeting-information-password = Meeting-Passwort

call-in-header = Beitritt via Telefon
call-in-hint = Sie können Ihr Telefon verwenden um diesem Meeting beizutreten.
call-in-desc = Einfach die unten angegebene Nummer wählen und Konferenzkennung sowie Konferenz-PIN angeben. Alternativ direkt mit dem Mobiltelefon auf die Schnelleinwahl klicken und direkt loslegen.

call-in-number = Telefonnummer
call-in-id = Konferenzkennung
call-in-pw = Konferenz-PIN
call-in-quick-dial = Schnelleinwahl

shared-folder = Geteilter Ordner
shared-folder-password = Ordnerpasswort

questions = Noch Fragen? Wir helfen Ihnen Gerne!
        .p = Um mit einem unserer Mitarbeiter in Kontakt zu treten, rufen Sie uns einfach kostenfrei unter { $phone } an. Alternativ können Sie auch gerne eine E-Mail mit ihrem Anliegen an { $mail } schicken.

fallback-footer = Sollten die Buttons oder Links nicht funktionieren, können Sie die folgenden URLs in ihrem Browser eingeben:
join-directly-fallback = Meeting beitreten: { $link }
view-in-dashboard-fallback = Meeting details: { $link }

quick-guide-hint-txt = Neu bei OpenTalk? Eine Kurzanleitung finden Sie hier: https://opentalk.eu/de/kurzanleitung
quick-guide-hint-html = Neu bei OpenTalk? Eine Kurzanleitung finden Sie hier: <a href="https://opentalk.eu/de/kurzanleitung">https://opentalk.eu/de/kurzanleitung</a>

data-protection-hint = Hinweis zum Datenschutz: Uns ist der Schutz Ihrer Daten besonders wichtig. Auch bei der Verwendung der Videokonferenzlösung können personenbezogene Daten ausgetauscht werden. Erfahren Sie mehr zum Umgang mit Ihren Daten auf unserer Internetseite unter { $link }

adhoc-meeting-info-hours =
        Dies ist ein Ad-hoc-Meeting, diese werden nach { $retentiontime ->
                [1] einer Stunde
                [2] zwei Stunden
                [3] drei Stunden
                [4] vier Stunden
                [5] fünf Stunden
                [6] sechs Stunden
                [7] sieben Stunden
                [8] acht Stunden
                [9] neun Stunden
                [10] zehn Stunden
                [11] elf Stunden
                [12] zwölf Stunden
               *[other] { $retentiontime } Stunden
        } automatisch gelöscht.
adhoc-meeting-info-minutes =
        Dies ist ein Ad-hoc-Meeting, diese werden nach { $retentiontime ->
                [1] einer Minute
                [2] zwei Minuten
                [3] drei Minuten
                [4] vier Minuten
                [5] fünf Minuten
                [6] sechs Minuten
                [7] sieben Minuten
                [8] acht Minuten
                [9] neun Minuten
                [10] zehn Minuten
                [11] elf Minuten
                [12] zwölf Minuten
               *[other] { $retentiontime } Minuten
        } automatisch gelöscht.

streaming-header =
        Dies ist eine öffentliche Sitzung, { $numberstreams ->
                [one] der Link zum Livestream:
               *[other] die Links zu den Livestreams:
        }
