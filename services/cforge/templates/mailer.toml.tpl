
{{- with .Values }}
{{- if .mailer.rabbitMq.url -}}
[rabbit_mq]
# The URL to use to connect to the rabbit mq broker
url = "{{ .mailer.rabbitMq.url }}"
# The queue to subscribe to
mail_task_queue = "{{ .mailer.rabbitMq.mailTaskQueue }}"
{{ end }}

{{- if .mailer.smtp.smtpServer -}}
[smtp]
# SMTP Cleartext: smtp://foo:bar@my-mailserver:1234/?disable_starttls=true
# SMTP with StartTLS: smtp://foo:bar@my-mailserver:1234/
# SMTP with implicit TLS: smtps://foo:bar@my-mailserver:1234/
smtp_server = "{{ .mailer.smtp.smtpServer }}"
# Set the From email address according to the requirements of your SMTP server.
from_name = "{{ .mailer.smtp.fromName }}"
from_email = "{{ .mailer.smtp.fromEmail }}"
{{ end }}

[template_builder]
# Link builder templates. `base_url` and `x_id` will be replaced. Need when the frontend uses different paths.
#join_link_builder = "{{ .mailer.templateBuilder.joinLinkBuilder }}"
#guest_link_builder = "{{ .mailer.templateBuilder.guestLinkBuilder }}"
#dashboard_event_link_builder = "{{ .mailer.templateBuilder.dashboardEventLinkBuilder }}"

[frontend]
base_url = "{{ .mailer.frontend.joinLinkBuilder }}"
#data_protection_url = "{{ .mailer.frontend.dataProtectionUrl }}"

[templates]
# Path to templates. These are loaded at runtime and are not compiled in.
#registered_invite = {{ .mailer.templates.registeredInvite }}
#unregistered_invite = {{ .mailer.templates.unregisteredInvite }}
#external_invite = {{ .mailer.templates.externalInvite }}
#registered_event_update = {{ .mailer.templates.registeredEventUpdate }}
#unregistered_event_update  = {{ .mailer.templates.unregisteredEventUpdate }}
#external_event_update = {{ .mailer.templates.externalEventUpdate }}
#registered_event_cancellation = {{ .mailer.templates.registeredEventCancellation }}
#unregistered_event_cancellation = {{ .mailer.templates.unregisteredEventCancellation }}
#external_event_cancellation = {{ .mailer.templates.externalEventCancellation }}
#registered_uninvite = {{ .mailer.templates.registeredUninvite }}
#unregistered_uninvite = {{ .mailer.templates.unregisteredUninvite }}
#external_uninvite = {{ .mailer.templates.externalUninvite }}

[languages]
default_language = 'de-DE'

# Optional support contact information
# Adds a support contact section to the template
[support_contact]
# Support phone contact
phone = "+49123321123"
# Support mail contact
mail = "support@example.com"
{{- end }}