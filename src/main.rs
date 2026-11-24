mod audio_data;
mod entities;
mod environment;
mod runner;

use macroquad::audio::{load_sound_from_bytes, play_sound_once};
use macroquad::prelude::*;
use runner::Runner;

fn window_conf() -> Conf {
    Conf {
        window_title: "T-Rex Runner".to_owned(),
        window_width: 600, // Standard Chrome Runner canvas width
        window_height: 200,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // 1. Validate & load Base64 audio (replaces your useEffect validation loop)
    let raw_audio = audio_data::load_all_audio();

    let press_sound = match raw_audio.press {
        Some(bytes) => load_sound_from_bytes(&bytes).await.ok(),
        None => None,
    };
    let hit_sound = match raw_audio.hit {
        Some(bytes) => load_sound_from_bytes(&bytes).await.ok(),
        None => None,
    };
    let reach_sound = match raw_audio.reach {
        Some(bytes) => load_sound_from_bytes(&bytes).await.ok(),
        None => None,
    };

    // 2. Load 1x and 2x Sprite Sheets (replaces <img id="offline-resources-1x/2x">)
    // `include_bytes!` embeds the PNGs directly into the standalone executable!
    let sprite_1x = Texture2D::from_file_with_format(
        include_bytes!("../assets/100-offline-sprite.png"),
        Some(ImageFormat::Png),
    );
    sprite_1x.set_filter(FilterMode::Nearest); // Keeps pixel art crisp

    let sprite_2x = Texture2D::from_file_with_format(
        include_bytes!("../assets/200-offline-sprite.png"),
        Some(ImageFormat::Png),
    );
    sprite_2x.set_filter(FilterMode::Nearest);

    // 3. Initialize Runner (replaces `new window.Runner('.interstitial-wrapper')`)
    let mut runner = Runner::new(sprite_1x, sprite_2x, press_sound, hit_sound, reach_sound);

    // 4. React `const [isStarted, setIsStarted] = useState(false);`
    let mut is_started = false;

    loop {
        let dt = get_frame_time();
        clear_background(Color::from_rgba(247, 247, 247, 255));

        if !is_started {
            // Replaces `#messageBox` overlay & `handleKeyDown` (evt.keyCode === 32) / `handleOkButtonClick`
            let prompt = "Press Space to start";
            let text_dims = measure_text(prompt, None, 28, 1.0);
            let text_x = (screen_width() - text_dims.width) / 2.0;
            let text_y = screen_height() / 2.0;

            draw_text(prompt, text_x, text_y, 28.0, DARKGRAY);

            // Start on Spacebar or Mouse Click (like clicking .niokbutton)
            if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left) {
                is_started = true;
                if let Some(ref snd) = runner.press_sound {
                    play_sound_once(snd);
                }
            }
        } else {
            // Game is active
            runner.update(dt);
            runner.draw();
        }

        next_frame().await;
    }
}