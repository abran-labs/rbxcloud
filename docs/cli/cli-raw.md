# Raw Open Cloud Request

The `raw` command sends an authenticated request to any Roblox Open Cloud endpoint. Use this as an escape hatch for endpoints that do not yet have a typed `rbxcloud` command.

It automatically adds the `x-api-key` header and accepts the key from `RBXCLOUD_API_KEY`.

```
Usage: rbxcloud raw [OPTIONS] --url <URL> --api-key <API_KEY>

Options:
  -m, --method <METHOD>  HTTP method [default: GET]
  -u, --url <URL>        Open Cloud path (e.g. /cloud/v2/groups/123) or full URL
  -b, --body <BODY>      JSON request body
  -q, --query <QUERY>    Query parameter as key=value. Can be repeated
  -p, --pretty           Pretty-print JSON response
  -a, --api-key <KEY>    Roblox Open Cloud API Key [env: RBXCLOUD_API_KEY=]
  -h, --help             Print help
```

## Examples

List group roles:

```
$ rbxcloud raw -p -u /cloud/v2/groups/12345/roles
```

Accept a group join request:

```
$ rbxcloud raw \
  --method POST \
  --url /cloud/v2/groups/12345/join-requests/156:accept \
  --body '{}'
```

Assign a role:

```
$ rbxcloud raw \
  --method POST \
  --url /cloud/v2/groups/12345/memberships/156:assignRole \
  --body '{"role":"groups/12345/roles/98765"}'
```

Use query parameters:

```
$ rbxcloud raw \
  -p \
  -u /cloud/v2/groups/12345/memberships \
  -q "filter=user == 'users/156'" \
  -q maxPageSize=10
```
