# SLO

## Disponibilidade da API

- **SLI:** proporção de requisições `/api/*` com status < 500.
- **SLO:** 99,5% em janela de 30 dias.
- **Error budget:** 0,5% das requisições, equivalente a ≈ 3 h 36 min de indisponibilidade total em 30 dias.

Consulta PromQL sobre `http_requests_total` (exposta em `GET /metrics`):

```promql
1 - sum(rate(http_requests_total{route=~"/api/.*",status=~"5.."}[30d]))
  / sum(rate(http_requests_total{route=~"/api/.*"}[30d]))
```

## Latência (não implementado)

Segundo SLO possível: p95 < 300 ms, a partir de `http_request_duration_seconds`.

## Como medir

O `compose.yml` não tem Prometheus. Para verificar o SLO, aponte um Prometheus externo para `app:3000/metrics`.
O rótulo `route` traz o padrão da rota (`/api/books/{id}`), não a URI, para manter a cardinalidade baixa.
