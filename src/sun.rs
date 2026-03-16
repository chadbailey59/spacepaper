use std::io::Read;
use std::time::Duration;

use anyhow::{Context, Result};

const TIMEOUT: Duration = Duration::from_secs(30);

/// SOHO EIT 171Å — Fe IX/X coronal loops, 1024x1024 JPEG.
const SOHO_URL: &str = "https://soho.nascom.nasa.gov/data/realtime/eit_171/1024/latest.jpg";

/// Fetch the latest full-disk sun image from SOHO EIT (171Å, 1024x1024).
///
/// Returns raw JPEG bytes.
pub fn fetch_sun_image() -> Result<Vec<u8>> {
    log::info!("Fetching sun image (1024px, EIT 171Å) from SOHO...");

    let resp = ureq::get(SOHO_URL)
        .timeout(TIMEOUT)
        .call()
        .context("Failed to download SOHO sun image")?;

    let len: usize = resp
        .header("Content-Length")
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    let mut data = Vec::with_capacity(len.max(1024));
    resp.into_reader()
        .read_to_end(&mut data)
        .context("Failed to read sun image data")?;

    log::info!("Sun image downloaded: {} KiB", data.len() / 1024);

    Ok(data)
}

/// Recolor a blue EIT 171Å image to the classic gold/orange look.
///
/// The EIT 171 image is predominantly blue. We extract luminance and
/// remap through a gold color ramp similar to NASA's AIA 171 palette.
pub fn recolor_gold(pixels: &mut [u8]) {
    for px in pixels.chunks_exact_mut(3) {
        // Luminance from the blue-dominant EIT image
        let lum = (0.1 * px[0] as f32 + 0.15 * px[1] as f32 + 0.75 * px[2] as f32).min(255.0);
        let t = lum / 255.0;
        // Gold ramp: dark brown → orange → bright yellow-white
        px[0] = (t.powf(0.6) * 255.0).min(255.0) as u8;           // R
        px[1] = (t.powf(0.9) * 200.0).min(255.0) as u8;           // G
        px[2] = (t.powf(1.8) * 80.0).min(255.0) as u8;            // B
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_sun_image() -> Result<()> {
        match fetch_sun_image() {
            Ok(data) => {
                assert!(data.len() > 1000, "Sun image too small: {} bytes", data.len());
                assert_eq!(&data[..2], &[0xFF, 0xD8], "Not a JPEG");
                eprintln!("Sun image size: {} KiB", data.len() / 1024);
            }
            Err(e) => {
                // SOHO server can be unreachable — don't fail the test suite
                eprintln!("SOHO unavailable (expected if NASA is down): {e}");
            }
        }
        Ok(())
    }
}
