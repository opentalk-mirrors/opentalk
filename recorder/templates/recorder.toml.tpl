[auth]
issuer = "{{ .Values.recorder.auth.issuer }}"
client_id = "{{ .Values.recorder.auth.clientId }}"
client_secret = "{{ .Values.recorder.auth.clientSecret }}"

[controller]
domain = "{{ .Values.recorder.controller.domain }}"
insecure = {{ .Values.recorder.controller.insecure }}

[rabbitmq]
uri = "{{ .Values.recorder.rabbitmq.uri }}"
queue = "{{ .Values.recorder.rabbitmq.queue }}"
