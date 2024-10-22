{{- with .Values }}
{{- with .livekit }}
---
port: {{ .port }}
{{- with .bindAdress }}
# A list urls of a etcd cluster
bind_addresses: [
{{ $allEntries := . }}{{ range $index, $entry := . }}
  "{{ $entry }}"{{ if lt $index (add (len $allEntries) -1) }},{{ end }}{{ end }}
]
{{- end }}
rtc:
    tcp_port: {{ .rtc.tcpPort }}
    port_range_start: {{ .rtc.portRangeStart }}
    port_range_end: {{ .rtc.portRangeEnd }}
    use_external_ip: {{ .rtc.useExternalIp }}
    enable_loopback_candidate: {{ .rtc.enableLoopbackCandidate }}
#redis:
    #address: "{{ .redis.address }}"
    #username: "{{ .redis.username }}"
    #password: "{{ .redis.password }}"
    #db: {{ .redis.db }}
    #use_tls: {{ .redis.useTls }}
    #sentinel_master_name: "{{ .redis.sentinelMasterName }}"
    #sentinel_username: "{{ .redis.sentinelUsername }}"
    #sentinel_password: "{{ .redis.sentinelPassword }}"
    #sentinel_addresses: {{ .redis.sentinelAddresses }}
    #cluster_addresses: {{ .redis.clusterAddresses }}
    #max_redirects: {{ .redis.maxRedirects }}
turn:
    enabled: {{ .turn.enabled }}
    #domain: "turn-test.opentalk.run"
    #tls_port: 5349
    #udp_port: 3478
    #external_tls: true
keys:
    {{ $.Values.global.livekit.apiKey }}: "{{ $.Values.global.livekit.apiSecret  }}"
logging:
    json: {{ .logging.json }}
    level: {{ .logging.level }}
{{- end }}
{{- end }}