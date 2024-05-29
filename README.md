# myfans-downloader

A Tools for download videos from [myfans](https://myfans.jp).

## Installation

```console
cargo install --git https://github.com/ekuinox/myfans-downloader
```

**And it requires `ffmpeg` command in `$PATH` env var.**

## Usage

Get token from browser network inspector.

![Get token](./token.png)

And set token to `$MYFANS_TOKEN` env var.

Open myfans plan and get `plan_id` from url.

![Get plan_id](./plan_id.png)

And start download with below command.

```console
myfans-downloader --plan-id <COPIED_PLAN_ID> --ouput <OUTPUT_DIRECTORY>
```

---

Thanks [FudgeRK/MyfansDownloader](https://github.com/FudgeRK/MyfansDownloader)
