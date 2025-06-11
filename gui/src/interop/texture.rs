use std::path::PathBuf;

use egui::ColorImage;

pub fn spawn_load_texture_thread(
    path: PathBuf,
    id: String,
    size: u8,
    tx: std::sync::mpsc::Sender<(String, ColorImage, u8)>,
) {
    std::thread::spawn(move || {
        match std::fs::read(&path) {
            Ok(bytes) => {
                match image::load_from_memory_with_format(&bytes, image::ImageFormat::Avif)
                    .map(|i| i.to_rgba8())
                {
                    Ok(img) => {
                        let (w, h) = img.dimensions();
                        let ci = ColorImage::from_rgba_unmultiplied(
                            [w as usize, h as usize],
                            &img.into_raw(),
                        );
                        tx.send((id.to_string(), ci, size)).unwrap();
                    }
                    Err(error) => {
                        tracing::warn!(?error, ?path, "unable to load texture")
                    }
                }
            }
            Err(error) => {
                tracing::warn!(?error, ?path, "unable to read texture")
            }
        };
    });
}
