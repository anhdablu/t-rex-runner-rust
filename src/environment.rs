// TODO: Migrate DistanceMeter, Cloud, NightMode, HorizonLine, Horizon here

use rand::Rng;
use rand::rng;
use std::collections::VecDeque;
use crate::entities::{Obstacle, ObstacleType};


pub struct HorizonLineDimensions;
pub struct CloudConfig;
pub struct NightModeConfig;
pub struct HorizonConfig;

#[derive(Debug, Clone, Copy)]
pub struct SpritePos {
    pub x: i32,
    pub y: i32,
}

/// Framework-agnostic draw instruction representing a single sprite blit.
#[derive(Debug, Clone, Copy)]
pub struct DrawCall {
    pub src_x: i32,
    pub src_y: i32,
    pub src_w: i32,
    pub src_h: i32,
    pub dst_x: i32,
    pub dst_y: i32,
    pub dst_w: i32,
    pub dst_h: i32,
    pub alpha: f32,
}

pub struct DistanceMeterDimensions;
impl DistanceMeterDimensions {
    pub const WIDTH: i32 = 10;
    pub const HEIGHT: i32 = 13;
    pub const DEST_WIDTH: i32 = 11;
}

pub struct DistanceMeterConfig {
    pub max_distance_units: usize,
    pub achievement_distance: u32,
    pub coefficient: f64,
    pub flash_duration: f64,
    pub flash_iterations: u32,
}

impl Default for DistanceMeterConfig {
    fn default() -> Self {
        Self {
            // Number of digits.
            max_distance_units: 5,
            // Distance that causes achievement animation.
            achievement_distance: 100,
            // Used for conversion from pixel distance to a scaled unit.
            coefficient: 0.025,
            // Flash duration in milliseconds.
            flash_duration: 1000.0 / 4.0,
            // Flash iterations for achievement animation.
            flash_iterations: 3,
        }
    }
}

pub struct DistanceMeter {
    pub sprite_pos: SpritePos,
    pub x: i32,
    pub y: i32,

    pub current_distance: f64,
    pub max_score: u32,
    pub high_score: Vec<u8>,
    pub digits: Vec<u8>,

    pub achievement: bool,
    pub default_string: String,
    pub flash_timer: f64,
    pub flash_iterations: u32,
    pub invert_trigger: bool,

    pub config: DistanceMeterConfig,
    pub max_score_units: usize,
}

impl DistanceMeter {
    /// Y positioning of the digits in the sprite sheet.
    /// X position is always 0.
    // pub const Y_POS: [i32; 10] = ;

    pub fn new(sprite_pos: SpritePos, canvas_width: i32) -> Self {
        let config = DistanceMeterConfig::default();
        let max_score_units = config.max_distance_units;

        let mut meter = Self {
            sprite_pos,
            x: 0,
            y: 5,
            current_distance: 0.0,
            max_score: 0,
            high_score: Vec::new(),
            digits: Vec::new(),
            achievement: false,
            default_string: String::with_capacity(max_score_units),
            flash_timer: 0.0,
            flash_iterations: 0,
            invert_trigger: false,
            config,
            max_score_units,
        };

        meter.init(canvas_width);
        meter
    }

    /// Initialise the distance meter to '00000'.
    pub fn init(&mut self, width: i32) {
        let mut max_distance_str = String::with_capacity(self.max_score_units);

        self.calc_x_pos(width);
        for i in 0..self.max_score_units {
            self.draw(i, 0, false);
            self.default_string.push('0');
            max_distance_str.push('9');
        }

        self.max_score = max_distance_str.parse::<u32>().unwrap_or(99999);
    }

    /// Calculate the x_pos in the canvas.
    pub fn calc_x_pos(&mut self, canvas_width: i32) {
        self.x = canvas_width
            - (DistanceMeterDimensions::DEST_WIDTH * (self.max_score_units as i32 + 1));
    }

    /// Draw a digit to the render target.
    pub fn draw<F>(&self, digit_pos: usize, value: u8, opt_high_score: bool, is_hidpi: bool, mut draw_fn: F)
    where
        F: FnMut(DrawCall),
    {
        let mut source_width = DistanceMeterDimensions::WIDTH;
        let mut source_height = DistanceMeterDimensions::HEIGHT;
        let mut source_x = DistanceMeterDimensions::WIDTH * (value as i32);
        let mut source_y = 0;

        let target_x = (digit_pos as i32) * DistanceMeterDimensions::DEST_WIDTH;
        let target_y = self.y;
        let target_width = DistanceMeterDimensions::WIDTH;
        let target_height = DistanceMeterDimensions::HEIGHT;

        // For high DPI we 2x source values.
        if is_hidpi {
            source_width *= 2;
            source_height *= 2;
            source_x *= 2;
        }

        source_x += self.sprite_pos.x;
        source_y += self.sprite_pos.y;

        // Equivalent to canvasCtx.translate(...) + drawImage(...)
        let offset_x = if opt_high_score {
            // Left of the current score.
            self.x - (self.max_score_units as i32 * 2) * DistanceMeterDimensions::WIDTH
        } else {
            self.x
        };

        draw_fn(DrawCall {
            src_x: source_x,
            src_y: source_y,
            src_w: source_width,
            src_h: source_height,
            dst_x: offset_x + target_x,
            dst_y: self.y + target_y,
            dst_w: target_width,
            dst_h: target_height,
        });
    }

    /// Convert pixel distance to a 'real' distance.
    pub fn get_actual_distance(&self, distance: f64) -> u32 {
        if distance > 0.0 {
            (distance * self.config.coefficient).round() as u32
        } else {
            0
        }
    }

    /// Update the distance meter.
    /// Returns `true` if the achievement sound fx should be played.
    pub fn update<F>(&mut self, delta_time: f64, distance: f64, is_hidpi: bool, mut draw_fn: F) -> bool
    where
        F: FnMut(DrawCall),
    {
        let mut paint = true;
        let mut play_sound = false;

        if !self.achievement {
            let actual_distance = self.get_actual_distance(distance);

            // Score has gone beyond the initial digit count.
            if actual_distance > self.max_score
                && self.max_score_units == self.config.max_distance_units
            {
                self.max_score_units += 1;
                self.max_score = self.max_score * 10 + 9;
                self.default_string.push('0');
            } else {
                self.current_distance = 0.0;
            }

            if actual_distance > 0 {
                // Achievement unlocked
                if actual_distance % self.config.achievement_distance == 0 {
                    // Flash score and play sound.
                    self.achievement = true;
                    self.flash_timer = 0.0;
                    play_sound = true;
                }

                // Format distance with leading zeros to `max_score_units` width.
                let formatted = format!(
                    "{:0>width$}",
                    actual_distance,
                    width = self.max_score_units
                );
                // Keep only the rightmost `max_score_units` digits (mirrors `.substr(-maxScoreUnits)`).
                let start = formatted.len().saturating_sub(self.max_score_units);
                self.digits = formatted[start..]
                    .bytes()
                    .map(|b| b - b'0')
                    .collect();
            } else {
                self.digits = vec![0; self.max_score_units];
            }
        } else {
            // Control flashing of the score on reaching achievement.
            if self.flash_iterations <= self.config.flash_iterations {
                self.flash_timer += delta_time;

                if self.flash_timer < self.config.flash_duration {
                    paint = false;
                } else if self.flash_timer > self.config.flash_duration * 2.0 {
                    self.flash_timer = 0.0;
                    self.flash_iterations += 1;
                }
            } else {
                self.achievement = false;
                self.flash_iterations = 0;
                self.flash_timer = 0.0;
            }
        }

        // Draw the digits if not flashing.
        if paint {
            for (i, &digit) in self.digits.iter().enumerate().rev() {
                self.draw(i, digit, false, is_hidpi, &mut draw_fn);
            }
        }

        self.draw_high_score(is_hidpi, &mut draw_fn);
        play_sound
    }

    /// Draw the high score (with 0.8 global alpha).
    pub fn draw_high_score<F>(&self, is_hidpi: bool, draw_fn: &mut F)
    where
        F: FnMut(DrawCall),
    {
        for (i, &slot) in self.high_score.iter().enumerate().rev() {
            // Skip the spacer slot (`None`), which was `''` -> `NaN` in JS.
            if let Some(glyph_index) = slot {
                self.draw_with_alpha(i, glyph_index, true, 0.8, is_hidpi, draw_fn);
            }
        }
    }

    /// Set the high score as a glyph index sequence.
    /// Position of char in the sprite: H = 10, I = 11, None = blank space.
    pub fn set_high_score(&mut self, distance: f64) {
        let actual_distance = self.get_actual_distance(distance);
        let formatted = format!(
            "{:0>width$}",
            actual_distance,
            width = self.max_score_units
        );
        let start = formatted.len().saturating_sub(self.max_score_units);

        // ['10', '11', ''] -> H (10), I (11), blank spacer (None)
        let mut high_score = Vec::with_capacity(3 + self.max_score_units);
        high_score.push(Some(10));
        high_score.push(Some(11));
        high_score.push(None);

        for b in formatted[start..].bytes() {
            high_score.push(Some(b - b'0'));
        }

        self.high_score = high_score;
    }

    /// Reset the distance meter back to '00000'.
    pub fn reset(&mut self) {
        self.current_distance = 0.0;
        self.digits = vec![0; self.max_score_units];
        self.achievement = false;
        self.flash_timer = 0.0;
        self.flash_iterations = 0;
    }

    /// Helper used by both `draw` (alpha 1.0) and `draw_high_score` (alpha 0.8).
    fn draw_with_alpha<F>(
        &self,
        digit_pos: usize,
        value: u8,
        opt_high_score: bool,
        alpha: f32,
        is_hidpi: bool,
        draw_fn: &mut F,
    ) where
        F: FnMut(DrawCall),
    {
        let mut source_width = DistanceMeterDimensions::WIDTH;
        let mut source_height = DistanceMeterDimensions::HEIGHT;
        let mut source_x = DistanceMeterDimensions::WIDTH * (value as i32);
        let mut source_y = 0;

        let target_x = (digit_pos as i32) * DistanceMeterDimensions::DEST_WIDTH;
        let target_y = self.y;
        let target_width = DistanceMeterDimensions::WIDTH;
        let target_height = DistanceMeterDimensions::HEIGHT;

        if is_hidpi {
            source_width *= 2;
            source_height *= 2;
            source_x *= 2;
        }

        source_x += self.sprite_pos.x;
        source_y += self.sprite_pos.y;

        let offset_x = if opt_high_score {
            self.x - (self.max_score_units as i32 * 2) * DistanceMeterDimensions::WIDTH
        } else {
            self.x
        };

        draw_fn(DrawCall {
            src_x: source_x,
            src_y: source_y,
            src_w: source_width,
            src_h: source_height,
            dst_x: offset_x + target_x,
            dst_y: self.y + target_y,
            dst_w: target_width,
            dst_h: target_height,
            alpha,
        });
    }


}

impl CloudConfig {
    pub const HEIGHT: i32 = 14;
    pub const MAX_CLOUD_GAP: i32 = 400;
    pub const MAX_SKY_LEVEL: i32 = 30;
    pub const MIN_CLOUD_GAP: i32 = 100;
    pub const MIN_SKY_LEVEL: i32 = 71;
    pub const WIDTH: i32 = 46;
}

pub struct Cloud {
    pub sprite_pos: SpritePos,
    pub container_width: i32,
    pub x_pos: i32,
    pub y_pos: i32,
    pub remove: bool,
    pub cloud_gap: i32,
}

impl Cloud {
    pub fn new(sprite_pos: SpritePos, container_width: i32) -> Self {
        let mut rng = rng();
        let cloud_gap = rng.gen_range(CloudConfig::MIN_CLOUD_GAP..=CloudConfig::MAX_CLOUD_GAP);

        let mut cloud = Self {
            sprite_pos,
            container_width,
            x_pos: container_width,
            y_pos: 0,
            remove: false,
            cloud_gap,
        };

        cloud.init();
        cloud
    }

    /// Initialise the cloud. Sets the Cloud height.
    pub fn init(&mut self) {
        let mut rng = rng();
        // Note: MAX_SKY_LEVEL (30) is smaller than MIN_SKY_LEVEL (71) because y=0 is the top of the screen.
        self.y_pos = rng.gen_range(CloudConfig::MAX_SKY_LEVEL..=CloudConfig::MIN_SKY_LEVEL);
    }

    /// Draw the cloud.
    pub fn draw<F>(&self, is_hidpi: bool, draw_fn: &mut F)
    where
        F: FnMut(DrawCall),
    {
        let mut source_width = CloudConfig::WIDTH;
        let mut source_height = CloudConfig::HEIGHT;

        if is_hidpi {
            source_width *= 2;
            source_height *= 2;
        }

        draw_fn(DrawCall {
            src_x: self.sprite_pos.x,
            src_y: self.sprite_pos.y,
            src_w: source_width,
            src_h: source_height,
            dst_x: self.x_pos,
            dst_y: self.y_pos,
            dst_w: CloudConfig::WIDTH,
            dst_h: CloudConfig::HEIGHT,
            alpha: 1.0,
        });
    }

    /// Update the cloud position and emit a draw call if still active.
    pub fn update<F>(&mut self, speed: f64, is_hidpi: bool, mut draw_fn: F)
    where
        F: FnMut(DrawCall),
    {
        if !self.remove {
            self.x_pos -= speed.ceil() as i32;
            self.draw(is_hidpi, &mut draw_fn);

            // Mark as removable if no longer in the canvas.
            if !self.is_visible() {
                self.remove = true;
            }
        }
    }

    /// Check if the cloud is visible on the stage.
    pub fn is_visible(&self) -> bool {
        self.x_pos + CloudConfig::WIDTH > 0
    }
}

impl NightModeConfig {
    pub const FADE_SPEED: f32 = 0.035;
    pub const HEIGHT: f32 = 40.0;
    pub const MOON_SPEED: f32 = 0.25;
    pub const NUM_STARS: usize = 2;
    pub const STAR_SIZE: f32 = 9.0;
    pub const STAR_SPEED: f32 = 0.3;
    pub const STAR_MAX_Y: i32 = 70;
    pub const WIDTH: f32 = 20.0;
    pub const PHASES: [f32; 7] = [140.0, 120.0, 100.0, 60.0, 40.0, 20.0, 0.0];
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub source_y: f32,
}

pub struct NightMode {
    pub sprite_pos: Position,
    pub x_pos: f32,
    pub y_pos: f32,
    pub current_phase: usize,
    pub opacity: f32,
    pub container_width: f32,
    pub stars: Vec<Star>,
    pub draw_stars: bool,
    pub is_hidpi: bool,
    pub star_sprite_pos: Position, // Passed from Runner.spriteDefinition
}

impl NightMode {
    pub fn new(
        sprite_pos: Position,
        star_sprite_pos: Position,
        container_width: f32,
        is_hidpi: bool,
    ) -> Self {
        let mut night_mode = Self {
            sprite_pos,
            x_pos: container_width - 50.0,
            y_pos: 30.0,
            current_phase: 0,
            opacity: 0.0,
            container_width,
            stars: Vec::with_capacity(NightModeConfig::NUM_STARS),
            draw_stars: false,
            is_hidpi,
            star_sprite_pos,
        };
        night_mode.place_stars();
        night_mode
    }

    /// Update moving moon and changing phases.
    /// Note: `_delta` is kept to match the JS signature if your main loop passes it.
    pub fn update(&mut self, activated: bool, _delta: f32) {
        // Moon phase progression
        if activated && self.opacity == 0.0 {
            self.current_phase += 1;
            if self.current_phase >= NightModeConfig::PHASES.len() {
                self.current_phase = 0;
            }
        }

        // Fade in / out
        if activated && (self.opacity < 1.0 || self.opacity == 0.0) {
            self.opacity = (self.opacity + NightModeConfig::FADE_SPEED).min(1.0);
        } else if self.opacity > 0.0 {
            self.opacity = (self.opacity - NightModeConfig::FADE_SPEED).max(0.0);
        }

        // Set moon and star positioning
        if self.opacity > 0.0 {
            self.x_pos = self.update_x_pos(self.x_pos, NightModeConfig::MOON_SPEED);

            if self.draw_stars {
                for i in 0..self.stars.len() {
                    self.stars[i].x =
                        self.update_x_pos(self.stars[i].x, NightModeConfig::STAR_SPEED);
                }
            }
        } else {
            self.opacity = 0.0;
            self.place_stars();
        }

        self.draw_stars = true;
    }

    fn update_x_pos(&self, mut current_pos: f32, speed: f32) -> f32 {
        if current_pos < -NightModeConfig::WIDTH {
            current_pos = self.container_width;
        } else {
            current_pos -= speed;
        }
        current_pos
    }

    /// Render the stars and moon. Replace the body with your engine's draw calls.
    pub fn draw(&self /*, ctx: &mut YourRenderContext, sprite_sheet: &Texture */) {
        if self.opacity <= 0.0 {
            return;
        }

        let mut moon_source_width = if self.current_phase == 3 {
            NightModeConfig::WIDTH * 2.0
        } else {
            NightModeConfig::WIDTH
        };
        let mut moon_source_height = NightModeConfig::HEIGHT;
        let mut moon_source_x = self.sprite_pos.x + NightModeConfig::PHASES[self.current_phase];
        let moon_output_width = moon_source_width;

        let mut star_size = NightModeConfig::STAR_SIZE;
        let star_source_x = self.star_sprite_pos.x;

        if self.is_hidpi {
            moon_source_width *= 2.0;
            moon_source_height *= 2.0;
            moon_source_x =
                self.sprite_pos.x + (NightModeConfig::PHASES[self.current_phase] * 2.0);
            star_size *= 2.0;
        }

        // 1. Draw Stars (apply `self.opacity` as alpha to your sprite color/tint)
        if self.draw_stars {
            for star in &self.stars {
                // Example draw call mapping:
                // src_rect: (star_source_x, star.source_y, star_size, star_size)
                // dst_rect: (star.x.round(), star.y, NightModeConfig::STAR_SIZE, NightModeConfig::STAR_SIZE)
                // alpha: self.opacity
            }
        }

        // 2. Draw Moon
        // Example draw call mapping:
        // src_rect: (moon_source_x, self.sprite_pos.y, moon_source_width, moon_source_height)
        // dst_rect: (self.x_pos.round(), self.y_pos, moon_output_width, NightModeConfig::HEIGHT)
        // alpha: self.opacity
    }

    /// Randomize star placement across screen segments.
    pub fn place_stars(&mut self) {
        let mut rng = rng();
        let segment_size = (self.container_width / NightModeConfig::NUM_STARS as f32).round() as i32;

        self.stars.clear();

        for i in 0..NightModeConfig::NUM_STARS {
            let min_x = segment_size * (i as i32);
            let max_x = segment_size * ((i + 1) as i32);

            let x = rng.gen_range(min_x..=max_x) as f32;
            let y = rng.gen_range(0..=NightModeConfig::STAR_MAX_Y) as f32;

            let source_y = if self.is_hidpi {
                self.star_sprite_pos.y + NightModeConfig::STAR_SIZE * 2.0 * (i as f32)
            } else {
                self.star_sprite_pos.y + NightModeConfig::STAR_SIZE * (i as f32)
            };

            self.stars.push(Star { x, y, source_y });
        }
    }

    pub fn reset(&mut self) {
        self.current_phase = 0;
        self.opacity = 0.0;
        self.update(false, 0.0);
    }
}

impl HorizonLineDimensions {
    pub const WIDTH: f32 = 600.0;
    pub const HEIGHT: f32 = 12.0;
    pub const YPOS: f32 = 127.0;
}

pub struct HorizonLine {
    pub sprite_pos: Position,
    pub source_width: f32,
    pub source_height: f32,
    pub source_x_pos: [f32; 2],
    pub x_pos: [f32; 2],
    pub y_pos: f32,
    pub bump_threshold: f32,
    pub is_hidpi: bool,
    pub fps: f32, // Usually 60.0 from Runner.config
}

impl HorizonLine {
    pub fn new(sprite_pos: Position, is_hidpi: bool, fps: f32) -> Self {
        let mut horizon = Self {
            sprite_pos,
            source_width: HorizonLineDimensions::WIDTH,
            source_height: HorizonLineDimensions::HEIGHT,
            source_x_pos: [sprite_pos.x, sprite_pos.x + HorizonLineDimensions::WIDTH],
            x_pos: [0.0, HorizonLineDimensions::WIDTH],
            y_pos: HorizonLineDimensions::YPOS,
            bump_threshold: 0.5,
            is_hidpi,
            fps,
        };

        horizon.set_source_dimensions();
        horizon
    }

    /// Set the source dimensions of the horizon line based on HiDPI mode.
    pub fn set_source_dimensions(&mut self) {
        if self.is_hidpi {
            self.source_width = HorizonLineDimensions::WIDTH * 2.0;
            self.source_height = HorizonLineDimensions::HEIGHT * 2.0;
        } else {
            self.source_width = HorizonLineDimensions::WIDTH;
            self.source_height = HorizonLineDimensions::HEIGHT;
        }

        self.x_pos = [0.0, HorizonLineDimensions::WIDTH];
        self.y_pos = HorizonLineDimensions::YPOS;
    }

    /// Return the crop x offset for either flat or bumpy ground.
    fn get_random_type(&self) -> f32 {
        let mut rng = rng();
        if rng.random::<f32>() > self.bump_threshold {
            HorizonLineDimensions::WIDTH
        } else {
            0.0
        }
    }

    /// Render the two connecting horizon segments. Replace body with your engine's draw calls.
    pub fn draw(&self /*, ctx: &mut YourRenderContext, sprite_sheet: &Texture */) {
        for i in 0..2 {
            // Example draw call mapping:
            // src_rect: (self.source_x_pos[i], self.sprite_pos.y, self.source_width, self.source_height)
            // dst_rect: (self.x_pos[i], self.y_pos, HorizonLineDimensions::WIDTH, HorizonLineDimensions::HEIGHT)
        }
    }

    /// Update the x position of an individual piece of the line.
    fn update_x_pos(&mut self, pos: usize, increment: f32) {
        let line1 = pos;
        let line2 = if pos == 0 { 1 } else { 0 };

        self.x_pos[line1] -= increment;
        self.x_pos[line2] = self.x_pos[line1] + HorizonLineDimensions::WIDTH;

        if self.x_pos[line1] <= -HorizonLineDimensions::WIDTH {
            self.x_pos[line1] += HorizonLineDimensions::WIDTH * 2.0;
            self.x_pos[line2] = self.x_pos[line1] - HorizonLineDimensions::WIDTH;
            self.source_x_pos[line1] = self.get_random_type() + self.sprite_pos.x;
        }
    }

    /// Update the horizon line position.
    /// `delta_time` is expected in milliseconds (matching JS `deltaTime`).
    pub fn update(&mut self, delta_time: f32, speed: f32) {
        let increment = (speed * (self.fps / 1000.0) * delta_time).floor();

        if self.x_pos[0] <= 0.0 {
            self.update_x_pos(0, increment);
        } else {
            self.update_x_pos(1, increment);
        }
    }

    /// Reset horizon to the starting position.
    pub fn reset(&mut self) {
        self.x_pos[0] = 0.0;
        self.x_pos[1] = HorizonLineDimensions::WIDTH;
    }
}

impl HorizonConfig {
    pub const BG_CLOUD_SPEED: f32 = 0.2;
    pub const BUMPY_THRESHOLD: f32 = 0.3;
    pub const CLOUD_FREQUENCY: f32 = 0.5;
    pub const HORIZON_HEIGHT: f32 = 16.0;
    pub const MAX_CLOUDS: usize = 6;
}

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub width: f32,
    pub height: f32,
}

/// Holds sprite coordinates needed by Horizon and its children
#[derive(Debug, Clone, Copy)]
pub struct HorizonSpritePos {
    pub horizon: Position,
    pub moon: Position,
    pub star: Position,
    pub cloud: Position,
    pub cactus_small: Position,
    pub cactus_large: Position,
    pub pterodactyl: Position,
}

impl HorizonSpritePos {
    pub fn for_obstacle(&self, obstacle_type: ObstacleType) -> Position {
        match obstacle_type {
            ObstacleType::CactusSmall => self.cactus_small,
            ObstacleType::CactusLarge => self.cactus_large,
            ObstacleType::Pterodactyl => self.pterodactyl,
        }
    }
}

pub struct Horizon {
    pub dimensions: Dimensions,
    pub gap_coefficient: f32,
    pub obstacles: VecDeque<Obstacle>,
    pub obstacle_history: VecDeque<ObstacleType>,
    pub cloud_frequency: f32,
    pub sprite_pos: HorizonSpritePos,
    pub night_mode: NightMode,
    pub clouds: Vec<Cloud>,
    pub cloud_speed: f32,
    pub horizon_line: HorizonLine,
    pub running_time: f32,
    pub max_obstacle_duplication: usize, // From Runner.config.MAX_OBSTACLE_DUPLICATION (default 2)
    pub is_hidpi: bool,
    pub fps: f32,
}

impl Horizon {
    pub fn new(
        sprite_pos: HorizonSpritePos,
        dimensions: Dimensions,
        gap_coefficient: f32,
        max_obstacle_duplication: usize,
        is_hidpi: bool,
        fps: f32,
    ) -> Self {
        let mut horizon = Self {
            dimensions,
            gap_coefficient,
            obstacles: VecDeque::new(),
            obstacle_history: VecDeque::with_capacity(max_obstacle_duplication + 1),
            cloud_frequency: HorizonConfig::CLOUD_FREQUENCY,
            sprite_pos,
            night_mode: NightMode::new(
                sprite_pos.moon,
                sprite_pos.star,
                dimensions.width,
                is_hidpi,
            ),
            clouds: Vec::with_capacity(HorizonConfig::MAX_CLOUDS),
            cloud_speed: HorizonConfig::BG_CLOUD_SPEED,
            horizon_line: HorizonLine::new(sprite_pos.horizon, is_hidpi, fps),
            running_time: 0.0,
            max_obstacle_duplication,
            is_hidpi,
            fps,
        };

        horizon.init();
        horizon
    }

    /// Initialise the horizon with the ground line and an initial cloud.
    pub fn init(&mut self) {
        self.add_cloud();
    }

    /// Update all horizon background elements and obstacles.
    pub fn update(
        &mut self,
        delta_time: f32,
        current_speed: f32,
        update_obstacles: bool,
        show_night_mode: bool,
    ) {
        self.running_time += delta_time;
        self.horizon_line.update(delta_time, current_speed);
        self.night_mode.update(show_night_mode, delta_time);
        self.update_clouds(delta_time, current_speed);

        if update_obstacles {
            self.update_obstacles(delta_time, current_speed);
        }
    }

    /// Render all horizon layers in back-to-front painter's order.
    pub fn draw(&self /*, ctx: &mut YourRenderContext, sprite_sheet: &Texture */) {
        self.horizon_line.draw(/* ctx, sprite_sheet */);
        self.night_mode.draw(/* ctx, sprite_sheet */);

        for cloud in &self.clouds {
            cloud.draw(/* ctx, sprite_sheet */);
        }

        for obstacle in &self.obstacles {
            obstacle.draw(/* ctx, sprite_sheet */);
        }
    }

    /// Update cloud positions and spawn/remove clouds as needed.
    fn update_clouds(&mut self, delta_time: f32, speed: f32) {
        let cloud_speed = (self.cloud_speed / 1000.0) * delta_time * speed;

        if !self.clouds.is_empty() {
            for cloud in self.clouds.iter_mut().rev() {
                cloud.update(cloud_speed);
            }

            let should_add_cloud = if let Some(last_cloud) = self.clouds.last() {
                let mut rng = rng();
                self.clouds.len() < HorizonConfig::MAX_CLOUDS
                    && (self.dimensions.width - last_cloud.x_pos) > last_cloud.cloud_gap
                    && self.cloud_frequency > rng.random::<f32>()
            } else {
                false
            };

            if should_add_cloud {
                self.add_cloud();
            }

            self.clouds.retain(|cloud| !cloud.remove);
        } else {
            self.add_cloud();
        }
    }

    /// Update obstacle positions and spawn new ones when spacing permits.
    fn update_obstacles(&mut self, delta_time: f32, current_speed: f32) {
        for obstacle in &mut self.obstacles {
            obstacle.update(delta_time, current_speed);
        }

        // Remove off-screen obstacles from the front
        while self.obstacles.front().map_or(false, |obs| obs.remove) {
            self.obstacles.pop_front();
        }

        let spawn_next = if let Some(last_obstacle) = self.obstacles.back_mut() {
            if !last_obstacle.following_obstacle_created
                && last_obstacle.is_visible()
                && (last_obstacle.x_pos + last_obstacle.width + last_obstacle.gap)
                    < self.dimensions.width
            {
                last_obstacle.following_obstacle_created = true;
                true
            } else {
                false
            }
        } else {
            true
        };

        if spawn_next {
            self.add_new_obstacle(current_speed);
        }
    }

    pub fn remove_first_obstacle(&mut self) {
        self.obstacles.pop_front();
    }

    /// Spawn a new valid obstacle avoiding excessive duplicates and speed locks.
    pub fn add_new_obstacle(&mut self, current_speed: f32) {
        let mut rng = rng();
        let types = Obstacle::types();

        // Loop instead of recursion to avoid stack overflow on unlucky RNG streaks
        loop {
            let idx = rng.gen_range(0..types.len());
            let obstacle_type = &types[idx];

            if !self.duplicate_obstacle_check(obstacle_type.kind)
                && current_speed >= obstacle_type.min_speed
            {
                let obstacle_sprite_pos = self.sprite_pos.for_obstacle(obstacle_type.kind);

                self.obstacles.push_back(Obstacle::new(
                    obstacle_type.clone(),
                    obstacle_sprite_pos,
                    self.dimensions,
                    self.gap_coefficient,
                    current_speed,
                    obstacle_type.width,
                    self.is_hidpi,
                    self.fps,
                ));

                self.obstacle_history.push_front(obstacle_type.kind);
                if self.obstacle_history.len() > self.max_obstacle_duplication {
                    self.obstacle_history.truncate(self.max_obstacle_duplication);
                }
                break;
            }
        }
    }

    /// Returns whether the previous obstacles match `next_obstacle_type` up to the limit.
    fn duplicate_obstacle_check(&self, next_obstacle_type: ObstacleType) -> bool {
        let mut duplicate_count = 0;

        for &history_type in &self.obstacle_history {
            duplicate_count = if history_type == next_obstacle_type {
                duplicate_count + 1
            } else {
                0
            };
        }

        duplicate_count >= self.max_obstacle_duplication
    }

    /// Reset the horizon layer, clearing obstacles and resetting ground/night mode.
    pub fn reset(&mut self) {
        self.obstacles.clear();
        self.obstacle_history.clear();
        self.horizon_line.reset();
        self.night_mode.reset();
    }

    /// Update stored logical dimensions on window resize.
    pub fn resize(&mut self, width: f32, height: f32) {
        self.dimensions.width = width;
        self.dimensions.height = height;
        self.night_mode.container_width = width;
    }

    /// Add a new cloud to the horizon.
    pub fn add_cloud(&mut self) {
        self.clouds.push(Cloud::new(
            self.sprite_pos.cloud,
            self.dimensions.width,
            self.is_hidpi,
        ));
    }
}