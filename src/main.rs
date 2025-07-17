mod course;
mod metextbook;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // list_apps();
    let app_xxyy = get_window_bounds("Finder").unwrap();

    std::process::Command::new("screencapture")
        .arg("-x")
        .arg("screenshot.png")
        .status()
        .expect("Failed to capture screen");

    crop_window("screenshot.png", app_xxyy, "cut_image.png");

    Ok(())
}

fn get_window_bounds(app: &str) -> Option<(u32, u32, u32, u32)> {
    let script = format!(
        "tell application \"{}\"\n\
            set win to front window\n\
            get bounds of win\n\
        end tell",
        app
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let nums: Vec<u32> = output_str
        .split(", ")
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    if nums.len() == 4 {
        Some((nums[0] * 2, nums[1] * 2, nums[2] * 2, nums[3] * 2))
    } else {
        None
    }
}

use image::imageops::crop_imm;

fn crop_window(image_path: &str, bounds: (u32, u32, u32, u32), output_path: &str) {
    let (x1, y1, x2, y2) = bounds;
    let img = image::open(image_path).expect("Failed to open image");

    let width = x2 - x1;
    let height = y2 - y1;

    let subimg = crop_imm(&img, x1, y1, width, height).to_image();
    subimg
        .save(output_path)
        .expect("Failed to save cropped image");
}

fn list_apps() {
    let output = std::process::Command::new("mdfind")
        .arg("kMDItemContentType == 'com.apple.application-bundle'")
        .output()
        .expect("failed");

    let apps = String::from_utf8_lossy(&output.stdout);
    for line in apps.lines() {
        println!("{}", line);
    }
}
