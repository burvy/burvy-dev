# Ports the server PC needs open

`build.ps1 -Deploy` copies this file into `deploy\`. It lists every backend in the deploy
folder, the port it listens on, and whether that port must be opened in the Windows firewall
and forwarded on the router. Ports are read from the code (file:line, paths relative to
`Z:\3. Rust Projects`), not from memory. To change one, edit this file in the burvy-dev repo.

## Summary

| Backend | Exe | Port | Proto | Purpose | Open in firewall? |
|---|---|---|---|---|---|
| sandbags | `sandbags\` bundle (`sandbags-server.exe`) | 5101 | **UDP** | game (WebTransport/QUIC), `sandbags.burvy.dev` | **yes**: browsers connect straight to it (DNS only, no Cloudflare proxy). Rule `Sandbags` exists already |
| amity | `amity-server.exe` | 5001 | **UDP** | game (WebTransport/QUIC) | **yes**: browsers connect straight to the home IP |
| floret | `floret-server.exe` | 5002 | **UDP** | game (WebTransport/QUIC) | **yes**: browsers connect straight to the home IP |
| chat | `server.exe` | 4433 | **UDP** | chat rooms (WebTransport/QUIC), `webtrans.burvy.dev` | **yes**: browsers connect straight to it |
| plgroup | `plgroup-server.exe` | 3120 | TCP (https) | `plgroup-api.burvy.dev` API | **yes, Cloudflare only**: Cloudflare's Origin Rule sends it straight to 3120; nginx does not proxy it |
| primer | `primer-server.exe` | 3140 | TCP (https) | `primer-api.burvy.dev` API (hosted mode) | **yes, Cloudflare only**, if you run it hosted (`PRIMER_ADDR=0.0.0.0:3140`). Default `127.0.0.1:3000` = loopback, nothing to open |
| nginx | (not in this folder) | 80 | TCP | the sites over http, and ACME http-01 challenges for every certificate | **yes**: confirmed listening (`listen 80` blocks for tether and sandbags, server-PC report 2026-11-09) |
| nginx | (not in this folder) | 443 | TCP | https for the sites | **assumed, not confirmed**: no nginx.conf in the repos, and the report only shows port-80 blocks. burvy.dev's Cloudflare zone default is **Flexible** (`burvy-dev/README.md:116`), so Cloudflare reaches the origin over http on 80; 443 is only needed for a host set to Full. Check `nginx.conf` for a `listen 443` before opening it |

Not in this deploy: **rambl** (TCP 3160, loopback by default, reached through a Cloudflare
tunnel: nothing to open) and **statesmc** (a static site served by nginx, no port of its own).

The four QUIC servers need **UDP** rules: a TCP rule does nothing for them. Forward the same
ports on the router to the server PC: UDP 4433, 5001, 5002, 5101; TCP 80, 3120, 3140 (443 only
if nginx really listens on it).

## Per backend

### sandbags: `sandbags-server.exe` (sandbags/server)
- **UDP 5101**, WebTransport via lightyear. 5101 so it runs beside amity (5001).
- `sandbags/protocol/src/net/config.rs:44` (`SERVER_BIND_ADDR` = 0.0.0.0:5101); clients dial
  `https://sandbags.burvy.dev:5101` (`server_url()`, hostname from `SB_SERVER_HOST` at build time;
  the port is fixed). No other listener in `sandbags/server`.
- Already set up on the server PC (task 274) as one port rule, so no build's exe path needs its own:
  `New-NetFirewallRule -DisplayName 'Sandbags' -Direction Inbound -Protocol UDP -LocalPort 5101 -Action Allow -Profile Any`
- Deploy it with `sandbags\scripts\update.ps1` (see `sandbags\` and sandbags' `tools/deploy/windows/README.md`).

### amity: `amity-server.exe` (amity/server)
- **UDP 5001**, WebTransport via lightyear.
- `amity/protocol/src/net/config.rs:17` (`SERVER_BIND_ADDR` = 0.0.0.0:5001); release clients
  dial 174.175.161.63:5001 (`:12`). Changing it means rebuilding server and clients.
- `New-NetFirewallRule -DisplayName 'amity (WebTransport)' -Direction Inbound -Protocol UDP -LocalPort 5001 -Action Allow`

### floret: `floret-server.exe` (floret/server)
- **UDP 5002**, WebTransport via lightyear (5001 is amity's).
- `floret/protocol/src/protocol.rs:28` (`const PORT: u16 = 5002`), bound on 0.0.0.0 at `:41`.
- `New-NetFirewallRule -DisplayName 'floret (WebTransport)' -Direction Inbound -Protocol UDP -LocalPort 5002 -Action Allow`

### chat: `server.exe` (burvy-dev/server)
- **UDP 4433**, WebTransport (wtransport).
- `burvy-dev/server/src/main.rs:11` (`const PORT: u16 = 4433`), bound on all interfaces with
  `with_bind_default(PORT)`. Certificate `C:\burvy\certs\webtrans.burvy.dev\`.
- `New-NetFirewallRule -DisplayName 'burvy chat (WebTransport)' -Direction Inbound -Protocol UDP -LocalPort 4433 -Action Allow`

### plgroup: `plgroup-server.exe` (burvy-dev/plgroup-server)
- **TCP 3120**, https itself (Cloudflare origin certificate, `C:\burvy\certs\plgroup-api.burvy.dev\`).
- `burvy-dev/plgroup-server/src/main.rs:20` (`const PORT: u16 = 3120`), bound on 0.0.0.0 at `:111`.
- Reached only through Cloudflare: the proxied CNAME `plgroup-api` with the Origin Rule
  "port 3120" (`burvy-dev/README.md:115`). nginx is not in front of it, so the port must be
  open, but only Cloudflare needs to reach it: limit it with `-RemoteAddress` and
  Cloudflare's ranges (https://www.cloudflare.com/ips/).
- `New-NetFirewallRule -DisplayName 'plgroup API (Cloudflare)' -Direction Inbound -Protocol TCP -LocalPort 3120 -Action Allow`

### primer: `primer-server.exe` (primer/crates/server)
- `PRIMER_ADDR` defaults to `127.0.0.1:3000` (`primer/crates/server/src/main.rs:35`): loopback,
  nothing to open.
- Hosted inside burvy.dev (`primer/docs/DEPLOY.md` part J, `burvy-dev/README.md:213-225`):
  `PRIMER_ADDR=0.0.0.0:3140` plus `PRIMER_TLS_CERT`/`PRIMER_TLS_KEY`; Cloudflare's Origin Rule
  for `primer-api.burvy.dev` points at 3140, no nginx. Open **TCP 3140**, Cloudflare only as for plgroup.
- (The other documented setup, standalone behind Caddy on TCP 3130, isn't part of this bundle;
  its own script adds the rule `Primer (Cloudflare only)`.)
- `New-NetFirewallRule -DisplayName 'primer API (Cloudflare)' -Direction Inbound -Protocol TCP -LocalPort 3140 -Action Allow`

### nginx: the sites
- **TCP 80**: the sites over http (Cloudflare Flexible connects to the origin on 80) and the
  ACME http-01 challenges from `C:\acme-challenge` (win-acme for tether and sandbags; Posh-ACME
  for the amity/floret IP certificate, `amity/server/src/connection/config.rs:1-11`). Must stay
  reachable from the internet or renewals fail.
- **TCP 443**: see the summary; open it only if `nginx.conf` has a `listen 443`.
- `New-NetFirewallRule -DisplayName 'nginx http' -Direction Inbound -Protocol TCP -LocalPort 80 -Action Allow`

## Notes
- The release clients of amity and floret dial the home IP 174.175.161.63 directly; a new IP
  means a rebuild. Sandbags dials the hostname, so an IP change is a DNS edit.
- Dev-local builds bind the same ports on all interfaces: don't run one beside the release exe.
