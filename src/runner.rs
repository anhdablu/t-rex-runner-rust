use macroquad::audio::{play_sound_once, Sound};
use macroquad::prelude::*;
use macroquad::prelude::*;
use macroquad::rand::gen_range;

use crate::entities::{Trex, GameOverPanel, TrexStatus};
use crate::entities::check_for_collision;
use crate::environment::{DistanceMeter, Horizon};


// ==========================================
// Constants (Replaces var DEFAULT_WIDTH, FPS, etc.)
// ==========================================
pub const DEFAULT_WIDTH: f32 = 600.0;
pub const DEFAULT_HEIGHT: f32 = 150.0;

// ==========================================
// 1. Key Mapping Helpers (Replaces Runner.keycodes)
// ==========================================
pub fn is_jump_pressed() -> bool {
    is_key_pressed(KeyCode::Space)
        || is_key_pressed(KeyCode::Up)
        || is_mouse_button_pressed(MouseButton::Left)
}

pub fn is_jump_released() -> bool {
    is_key_released(KeyCode::Space)
        || is_key_released(KeyCode::Up)
        || is_mouse_button_released(MouseButton::Left)
}

pub fn is_duck_down() -> bool {
    is_key_down(KeyCode::Down)
}

pub fn is_duck_released() -> bool {
    is_key_released(KeyCode::Down)
}

pub fn is_restart_pressed() -> bool {
    is_key_pressed(KeyCode::Enter)
        || is_key_pressed(KeyCode::Space)
        || is_mouse_button_pressed(MouseButton::Left)
}

/// Inclusive random integer in [min, max], matching JS `getRandomNum(min, max)`
pub fn get_random_num(min: i32, max: i32) -> i32 {
    gen_range(min, max + 1)
}

// ==========================================
// 2. Type-Safe Config Updates (Replaces updateConfigSetting)
// ==========================================
#[derive(Clone, Copy, Debug)]
pub enum ConfigSetting {
    Gravity(f32),
    MinJumpHeight(f32),
    SpeedDropCoefficient(f32),
    InitialJumpVelocity(f32),
    Speed(f32),
}

// ==========================================
// Configuration (Replaces Runner.config)
// ==========================================
#[derive(Clone, Debug)]
pub struct RunnerConfig {
    pub acceleration: f32,
    pub bg_cloud_speed: f32,
    pub bottom_pad: f32,
    pub clear_time: f32,
    pub cloud_frequency: f32,
    pub gameover_clear_time: f32,
    pub gap_coefficient: f32,
    pub gravity: f32,
    pub initial_jump_velocity: f32,
    pub invert_fade_duration: f32,
    pub invert_distance: f32,
    pub max_blink_count: u32,
    pub max_clouds: usize,
    pub max_obstacle_length: usize,
    pub max_obstacle_duplication: usize,
    pub max_speed: f32,
    pub min_jump_height: f32,
    pub mobile_speed_coefficient: f32,
    pub speed: f32,
    pub speed_drop_coefficient: f32,
    pub arcade_mode_initial_top_position: f32,
    pub arcade_mode_top_position_percent: f32,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            acceleration: 0.001,
            bg_cloud_speed: 0.2,
            bottom_pad: 10.0,
            clear_time: 3000.0,
            cloud_frequency: 0.5,
            gameover_clear_time: 750.0,
            gap_coefficient: 0.6,
            gravity: 0.6,
            initial_jump_velocity: 12.0,
            invert_fade_duration: 12000.0,
            invert_distance: 700.0,
            max_blink_count: 3,
            max_clouds: 6,
            max_obstacle_length: 3,
            max_obstacle_duplication: 2,
            max_speed: 13.0,
            min_jump_height: 35.0,
            mobile_speed_coefficient: 1.2,
            speed: 6.0,
            speed_drop_coefficient: 3.0,
            arcade_mode_initial_top_position: 35.0,
            arcade_mode_top_position_percent: 0.1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteDefinition {
    pub cactus_large: Vec2,
    pub cactus_small: Vec2,
    pub cloud: Vec2,
    pub horizon: Vec2,
    pub moon: Vec2,
    pub pterodactyl: Vec2,
    pub restart: Vec2,
    pub text_sprite: Vec2,
    pub trex: Vec2,
    pub star: Vec2,
}

impl SpriteDefinition {
    pub const LDPI: Self = Self {
        cactus_large: vec2(332.0, 2.0),
        cactus_small: vec2(228.0, 2.0),
        cloud: vec2(86.0, 2.0),
        horizon: vec2(2.0, 54.0),
        moon: vec2(484.0, 2.0),
        pterodactyl: vec2(134.0, 2.0),
        restart: vec2(2.0, 2.0),
        text_sprite: vec2(655.0, 2.0),
        trex: vec2(848.0, 2.0),
        star: vec2(645.0, 2.0),
    };

    pub const HDPI: Self = Self {
        cactus_large: vec2(652.0, 2.0),
        cactus_small: vec2(446.0, 2.0),
        cloud: vec2(166.0, 2.0),
        horizon: vec2(2.0, 104.0),
        moon: vec2(954.0, 2.0),
        pterodactyl: vec2(260.0, 2.0),
        restart: vec2(2.0, 2.0),
        text_sprite: vec2(1294.0, 2.0),
        trex: vec2(1678.0, 2.0),
        star: vec2(1276.0, 2.0),
    };

    /// Helper to grab the right coordinates based on DPI
    pub fn get(is_hidpi: bool) -> Self {
        if is_hidpi {
            Self::HDPI
        } else {
            Self::LDPI
        }
    }
}

// ==========================================
// Runner Struct (Replaces function Runner)
// ==========================================
pub struct Runner {
    pub config: RunnerConfig,
    pub dimensions: Vec2,
    pub sprite_def: SpriteDefinition,

    // Entities & Environment (from your other modules)
    // pub t_rex: Trex,
    // pub distance_meter: DistanceMeter,
    // pub horizon: Horizon,

    // Game Progress & Timing
    pub distance_ran: f32,
    pub highest_score: u32,
    pub time: f32,
    pub running_time: f32,
    pub current_speed: f32,

    // State Flags
    pub activated: bool, // Whether the easter egg has been activated
    pub playing: bool,   // Whether the game is currently in play state
    pub crashed: bool,
    pub paused: bool,
    pub inverted: bool,
    pub invert_timer: f32,
    pub play_count: u32,

    // Assets (Passed in from main.rs)
    pub sprite_1x: Texture2D,
    pub sprite_2x: Texture2D,
    pub is_hidpi: bool,

    // Audio (Made pub so main.rs can access runner.press_sound)
    pub press_sound: Option<Sound>,
    pub hit_sound: Option<Sound>,
    pub reach_sound: Option<Sound>,

    pub playing_intro: bool,
    pub intro_timer: f32, // Tracks the 0.4s (400ms) intro expansion
    pub container_width: f32, // Replaces CSS width animation (44.0 -> 600.0)
    pub game_over_panel: GameOverPanel,
}

impl Runner {
    pub fn new(
        sprite_1x: Texture2D,
        sprite_2x: Texture2D,
        press_sound: Option<Sound>,
        hit_sound: Option<Sound>,
        reach_sound: Option<Sound>,
    ) -> Self {
        let config = RunnerConfig::default();
        let initial_speed = config.speed;
        let dimensions = vec2(DEFAULT_WIDTH, DEFAULT_HEIGHT);

        let is_hidpi = screen_dpi_scale() > 1.0;
        let sprite_def = SpriteDefinition::get(is_hidpi);

        // Initialize entities (Uncomment as you migrate each module)
        let horizon = Horizon::new(sprite_def, dimensions, config.gap_coefficient);
        let distance_meter = DistanceMeter::new(sprite_def.text_sprite, dimensions.x);
        let t_rex = Trex::new(sprite_def.trex);

        let mut runner = Self {
            config,
            dimensions,
            sprite_def,
            // horizon,
            // distance_meter,
            // t_rex,
            distance_ran: 0.0,
            highest_score: 0,
            time: 0.0,
            running_time: 0.0,
            current_speed: initial_speed,
            activated: false,
            playing: false,
            crashed: false,
            paused: false,
            inverted: false,
            invert_timer: 0.0,
            play_count: 0,
            sprite_1x,
            sprite_2x,
            is_hidpi,
            press_sound,
            hit_sound,
            reach_sound,
        };

        let game_over_panel = GameOverPanel::new(
            sprite_def.text_sprite,
            sprite_def.restart,
            dimensions,
        );

        // Matches `this.setSpeed();` at the start of `init()`
        runner.set_speed(None);
        runner
    }

    /// Replaces `updateConfigSetting(setting, value)`
    pub fn update_config_setting(&mut self, setting: ConfigSetting) {
        match setting {
            ConfigSetting::Gravity(val) => {
                self.config.gravity = val;
                // self.t_rex.config.gravity = val;
            }
            ConfigSetting::MinJumpHeight(val) => {
                self.config.min_jump_height = val;
                // self.t_rex.config.min_jump_height = val;
            }
            ConfigSetting::SpeedDropCoefficient(val) => {
                self.config.speed_drop_coefficient = val;
                // self.t_rex.config.speed_drop_coefficient = val;
            }
            ConfigSetting::InitialJumpVelocity(val) => {
                self.config.initial_jump_velocity = val;
                // self.t_rex.set_jump_velocity(val);
            }
            ConfigSetting::Speed(val) => {
                self.config.speed = val;
                self.set_speed(Some(val));
            }
        }
    }

    /// Helper to get the active sprite sheet based on HiDPI
    /// (Replaces the texture selection in `loadImages()`)
    pub fn active_sprite(&self) -> &Texture2D {
        if self.is_hidpi {
            &self.sprite_2x
        } else {
            &self.sprite_1x
        }
    }

    /// Replaces `Runner.prototype.setSpeed(opt_speed)`
    pub fn set_speed(&mut self, opt_speed: Option<f32>) {
        let speed = opt_speed.unwrap_or(self.current_speed);

        // Reduce the speed on smaller screens (kept for exact parity with JS)
        if self.dimensions.x < DEFAULT_WIDTH {
            let mobile_speed = speed * self.dimensions.x / DEFAULT_WIDTH
                * self.config.mobile_speed_coefficient;
            self.current_speed = mobile_speed.min(speed);
        } else if let Some(new_speed) = opt_speed {
            self.current_speed = new_speed;
        }
    }

    /// Replaces `Runner.prototype.playIntro()`
    pub fn play_intro(&mut self) {
        if !self.activated && !self.crashed {
            self.playing_intro = true;
            self.intro_timer = 0.0;
            self.container_width = 44.0; // Trex.config.WIDTH

            // self.t_rex.playing_intro = true;

            self.playing = true;
            self.activated = true;
        } else if self.crashed {
            self.restart();
        }
    }

    /// Call this inside `Runner::update(dt)` to replace the CSS `webkitAnimationEnd` listener
    pub fn update_intro(&mut self, delta_ms: f32) {
        if !self.playing_intro {
            return;
        }

        const INTRO_DURATION_MS: f32 = 400.0; // Matches `.4s` in CSS keyframes
        const TREX_WIDTH: f32 = 44.0;

        self.intro_timer += delta_ms;
        let progress = (self.intro_timer / INTRO_DURATION_MS).min(1.0);

        // Ease-out interpolation matching `ease-out` in CSS
        let ease_out = 1.0 - (1.0 - progress).powi(2);
        self.container_width = TREX_WIDTH + (self.dimensions.x - TREX_WIDTH) * ease_out;

        if self.intro_timer >= INTRO_DURATION_MS {
            self.playing_intro = false;
            self.container_width = self.dimensions.x;
            // Replaces `this.containerEl.addEventListener(ANIM_END, this.startGame)`
            self.start_game();
        }
    }

    /// Replaces `Runner.prototype.restart()`
    pub fn restart(&mut self) {
        if !self.playing {
            self.play_count += 1;
            self.running_time = 0.0;
            self.playing = true;
            self.crashed = false;
            self.paused = false;
            self.distance_ran = 0.0;
            self.crash_timer = 0.0;

            self.set_speed(Some(self.config.speed));

            // Reset entities (Uncomment as you migrate entities.rs & environment.rs)
            // self.distance_meter.reset(self.highest_score);
            // self.horizon.reset();
            // self.t_rex.reset();

            self.play_sound(&self.press_sound);
            self.invert(true);
        }
    }

    /// Replaces `Runner.prototype.startGame()`
    pub fn start_game(&mut self) {
        self.running_time = 0.0;
        self.playing_intro = false;
        // self.t_rex.playing_intro = false;
        self.play_count += 1;
    }

    /// Replaces `Runner.prototype.update()`
    /// Call this every frame from `main.rs` with `dt = get_frame_time()`
    pub fn update(&mut self, dt: f32) {
        // Convert seconds to milliseconds to match JS `deltaTime`
        let mut delta_time = dt * 1000.0;
        self.time += delta_time;

        // Advance the 0.4s intro reveal timer if active
        self.update_intro(delta_time);

        if self.playing {
            // 1. Update T-Rex jump physics
            if self.t_rex.jumping {
                self.t_rex.update_jump(delta_time);
            }

            self.running_time += delta_time;
            let has_obstacles = self.running_time > self.config.clear_time;

            // 2. First jump triggers the intro
            if self.t_rex.jump_count == 1 && !self.playing_intro {
                self.play_intro();
            }

            // 3. Move horizon (ground, clouds, obstacles)
            if self.playing_intro {
                // Horizon doesn't move until the intro is over
                self.horizon.update(0.0, self.current_speed, has_obstacles, self.inverted);
            } else {
                if !self.activated {
                    delta_time = 0.0;
                }
                self.horizon.update(delta_time, self.current_speed, has_obstacles, self.inverted);
            }

            // 4. Collision detection
            // let collision = false;
            // let collision = has_obstacles
            //     && self.horizon.obstacles.first().map_or(false, |obs| {
            //         check_for_collision(obs, &self.t_rex)
            //     });

            // if !collision {
            //     self.distance_ran += self.current_speed * delta_time / MS_PER_FRAME;

            //     if self.current_speed < self.config.max_speed {
            //         self.current_speed += self.config.acceleration;
            //     }
            // } else {
            //     self.game_over();
            // }
            let collision = has_obstacles
                && self
                    .horizon
                    .obstacles
                    .first()
                    .and_then(|obs| check_for_collision(obs, &self.t_rex, None))
                    .is_some();

            // 5. Score & achievement sound (every 100 points)
            // let play_achievement_sound = false;
            let play_achievement_sound = self
                .distance_meter
                .update(delta_time, self.distance_ran.ceil() as u32);

            if play_achievement_sound {
                if let Some(ref snd) = self.reach_sound {
                    play_sound_once(snd);
                }
            }

            // 6. Day / Night mode inversion timer
            if self.invert_timer > self.config.invert_fade_duration {
                self.invert_timer = 0.0;
                self.invert_trigger = false;
                self.invert(false);
            } else if self.invert_timer > 0.0 {
                self.invert_timer += delta_time;
            } else {
                // let actual_distance = 0;
                let actual_distance = self
                    .distance_meter
                    .get_actual_distance(self.distance_ran.ceil() as u32);

                if actual_distance > 0 {
                    self.invert_trigger =
                        (actual_distance % (self.config.invert_distance as u32)) == 0;

                    if self.invert_trigger && self.invert_timer == 0.0 {
                        self.invert_timer += delta_time;
                        self.invert(false);
                    }
                }
            }
        }

        // 7. Idle blinking animation before first jump
        if self.playing || (!self.activated && self.t_rex.blink_count < self.config.max_blink_count) {
            self.t_rex.update(delta_time);
        }

        if self.crashed {
            self.crash_timer += delta_time;
        }
    }

    // /// Replaces `onKeyDown` (and `onKeyUp` once we add it next)
    // /// Call this at the very start of `Runner::update(dt)`
    // pub fn handle_input(&mut self) {
    //     // 1. Jump / Start Input (Spacebar, Up Arrow, or Left Click)
    //     if is_jump_pressed() {
    //         if !self.crashed {
    //             if !self.playing {
    //                 // Note: Sounds are already loaded in main.rs, so no need for `loadSounds()`
    //                 self.playing = true;
    //             }

    //             // Play sound effect and jump
    //             if !self.t_rex.jumping && !self.t_rex.ducking {
    //                 if let Some(ref snd) = self.press_sound {
    //                     play_sound_once(snd);
    //                 }
    //                 self.t_rex.start_jump(self.current_speed);
    //             }
    //         }
    //     }

    //     // 2. Duck / Speed-Drop Input (Down Arrow)
    //     if self.playing && !self.crashed && is_duck_down() {
    //         if self.t_rex.jumping {
    //             // Speed drop, activated only when jump key is not pressed
    //             self.t_rex.set_speed_drop();
    //         } else if !self.t_rex.jumping && !self.t_rex.ducking {
    //             // Duck
    //             self.t_rex.set_duck(true);
    //         }
    //     }
    // }

    /// Replaces `Runner.prototype.isRunning()`
    pub fn is_running(&self) -> bool {
        self.playing && !self.paused && !self.crashed
    }

    /// Combined `onKeyDown` + `onKeyUp` handler
    /// Call this at the top of `Runner::update(dt)`
    pub fn handle_input(&mut self) {
        // ==========================================
        // 1. Key Down Checks (Replaces onKeyDown)
        // ==========================================
        if is_jump_pressed() {
            if !self.crashed {
                if !self.playing {
                    self.playing = true;
                }

                if !self.t_rex.jumping && !self.t_rex.ducking {
                    if let Some(ref snd) = self.press_sound {
                        play_sound_once(snd);
                    }
                    self.t_rex.start_jump(self.current_speed);
                }
            }
        }

        if self.playing && !self.crashed && is_duck_down() {
            if self.t_rex.jumping {
                self.t_rex.set_speed_drop();
            } else if !self.t_rex.jumping && !self.t_rex.ducking {
                self.t_rex.set_duck(true);
            }
        }

        // ==========================================
        // 2. Key Up Checks (Replaces onKeyUp)
        // ==========================================
        let jump_released = is_jump_released();
        let duck_released = is_duck_released();

        if self.is_running() && jump_released {
            self.t_rex.end_jump();
        } else if duck_released {
            self.t_rex.speed_drop = false;
            self.t_rex.set_duck(false);
        } else if self.crashed {
            // Enforce GAMEOVER_CLEAR_TIME (750ms) before Space/Click restarts,
            // while Enter restarts immediately
            let enter_released = is_key_released(KeyCode::Enter);
            let can_jump_restart =
                self.crash_timer >= self.config.gameover_clear_time && jump_released;

            if enter_released || can_jump_restart {
                self.restart();
            }
        } else if self.paused && jump_released {
            // Reset the jump state and resume
            self.t_rex.reset();
            self.play();
        }
    }

    /// Replaces `Runner.prototype.gameOver()`
    pub fn game_over(&mut self) {
        if let Some(ref snd) = self.hit_sound {
            play_sound_once(snd);
        }

        self.stop();
        self.crashed = true;
        self.crash_timer = 0.0; // Starts the 750ms restart cooldown

        self.distance_meter.achievement = false;
        self.t_rex.update(100.0, TrexStatus::Crashed);

        // Update high score
        let current_dist = self.distance_ran.ceil() as u32;
        if current_dist > self.highest_score {
            self.highest_score = current_dist;
            self.distance_meter.set_high_score(self.highest_score);
        }
    }

    /// Replaces `Runner.prototype.stop()`
    pub fn stop(&mut self) {
        self.playing = false;
        self.paused = true;
    }

    /// Replaces `Runner.prototype.play()`
    pub fn play(&mut self) {
        if !self.crashed {
            self.playing = true;
            self.paused = false;
            self.t_rex.update(0.0, TrexStatus::Running);
        }
    }

    /// Replaces `Runner.prototype.playSound(soundBuffer)`
    pub fn play_sound(&self, sound: &Option<Sound>) {
        if let Some(ref snd) = sound {
            play_sound_once(snd);
        }
    }

    /// Replaces `Runner.prototype.invert(reset)`
    pub fn invert(&mut self, reset: bool) {
        if reset {
            self.invert_timer = 0.0;
            self.inverted = false;
        } else {
            self.inverted = self.invert_trigger;
        }
    }

    /// Renders all game layers (Horizon, Obstacles, T-Rex, Score, Game Over Panel)
    pub fn draw(&self) {
        // Vertical offset to center the 150px canvas inside your 600x200 window
        let y_offset = 25.0;

        // 1. Draw Horizon (ground line, clouds, night moon/stars, obstacles)
        self.horizon.draw(self.active_sprite(), y_offset, self.inverted);

        // 2. Draw T-Rex
        self.t_rex.draw(self.active_sprite(), y_offset);

        // 3. Draw Distance Meter (Score & HI score)
        self.distance_meter.draw(self.active_sprite(), y_offset);

        // 4. Draw Game Over Panel if crashed
        if self.crashed {
            // if let Some(ref panel) = self.game_over_panel {
            //     panel.draw(self.active_sprite(), y_offset);
            // }
            self.game_over_panel.draw(self.active_sprite(), self.is_hidpi, y_offset);
        }

        // if self.crashed {
        //     self.game_over_panel.draw(self.active_sprite(), self.is_hidpi, y_offset);
        // }

        // 5. Intro curtain-wipe reveal (replaces CSS @keyframes width: 44px -> 600px)
        if self.playing_intro {
            let bg_color = if self.inverted {
                Color::from_rgba(33, 33, 33, 255)
            } else {
                Color::from_rgba(247, 247, 247, 255)
            };
            draw_rectangle(
                self.container_width,
                y_offset,
                self.dimensions.x - self.container_width,
                self.dimensions.y,
                bg_color,
            );
        }
    }

}
