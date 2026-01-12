use anyhow::{Context as _, Result, ensure};
use camino::Utf8Path;
use futures::StreamExt as _;
use tokio::{
    fs::File,
    io::{AsyncWriteExt as _, BufWriter},
    process::Command,
};

pub async fn download(post_id: &str, video_url: &str, output: impl AsRef<Utf8Path>) -> Result<()> {
    tracing::info!(post_id, video_url, "Download has started.");

    let output = output.as_ref().join(post_id).with_extension("mp4");
    if output.exists() {
        tracing::info!(post_id, video_url, "exists.");
        return Ok(());
    }

    let temp = output.with_added_extension("tmp");

    if video_url.ends_with(".m3u8") {
        download_m3u8_to_mp4(video_url, &temp).await?;
    } else if video_url.ends_with(".mp4") {
        download_mp4(video_url, &temp).await?;
    } else {
        tracing::info!(post_id, video_url, "Skipped.");
        return Ok(());
    }

    tokio::fs::rename(&temp, &output).await?;

    tracing::info!(post_id, video_url, "Download completed.");

    Ok(())
}

/// `ffmpeg` を呼び出して `.m3u8` を `.mp4` としてダウンロードする
async fn download_m3u8_to_mp4(video_url: &str, saved_to: impl AsRef<Utf8Path>) -> Result<()> {
    let output = Command::new("ffmpeg")
        .args([
            "-i",
            video_url,
            "-c",
            "copy",
            "-bsf:a",
            "aac_adtstoasc",
            "-f",
            "mp4",
            saved_to.as_ref().as_str(),
        ])
        .output()
        .await?;
    ensure!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

/// `.mp4` をそのままダウンロードする
async fn download_mp4(video_url: &str, saved_to: impl AsRef<Utf8Path>) -> Result<()> {
    let mut reader = reqwest::get(video_url)
        .await
        .context("Failed to request.")?
        .error_for_status()?
        .bytes_stream();
    let file = File::create(saved_to.as_ref())
        .await
        .context("Failed to create output file.")?;
    let mut writer = BufWriter::new(file);

    while let Some(chunk) = reader.next().await {
        let chunk = chunk.context("Failed to read stream.")?;
        writer
            .write_all(&chunk)
            .await
            .context("Failed to write to stream.")?;
    }

    writer
        .flush()
        .await
        .context("Failed to flush writer stream.")?;

    Ok(())
}
