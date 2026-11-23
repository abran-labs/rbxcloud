# API Key
`rbxcloud` requires a Roblox API key to function. These keys can be created and managed on the [Credentials](https://create.roblox.com/credentials) Roblox page. Remember to always keep these keys secret.

Roblox API keys will auto-expire if not used for over 60 days. Use the Credentials page to regenerate the key if this occurs.

## Security
As a general rule of practice for any API key:

- Use the least number of permissions as needed
- Keep the key in a secret location (e.g. GitHub Secrets, AWS Secrets Manager, etc.)
- Never commit a key to a source code repository
- Use as strict of a CIDR as possible
- Use an expiration date to force periodic key rollovers (must be manually regenerated)

### Handling Compromised Keys
If a key is suspected to have been compromised, use the Credentials page to immediately invalidate the key. This can be done by regenerating or deleting the key. Keys can also be disabled, but re-enabling the key will not change the key, thus is not a safe option to protect from a compromised key. If in doubt, regenerate the key.

### Storing Keys
There are many tools that can be used to securely store a key. GitHub has a Secrets page for each repository, which can then be used by GitHub Actions securely. AWS and GCP have a Secrets Manager service. Azure has the Key Vault service.

If a key must be located within a local repository's directory, be sure to add it to the `.gitignore` file. This is common in some `.env` file setups.

## Environment Variable
Typed commands such as `assets upload-batch` and `workflow run` load
`ROBLOX_OPEN_CLOUD_API_KEY`, then `RBXCLOUD_API_KEY`, then either variable in
`~/.config/opencode/secrets/roblox-open-cloud.env`. The file must be regular, not
a symlink, and owner-only (`chmod 600` on Unix). Inject keys through your secret
manager; don't put them in command arguments, TOML, or logs.

`rbxcloud auth status` uses the same loader and never contacts Roblox. A passing
check does not verify key scope, validity, or permission to create group assets.
Some older commands still expect `RBXCLOUD_API_KEY`; check the command's `--help`.
