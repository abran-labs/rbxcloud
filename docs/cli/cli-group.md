# Group API

## Get Group Info
Get information about a group.
```
Usage: rbxcloud group get [OPTIONS] --group-id <GROUP_ID> --api-key <API_KEY>

Options:
  -a, --api-key <API_KEY>    Roblox Open Cloud API Key [env: RBXCLOUD_API_KEY=]
  -g, --group-id <GROUP_ID>  Group ID
  -p, --pretty               Pretty-print the JSON response
  -h, --help                 Print help
```

### Example
```
$ rbxcloud group get -p -g 12345 -a MY_KEY
```

## Get Group Shout
Get a group's current shout and its metadata.
```
Usage: rbxcloud group shout [OPTIONS] --group-id <GROUP_ID> --api-key <API_KEY>

Options:
  -a, --api-key <API_KEY>    Roblox Open Cloud API Key [env: RBXCLOUD_API_KEY=]
  -g, --group-id <GROUP_ID>  Group ID
  -p, --pretty               Pretty-print the JSON response
  -o, --only-message         Only return the shout message string
  -h, --help                 Print help
```

### Example
```
$ rbxcloud group shout -p -g 12345 -a MY_KEY
```

## List Forum Categories
List forum categories for a group.
```
Usage: rbxcloud group forum-categories [OPTIONS] --group-id <GROUP_ID> --api-key <API_KEY>
```

### Example
```
$ rbxcloud group forum-categories -p -g 12345 -a MY_KEY
```

## List Forum Posts
List forum posts for a forum category.
```
Usage: rbxcloud group forum-posts [OPTIONS] --group-id <GROUP_ID> --forum-category-id <FORUM_CATEGORY_ID> --api-key <API_KEY>
```

### Example
```
$ rbxcloud group forum-posts -p -g 12345 --forum-category-id 67890 -a MY_KEY
```

## List Forum Comments
List forum comments for a forum post.
```
Usage: rbxcloud group forum-comments [OPTIONS] --group-id <GROUP_ID> --forum-category-id <FORUM_CATEGORY_ID> --post-id <POST_ID> --api-key <API_KEY>
```

### Example
```
$ rbxcloud group forum-comments -p -g 12345 --forum-category-id 67890 --post-id 24680 -a MY_KEY
```

## List Join Requests
List pending join requests for a group. Supports Roblox Cloud API filters such as `user == 'users/156'`.
```
Usage: rbxcloud group join-requests [OPTIONS] --group-id <GROUP_ID> --api-key <API_KEY>
```

### Example
```
$ rbxcloud group join-requests -p -g 12345 -f "user == 'users/156'" -a MY_KEY
```

## Accept Join Request
Accept a pending group join request.
```
Usage: rbxcloud group accept-join-request [OPTIONS] --group-id <GROUP_ID> --join-request-id <JOIN_REQUEST_ID> --api-key <API_KEY>
```

For normal user join requests, `JOIN_REQUEST_ID` is usually the Roblox user ID.

### Example
```
$ rbxcloud group accept-join-request -p -g 12345 --join-request-id 156 -a MY_KEY
```

## Decline Join Request
Decline a pending group join request.
```
Usage: rbxcloud group decline-join-request [OPTIONS] --group-id <GROUP_ID> --join-request-id <JOIN_REQUEST_ID> --api-key <API_KEY>
```

### Example
```
$ rbxcloud group decline-join-request -p -g 12345 --join-request-id 156 -a MY_KEY
```

## List Group Roles
List the roles of a given group.
```
Usage: rbxcloud group roles [OPTIONS] --group-id <GROUP_ID> --api-key <API_KEY>
```

### Example
```
$ rbxcloud group roles -p -g 12345 -a MY_KEY
```

## Get Group Role
Get one group role by role ID.
```
Usage: rbxcloud group role [OPTIONS] --group-id <GROUP_ID> --role-id <ROLE_ID> --api-key <API_KEY>
```

### Example
```
$ rbxcloud group role -p -g 12345 --role-id 98765 -a MY_KEY
```

## List Group Memberships
List members of the group. For more info on the `--filter` option, see [Roblox's documentation](https://create.roblox.com/docs/cloud/reference/patterns#list-group-memberships).
```
Usage: rbxcloud group memberships [OPTIONS] --group-id <GROUP_ID> --api-key <API_KEY>
```

### Example
```
$ rbxcloud group memberships -p -g 12345 -f "user == 'users/156'" -a MY_KEY
```

## Update Membership
Patch a group membership with a raw JSON request body.
```
Usage: rbxcloud group update-membership [OPTIONS] --group-id <GROUP_ID> --membership-id <MEMBERSHIP_ID> --body <BODY> --api-key <API_KEY>
```

Roblox Cloud docs allow a user ID for membership-specific operations where a membership ID is required.

### Example
```
$ rbxcloud group update-membership -p -g 12345 --membership-id 156 --body '{"role":"groups/12345/roles/98765"}' -a MY_KEY
```

## Assign Role
Assign a role to a group membership.
```
Usage: rbxcloud group assign-role [OPTIONS] --group-id <GROUP_ID> --membership-id <MEMBERSHIP_ID> --role <ROLE> --api-key <API_KEY>
```

`ROLE` must be the full Cloud API role path, e.g. `groups/12345/roles/98765`.

### Example
```
$ rbxcloud group assign-role -p -g 12345 --membership-id 156 --role groups/12345/roles/98765 -a MY_KEY
```

## Unassign Role
Unassign a role from a group membership.
```
Usage: rbxcloud group unassign-role [OPTIONS] --group-id <GROUP_ID> --membership-id <MEMBERSHIP_ID> --role <ROLE> --api-key <API_KEY>
```

### Example
```
$ rbxcloud group unassign-role -p -g 12345 --membership-id 156 --role groups/12345/roles/98765 -a MY_KEY
```
