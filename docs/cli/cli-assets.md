# Assets API

## Upload Batch

Create assets from files, recursive directories, or quoted globs with a
personal Open Cloud API key. Select `--creator-type user` or `group` and the
respective numeric ID; a group upload requires the key owner's group permissions
as well as the appropriate Assets scope. See [API Key](cli-api-key.md).

```sh
rbxcloud assets upload-batch --creator-type group --creator-id 1234 \
  --asset-type image --dry-run 'icons/**/*.png'
```

Dry run hashes the files and prints the plan without loading credentials,
accessing the manifest, or contacting Roblox. Supported types: `image` and
`decal` (PNG, JPG/JPEG, BMP, TGA), `audio` (MP3, OGG, WAV, FLAC), `model`
(FBX, GLTF, GLB). Explicit files must match the chosen type; directories and
globs skip other extensions. Files must be 1–20,000,000 bytes. Display names
come from file stems. Upload creation specifies a maximum fee of zero Robux.

Remove `--dry-run` to create. `--concurrency` defaults to 2 and accepts 1–8.
Keep the same `--manifest` (default `.rbxcloud/upload-batch.json`) to resume.
Keep its sibling `.lock` file in place; the OS releases the lock on exit.
The manifest identifies each upload by SHA-256, creator type and ID, and asset
type. Different filenames with the same bytes share one upload for that target.
Use a single manifest per batch scope, including across concurrent invocations.
Consider ignoring the manifest/lock in Git if IDs should stay local.

- `created`: reuse the recorded ID; don't create or update a remote asset again.
- `accepted`: resume by GET against the known operation ID. If still pending or
  unavailable, run again with the same manifest.
- `failed`: Roblox returned a documented rejection or terminal operation failure;
  review it before making a new attempt.
- `unconfirmed`: the POST might have been accepted, but there is no trusted
  operation ID. No documented SHA-256 lookup or idempotency key is available.
  Check Creator Dashboard manually; the CLI won't guess, create another copy,
  or overwrite an existing asset.

An unresolved batch exits nonzero; inspect the manifest for recorded IDs.
Deleting or choosing a different manifest loses deduplication history.

## Get
Get information on a given asset.
```
Usage: rbxcloud assets get [OPTIONS] --asset-id <ASSET_ID> --api-key <API_KEY>

Options:
  -i, --asset-id <ASSET_ID>    Asset ID
  -m, --read-mask <READ_MASK>
  -a, --api-key <API_KEY>      Roblox Open Cloud API Key [env: RBXCLOUD_API_KEY=]
  -h, --help                   Print help
```

## Create
Create a new asset.
```
Usage: rbxcloud assets create [OPTIONS] --display-name <DISPLAY_NAME> --description <DESCRIPTION> --creator-id <CREATOR_ID> --creator-type <CREATOR_TYPE> --filepath <FILEPATH> --api-key <API_KEY>

Options:
  -t, --asset-type <ASSET_TYPE>
          Asset type [possible values: audio-mp3, audio-ogg, audio-flac, audio-wav, decal-png, decal-jpeg, decal-bmp, decal-tga, model-fbx]
  -n, --display-name <DISPLAY_NAME>
          Display name
  -d, --description <DESCRIPTION>
          Description
  -e, --expected-price <EXPECTED_PRICE>
          Expected Robux price
  -i, --creator-id <CREATOR_ID>
          Creator ID
  -c, --creator-type <CREATOR_TYPE>
          Creator type [possible values: user, group]
  -f, --filepath <FILEPATH>
          File (full or relative path)
  -a, --api-key <API_KEY>
          Roblox Open Cloud API Key [env: RBXCLOUD_API_KEY=]
  -h, --help
          Print help
```

## Update
Update an asset.
```
Usage: rbxcloud assets update [OPTIONS] --asset-id <ASSET_ID> --filepath <FILEPATH> --api-key <API_KEY>

Options:
  -t, --asset-type <ASSET_TYPE>  Asset type [possible values: audio-mp3, audio-ogg, audio-flac, audio-wav, decal-png, decal-jpeg, decal-bmp, decal-tga, model-fbx]
  -i, --asset-id <ASSET_ID>      Asset ID
  -f, --filepath <FILEPATH>      File (full or relative path)
  -a, --api-key <API_KEY>        Roblox Open Cloud API Key [env: RBXCLOUD_API_KEY=]
  -h, --help                     Print help
```

## Archive
Archive an asset.
```
Usage: rbxcloud assets archive --asset-id <ASSET_ID> --api-key <API_KEY>

Options:
  -i, --asset-id <ASSET_ID>  Asset ID
  -a, --api-key <API_KEY>    Roblox Open Cloud API Key [env: RBXCLOUD_API_KEY=]
  -h, --help                 Print help
```

## Restore
Restore an archived asset.
```
Usage: rbxcloud assets restore --asset-id <ASSET_ID> --api-key <API_KEY>

Options:
  -i, --asset-id <ASSET_ID>  Asset ID
  -a, --api-key <API_KEY>    Roblox Open Cloud API Key [env: RBXCLOUD_API_KEY=]
  -h, --help                 Print help
```

## Get Operation
Get information on an asset operation.
```
Usage: rbxcloud assets get-operation --operation-id <OPERATION_ID> --api-key <API_KEY>

Options:
  -i, --operation-id <OPERATION_ID>  Operation ID
  -a, --api-key <API_KEY>            Roblox Open Cloud API Key [env: RBXCLOUD_API_KEY=]
  -h, --help                         Print help

```
