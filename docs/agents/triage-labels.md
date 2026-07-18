# Triage Labels

The engineering skills refer to five canonical triage roles. This table maps each role to the exact GitHub label in `jorgehermo9/dbmd`.

| Canonical role | GitHub label | Meaning |
|---|---|---|
| `needs-triage` | `needs-triage` | A maintainer needs to evaluate the request. |
| `needs-info` | `needs-info` | Progress is waiting on information from the reporter. |
| `ready-for-agent` | `ready-for-agent` | The issue is fully specified and can be implemented by an AFK agent without additional human context. |
| `ready-for-human` | `ready-for-human` | The issue is specified but requires human implementation or judgment. |
| `wontfix` | `wontfix` | The request will not be actioned. |

When a skill names a canonical role, apply the corresponding GitHub label exactly. Do not create synonyms such as `triage`, `needs-details`, or `agent-ready`.

Only one active workflow-state label should normally be present. Remove the previous state label when advancing an issue, while preserving unrelated type or area labels.
