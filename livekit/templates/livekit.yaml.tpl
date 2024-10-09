---
port: {{ .Values.livekit.port }}
{{ if .Values.livekit.bindAdress }}
bind_addresses: [
{{- range $index, $directive := .Values.livekit.bindAdress }}
    "{{ $directive }}"{{ if lt $index (add (len $.Values.livekit.bindAdress) -1) }},{{ end }}
{{- end }}
]
{{ end }}
rtc:
    tcp_port: {{ .Values.livekit.rtc.tcpPort }}
    port_range_start: {{ .Values.livekit.rtc.portRangeStart }}
    port_range_end: {{ .Values.livekit.rtc.portRangeEnd }}
    use_external_ip: {{ .Values.livekit.rtc.useExternalIp }}
    enable_loopback_candidate: {{ .Values.livekit.rtc.enableLoopbackCandidate }}
#redis:
    #address: "{{ .Values.livekit.redis.address }}"
    #username: "{{ .Values.livekit.redis.username }}"
    #password: "{{ .Values.livekit.redis.password }}"
    #db: {{ .Values.livekit.redis.db }}
    #use_tls: {{ .Values.livekit.redis.useTls }}
    #sentinel_master_name: "{{ .Values.livekit.redis.sentinelMasterName }}"
    #sentinel_username: "{{ .Values.livekit.redis.sentinelUsername }}"
    #sentinel_password: "{{ .Values.livekit.redis.sentinelPassword }}"
    #sentinel_addresses: {{ .Values.livekit.redis.sentinelAddresses }}
    #cluster_addresses: {{ .Values.livekit.redis.clusterAddresses }}
    #max_redirects: {{ .Values.livekit.redis.maxRedirects }}
turn:
    enabled: {{ .Values.livekit.turn.enabled }}
    #domain: "turn.livekit-test.opentalk.run"
    #tls_port: 5349
    #udp_port: 3478
    #external_tls: true
keys:
    controller: "{{ .Values.livekit.keys.controller }}"
logging:
    json: {{ .Values.livekit.logging.json }}
    level: {{ .Values.livekit.logging.level }}
