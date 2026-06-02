# Production Deployment Lessons

Use this note when deploying or debugging the production API, object storage, edge router, or Mini Program networking setup. Keep every example sanitized: use placeholders such as `<api-host>`, `<assets-host>`, `<edge-network>`, `<backend-network>`, `<existing-data-volume>`, and `<admin-token>`.

## Safety Rules

- Never record real hostnames, public IP addresses, access keys, passwords, tokens, bucket credentials, database URLs, or private config values in `.agent/`.
- Treat server-local files such as `.env`, `config.yaml`, real compose overrides, local backups, and generated HTML as private operational state. Inspect them only when necessary and redact values in notes.
- Do not commit server-local production overrides unless the user explicitly asks and the file is already intended to be versioned.
- Preserve unrelated dirty worktree changes, especially client work in other platforms.

## Compose Project Identity

- When running Docker Compose on a production server, always confirm the compose project name before `up`, `down`, or `rm`.
- If production containers were created with a specific project name, pass that project name explicitly with `-p <project-name>` for later maintenance commands.
- Running compose from a nested directory without `-p` can create a second set of containers and anonymous/default volumes. If that happens, stop and remove only the accidental containers after confirming they are not serving traffic and do not hold the intended data.

## Existing Data Volumes

- Before replacing production compose files, inspect the currently mounted data volumes for the database, cache, and object store.
- When production already has data, use an override that maps compose volumes to the existing external volume names. Do not let compose silently create fresh empty volumes.
- Verify volume mounts on the live containers after deployment. Object storage can look healthy while serving a fresh empty data volume if the wrong volume is attached.
- Deploy the production API stack through `infra/production/api/compose-from-config.sh`, not a raw `docker compose -f docker-compose.yml up` command.
- Keep `docker-compose.production-local.override.yml` server-local and require it to pin `postgres-data`, `redis-data`, and `minio-data` as external named volumes before any production `up`, `create`, `start`, or `run`.
- Use the unpinned-volume bypass only for a user-confirmed first production bootstrap; never use it for routine image rollouts or service recreates.

## Edge Router Labels

- Production edge routers may use local entrypoint and certificate resolver names that differ from checked-in examples. Confirm the live names from the edge router configuration before declaring a router healthy.
- Prefer self-contained Docker labels for service-specific middlewares when possible. A router that references a missing file-provider middleware can disappear or return router-level 404s even when the backend service and data are healthy.
- If a public asset request returns 404 with no upstream service recorded in edge access logs, suspect router configuration before suspecting missing object data.
- If edge logs show an upstream service and an upstream status, debug the service or storage layer instead.

## Object Storage Delivery

- Distinguish these cases:
  - API returns media URLs correctly, but the asset host returns router-level 404: edge router or middleware issue.
  - Object files exist in the object-store data volume, but public URLs 404: edge router, bucket policy, or public base URL issue.
  - API media records exist, but object files are absent: media upload or data volume issue.
- Validate both an image and an animated media object with `HEAD` or a small `GET` after fixing asset routing.
- Re-check the API health endpoint after changing object storage or edge routing, even when only asset delivery changed.

## Mini Program Domains

- Do not disable Mini Program URL/domain checks to hide production networking problems.
- Keep local project URL checks enabled unless the user explicitly asks for a temporary local-only debugging exception.
- Configure the Mini Program platform allowlist for every runtime capability that uses the domain. For example, the API host belongs in request domains, while the asset host must also be allowed for file download when the app uses `downloadFile` for offline media caching.
- If direct image rendering works but `downloadFile` fails with a domain allowlist error, the asset host is missing from the download allowlist, not necessarily broken at the HTTP layer.

## Verification Checklist

- Confirm the current server checkout commit and branch before deploying.
- Run compose config validation on the production server using the same env file, compose files, override files, and project name used for deployment.
- Inspect live container labels after recreate to verify entrypoints, resolvers, router rules, middlewares, service ports, project name, and network selection.
- Verify the API health endpoint.
- Verify one representative public asset URL.
- Verify edge logs no longer show router-level 404s for the fixed host.
- Confirm no accidental temporary containers remain after deployment.
