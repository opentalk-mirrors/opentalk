{{- with .Values }}
[auth]
issuer = "{{ .recorder.auth.issuer }}"
client_id = "{{ .recorder.auth.clientId }}"
client_secret = "{{ .recorder.auth.clientSecret }}"

[controller]
domain = "{{ .recorder.controller.domain }}"
insecure = {{ .recorder.controller.insecure }}

[rabbitmq]
uri = "{{ .recorder.rabbitmq.uri }}"
queue = "{{ .recorder.rabbitmq.queue }}"
{{- end }}