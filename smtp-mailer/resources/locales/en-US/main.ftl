registered-event-invite-subject = OpenTalk Meeting Invitation - {$event-name}
unregistered-event-invite-subject = OpenTalk Meeting Invitation - {$event-name}
event-update-subject = OpenTalk Meeting was updated - {$event-name}
event-cancellation-subject = OpenTalk Meeting was cancelled - {$event-name}
event-uninvite-subject = You have been removed from an OpenTalk-Meeting - {$event-name}

# Arguments name
invite = OpenTalk Meeting Invitation - { $name }
        .p = {$firstname} {$lastname} invites you to an OpenTalk Meeting. You can view this invite in OpenTalk.

view-in-dashboard-link = View Invitation
        .alt = Click to view in dashboard

invite-unregistered = OpenTalk Meeting Invitation - { $name }
        .p = {$firstname} {$lastname} invites you to an OpenTalk Meeting. You can login now at OpenTalk to view this invitation.

view-in-dashboard-link-unregistered = Login and View Invitation
        .alt = Click to view in dashboard

invite-external = OpenTalk Meeting Invitation - { $name }
        .p = {$firstname} {$lastname} invites you to an OpenTalk Meeting.

event_update = OpenTalk Meeting was updated - { $name }
        .p = {$firstname} {$lastname} updated an OpenTalk Meeting. You can view this invite in OpenTalk.

event_cancellation = OpenTalk Meeting was cancelled - { $name }
        .p = {$firstname} {$lastname} cancelled an OpenTalk Meeting.

event_uninvite = OpenTalk Meeting Invitation was revoked - { $name }
        .p = {$firstname} {$lastname} removed you from an OpenTalk Meeting.

meeting-information = Meeting Information
meeting-information-title = Title
meeting-information-details = Details
meeting-information-when = Time
meeting-information-link = Link to Meeting
join-directly-link = Open the conference room in new tab/window
        .alt = Click to open

meeting-information-password = Meeting Password

call-in-header = Participate via Phone
call-in-hint = You can use your phone to participate in this meeting.
call-in-desc = Just dial in the following number and enter the given conference ID and conference PIN, or click the Quick-Dial number on your phone.

call-in-number = Number
call-in-id = Conference ID
call-in-pw = Conference PIN
call-in-quick-dial = Quick-Dial

shared-folder = Shared Folder
shared-folder-password = Folder Password

questions = Questions? We are here to help!
        .p = To talk to our help desk, give us a call via { $phone }. Alternatively, you can send us an email to { $mail }.

fallback-footer = If the buttons or links don’t work open your browser and type in the following links:
join-directly-fallback = Join Meeting: { $link }
view-in-dashboard-fallback = Meeting Details: { $link }

quick-guide-hint-txt = New to OpenTalk? Read our quick guide here: https://opentalk.eu/en/quick-manual
quick-guide-hint-html = New to OpenTalk? Read our quick guide here: <a href="https://opentalk.eu/en/quick-manual">https://opentalk.eu/en/quick-manual</a>

data-protection-hint = Note for data protection: The protection of your data is important to us. Personal data can also be exchanged when using the video conference solution. For more information on how we handle your personal data, please visit { $link }

adhoc-meeting-info-hours =
        This is an ad-hoc meeting, it will be automatically deleted after { $retentiontime ->
                [one] one hour
               *[other] { $retentiontime } hours
        }.
adhoc-meeting-info-minutes =
        This is an ad-hoc meeting, it will be automatically deleted after { $retentiontime ->
                [one] one minute
               *[other] { $retentiontime } minutes
        }.

streaming-header =
        This is a public meeting, { $numberstreams ->
                [one] the link to the livestream:
               *[other] the links to the livestreams:
        }
