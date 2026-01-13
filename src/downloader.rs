use anyhow::{Context as _, Result, bail, ensure};
use camino::Utf8Path;
use futures::StreamExt as _;
use tokio::{
    fs::File,
    io::{AsyncWriteExt as _, BufWriter},
    process::Command,
};

/// ffmpegコマンドが利用可能かチェックする
pub async fn check_ffmpeg_available() -> Result<()> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .await
        .context("Failed to execute ffmpeg. Make sure ffmpeg is installed and available in PATH.")?;

    if !output.status.success() {
        bail!("ffmpeg command failed. Make sure ffmpeg is properly installed.");
    }

    Ok(())
}

pub async fn download(post_id: &str, video_url: &str, output: impl AsRef<Utf8Path>) -> Result<()> {
    tracing::info!(post_id, video_url, "Download has started.");

    let output = output.as_ref().join(post_id).with_extension("mp4");
    if output.exists() {
        tracing::info!(post_id, video_url, "exists.");
        return Ok(());
    }

    let temp = output.with_added_extension("tmp");

    // ダウンロード処理を実行し、失敗時には一時ファイルをクリーンアップ
    let result = async {
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
    .await;

    // エラーが発生した場合、一時ファイルが残っていればクリーンアップ
    if result.is_err() && temp.exists() {
        if let Err(e) = tokio::fs::remove_file(&temp).await {
            tracing::warn!(post_id, video_url, error = %e, "Failed to remove temporary file.");
        }
    }

    result
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
