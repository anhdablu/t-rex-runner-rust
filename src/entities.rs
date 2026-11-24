// TODO: Migrate CollisionBox, Trex, Obstacle here
// ==========================================
// GameOverPanel Dimensions (Replaces GameOverPanel.dimensions)
// ==========================================
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use macroquad::time::get_time;
use std::collections::HashMap;
pub struct GameOverPanelDimensions;

/// Replaces `Trex.BLINK_TIMING`
pub const BLINK_TIMING: f32 = 7000.0;
pub const FPS: f32 = 60.0;
pub const MS_PER_FRAME: f32 = 1000.0 / FPS;

// ==========================================
// 1. Constants (Replaces Obstacle.MAX_*)
// ==========================================
pub const MAX_GAP_COEFFICIENT: f32 = 1.5;
pub const MAX_OBSTACLE_LENGTH: usize = 3;

impl GameOverPanelDimensions {
    pub const TEXT_X: f32 = 0.0;
    pub const TEXT_Y: f32 = 13.0;
    pub const TEXT_WIDTH: f32 = 191.0;
    pub const TEXT_HEIGHT: f32 = 11.0;
    pub const RESTART_WIDTH: f32 = 36.0;
    pub const RESTART_HEIGHT: f32 = 32.0;
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum TrexStatus {
//     Crashed,
//     Ducking,
//     Jumping,
//     Running,
//     Waiting,
// }

// ==========================================
// GameOverPanel Struct (Replaces function GameOverPanel)
// ==========================================
#[derive(Clone, Debug)]
pub struct GameOverPanel {
    pub canvas_dimensions: Vec2,
    pub text_img_pos: Vec2,
    pub restart_img_pos: Vec2,
}

impl GameOverPanel {
    pub fn new(text_img_pos: Vec2, restart_img_pos: Vec2, dimensions: Vec2) -> Self {
        Self {
            canvas_dimensions: dimensions,
            text_img_pos,
            restart_img_pos,
        }
    }

    /// Replaces `GameOverPanel.prototype.updateDimensions`
    pub fn update_dimensions(&mut self, width: f32, opt_height: Option<f32>) {
        self.canvas_dimensions.x = width;
        if let Some(height) = opt_height {
            self.canvas_dimensions.y = height;
        }
    }

    /// Replaces `GameOverPanel.prototype.draw`
    pub fn draw(&self, sprite: &Texture2D, is_hidpi: bool, y_offset: f32) {
        let center_x = self.canvas_dimensions.x / 2.0;

        // Game Over text coordinates
        let mut text_source_x = GameOverPanelDimensions::TEXT_X;
        let mut text_source_y = GameOverPanelDimensions::TEXT_Y;
        let mut text_source_width = GameOverPanelDimensions::TEXT_WIDTH;
        let mut text_source_height = GameOverPanelDimensions::TEXT_HEIGHT;

        let text_target_x = (center_x - (GameOverPanelDimensions::TEXT_WIDTH / 2.0)).round();
        let text_target_y = ((self.canvas_dimensions.y - 25.0) / 3.0).round() + y_offset;
        let text_target_width = GameOverPanelDimensions::TEXT_WIDTH;
        let text_target_height = GameOverPanelDimensions::TEXT_HEIGHT;

        // Restart icon coordinates
        let mut restart_source_width = GameOverPanelDimensions::RESTART_WIDTH;
        let mut restart_source_height = GameOverPanelDimensions::RESTART_HEIGHT;
        let restart_target_x = center_x - (GameOverPanelDimensions::RESTART_WIDTH / 2.0);
        let restart_target_y = (self.canvas_dimensions.y / 2.0) + y_offset;

        // Scale source coordinates by 2x when using the HiDPI sprite sheet
        if is_hidpi {
            text_source_y *= 2.0;
            text_source_x *= 2.0;
            text_source_width *= 2.0;
            text_source_height *= 2.0;
            restart_source_width *= 2.0;
            restart_source_height *= 2.0;
        }

        text_source_x += self.text_img_pos.x;
        text_source_y += self.text_img_pos.y;

        // 1. Draw "GAME OVER" text from sprite sheet
        draw_texture_ex(
            sprite,
            text_target_x,
            text_target_y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    text_source_x,
                    text_source_y,
                    text_source_width,
                    text_source_height,
                )),
                dest_size: Some(vec2(text_target_width, text_target_height)),
                ..Default::default()
            },
        );

        // 2. Draw Restart button icon from sprite sheet
        draw_texture_ex(
            sprite,
            restart_target_x,
            restart_target_y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    self.restart_img_pos.x,
                    self.restart_img_pos.y,
                    restart_source_width,
                    restart_source_height,
                )),
                dest_size: Some(vec2(
                    GameOverPanelDimensions::RESTART_WIDTH,
                    GameOverPanelDimensions::RESTART_HEIGHT,
                )),
                ..Default::default()
            },
        );
    }
}

// ==========================================
// 1. CollisionBox Struct
// ==========================================
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollisionBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl CollisionBox {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

// ==========================================
// 2. Axis-Aligned Bounding Box (AABB) Check
// ==========================================
/// Replaces `boxCompare(tRexBox, obstacleBox)`
pub fn box_compare(trex_box: &CollisionBox, obstacle_box: &CollisionBox) -> bool {
    let crashed = trex_box.x < obstacle_box.x + obstacle_box.width
        && trex_box.x + trex_box.width > obstacle_box.x
        && trex_box.y < obstacle_box.y + obstacle_box.height
        && trex_box.height + trex_box.y > obstacle_box.y;

    crashed
}

/// Replaces `createAdjustedCollisionBox(box, adjustment)`
pub fn create_adjusted_collision_box(
    box_orig: &CollisionBox,
    adjustment: &CollisionBox,
) -> CollisionBox {
    CollisionBox::new(
        box_orig.x + adjustment.x,
        box_orig.y + adjustment.y,
        box_orig.width,
        box_orig.height,
    )
}

/// Replaces `drawCollisionBoxes(canvasCtx, tRexBox, obstacleBox)`
pub fn draw_collision_boxes(
    trex_box: &CollisionBox,
    obstacle_box: &CollisionBox,
    y_offset: f32,
) {
    // Red box for T-Rex (#f00)
    draw_rectangle_lines(
        trex_box.x,
        trex_box.y + y_offset,
        trex_box.width,
        trex_box.height,
        1.0,
        RED,
    );

    // Green box for Obstacle (#0f0)
    draw_rectangle_lines(
        obstacle_box.x,
        obstacle_box.y + y_offset,
        obstacle_box.width,
        obstacle_box.height,
        1.0,
        GREEN,
    );
}

// ==========================================
// 3. Main Collision Check
// ==========================================
/// Replaces `checkForCollision(obstacle, tRex, opt_canvasCtx)`
/// Pass `debug_y_offset: Some(25.0)` to render hitboxes, or `None` during normal play.
pub fn check_for_collision(
    obstacle: &Obstacle,
    trex: &Trex,
    debug_y_offset: Option<f32>,
) -> Option<(CollisionBox, CollisionBox)> {
    // Adjustments are made to the bounding box as there is a 1 pixel white
    // border around the t-rex and obstacles.
    let trex_box = CollisionBox::new(
        trex.x_pos + 1.0,
        trex.y_pos + 1.0,
        trex.width() - 2.0,
        trex.height() - 2.0,
    );

    let obstacle_box = CollisionBox::new(
        obstacle.x_pos + 1.0,
        obstacle.y_pos + 1.0,
        (obstacle.type_config.width * obstacle.size as f32) - 2.0,
        obstacle.type_config.height - 2.0,
    );

    // Debug outer box
    if let Some(y_offset) = debug_y_offset {
        draw_collision_boxes(&trex_box, &obstacle_box, y_offset);
    }

    // Simple outer bounds check
    if box_compare(&trex_box, &obstacle_box) {
        // let collision_boxes = &obstacle.collision_boxes;
        let trex_collision_boxes = if trex.ducking {
            Trex::COLLISION_BOXES_DUCKING
        } else {
            Trex::COLLISION_BOXES_RUNNING
        };

        // Detailed axis-aligned box check
        for t_box in trex_collision_boxes {
            // for o_box in collision_boxes {
            for o_box in obstacle.collision_boxes {
                let adj_trex_box = create_adjusted_collision_box(t_box, &trex_box);
                let adj_obstacle_box = create_adjusted_collision_box(o_box, &obstacle_box);
                let crashed = box_compare(&adj_trex_box, &adj_obstacle_box);

                // Draw inner boxes for debug
                if let Some(y_offset) = debug_y_offset {
                    draw_collision_boxes(&adj_trex_box, &adj_obstacle_box, y_offset);
                }

                if crashed {
                    return Some((adj_trex_box, adj_obstacle_box));
                }
            }
        }
    }

    None
}

// // ==========================================
// // 2. ObstacleType Config (Replaces Obstacle.types entries)
// // ==========================================
// #[derive(Clone, Debug)]
// pub struct ObstacleType {
//     pub type_name: &'static str,
//     pub width: f32,
//     pub height: f32,
//     pub y_pos: &'static [f32],
//     pub multiple_speed: f32,
//     pub min_gap: f32,
//     pub min_speed: f32,
//     pub num_frames: Option<usize>,
//     pub frame_rate: Option<f32>,
//     pub speed_offset: Option<f32>,
//     pub collision_boxes: Vec<CollisionBox>,
// }

// ==========================================
// 3. Obstacle Struct (Replaces function Obstacle)
// ==========================================
#[derive(Clone, Copy, Debug)]
pub struct Obstacle {
    pub sprite_pos: Vec2,
    pub type_config: ObstacleType,
    pub gap_coefficient: f32,
    pub size: usize,
    pub dimensions: Vec2,
    pub remove: bool,
    pub x_pos: f32,
    pub y_pos: f32,
    pub width: f32,
    // pub collision_boxes: Vec<CollisionBox>,
    pub collision_boxes: &'static [CollisionBox],
    pub gap: f32,
    pub speed_offset: f32,

    // For animated obstacles (e.g., Pterodactyl)
    pub current_frame: usize,
    pub timer: f32,
}

impl Obstacle {
    /// Replaces `new Obstacle(...)` and `Obstacle.prototype.init(speed)`
    pub fn new(
        type_config: ObstacleType,
        sprite_img_pos: Vec2,
        dimensions: Vec2,
        gap_coefficient: f32,
        speed: f32,
        opt_x_offset: Option<f32>,
    ) -> Self {
        let mut size = gen_range(1, MAX_OBSTACLE_LENGTH + 1);

        // Only allow grouping if we're at the right speed
        if size > 1 && type_config.multiple_speed > speed {
            size = 1;
        }

        let width = type_config.width * (size as f32);
        let x_pos = dimensions.x + opt_x_offset.unwrap_or(0.0);

        // Pick single Y position or random height from slice (for Pterodactyls)
        let y_pos = if type_config.y_pos.len() > 1 {
            let idx = gen_range(0, type_config.y_pos.len());
            type_config.y_pos[idx]
        } else {
            type_config.y_pos[0]
        };

        // Replaces `this.cloneCollisionBoxes()`
        let mut collision_boxes = type_config.collision_boxes.to_vec();

        // Adjust central & right collision boxes when multiple cacti are grouped together
        if size > 1 && collision_boxes.len() >= 3 {
            collision_boxes[1].width =
                width - collision_boxes[0].width - collision_boxes[2].width;
            collision_boxes[2].x = width - collision_boxes[2].width;
        }

        // For obstacles that fly at a different speed from the horizon (Pterodactyl)
        let speed_offset = if let Some(offset) = type_config.speed_offset {
            if gen_range(0.0, 1.0) > 0.5 {
                offset
            } else {
                -offset
            }
        } else {
            0.0
        };

        let mut obstacle = Self {
            sprite_pos: sprite_img_pos,
            type_config,
            gap_coefficient,
            size,
            dimensions,
            remove: false,
            x_pos,
            y_pos,
            width,
            collision_boxes,
            gap: 0.0,
            speed_offset,
            current_frame: 0,
            timer: 0.0,
        };

        obstacle.gap = obstacle.get_gap(gap_coefficient, speed);
        obstacle
    }

    /// Replaces `Obstacle.prototype.draw()`
    pub fn draw(&self, sprite: &Texture2D, is_hidpi: bool, y_offset: f32) {
        let mut source_width = self.type_config.width;
        let mut source_height = self.type_config.height;

        if is_hidpi {
            source_width *= 2.0;
            source_height *= 2.0;
        }

        // X position in sprite sheet (accounts for 1, 2, or 3 grouped cacti)
        let mut source_x = (source_width * (self.size as f32))
            * (0.5 * ((self.size as f32) - 1.0))
            + self.sprite_pos.x;

        // Animation frames (e.g., Pterodactyl wing flap)
        if self.current_frame > 0 {
            source_x += source_width * (self.current_frame as f32);
        }

        draw_texture_ex(
            sprite,
            self.x_pos,
            self.y_pos + y_offset,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    source_x,
                    self.sprite_pos.y,
                    source_width * (self.size as f32),
                    source_height,
                )),
                dest_size: Some(vec2(
                    self.type_config.width * (self.size as f32),
                    self.type_config.height,
                )),
                ..Default::default()
            },
        );
    }

    /// Replaces `Obstacle.prototype.update(deltaTime, speed)`
    pub fn update(&mut self, delta_time: f32, mut speed: f32) {
        if !self.remove {
            if self.type_config.speed_offset.is_some() {
                speed += self.speed_offset;
            }

            // Matches `Math.floor((speed * FPS / 1000) * deltaTime)`
            self.x_pos -= ((speed * FPS / 1000.0) * delta_time).floor();

            // Update animation frame (Pterodactyl)
            if let (Some(num_frames), Some(frame_rate)) =
                (self.type_config.num_frames, self.type_config.frame_rate)
            {
                self.timer += delta_time;
                if self.timer >= frame_rate {
                    self.current_frame = if self.current_frame == num_frames - 1 {
                        0
                    } else {
                        self.current_frame + 1
                    };
                    self.timer = 0.0;
                }
            }

            if !self.is_visible() {
                self.remove = true;
            }
        }
    }

    /// Replaces `Obstacle.prototype.getGap(gapCoefficient, speed)`
    pub fn get_gap(&self, gap_coefficient: f32, speed: f32) -> f32 {
        let min_gap =
            (self.width * speed + self.type_config.min_gap * gap_coefficient).round() as i32;
        let max_gap = ((min_gap as f32) * MAX_GAP_COEFFICIENT).round() as i32;
        gen_range(min_gap, max_gap + 1) as f32
    }

    /// Replaces `Obstacle.prototype.isVisible()`
    pub fn is_visible(&self) -> bool {
        self.x_pos + self.width > 0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObstacleKind {
    CactusSmall,
    CactusLarge,
    Pterodactyl,
}

#[derive(Clone, Copy, Debug)]
pub struct ObstacleType {
    pub kind: ObstacleKind,
    pub width: f32,
    pub height: f32,
    pub y_pos: &'static [f32],
    pub multiple_speed: f32,
    pub min_gap: f32,
    pub min_speed: f32,
    pub num_frames: Option<usize>,
    pub frame_rate: Option<f32>,
    pub speed_offset: Option<f32>,
    pub collision_boxes: &'static [CollisionBox],
}

/// Replaces `Obstacle.types`
pub const OBSTACLE_TYPES: [ObstacleType; 3] = [
    // 0: CACTUS_SMALL
    ObstacleType {
        kind: ObstacleKind::CactusSmall,
        width: 17.0,
        height: 35.0,
        y_pos: &[105.0],
        multiple_speed: 4.0,
        min_gap: 120.0,
        min_speed: 0.0,
        num_frames: None,
        frame_rate: None,
        speed_offset: None,
        collision_boxes: &[
            CollisionBox::new(0.0, 7.0, 5.0, 27.0),
            CollisionBox::new(4.0, 0.0, 6.0, 34.0),
            CollisionBox::new(10.0, 4.0, 7.0, 14.0),
        ],
    },
    // 1: CACTUS_LARGE
    ObstacleType {
        kind: ObstacleKind::CactusLarge,
        width: 25.0,
        height: 50.0,
        y_pos: &[90.0],
        multiple_speed: 7.0,
        min_gap: 120.0,
        min_speed: 0.0,
        num_frames: None,
        frame_rate: None,
        speed_offset: None,
        collision_boxes: &[
            CollisionBox::new(0.0, 12.0, 7.0, 38.0),
            CollisionBox::new(8.0, 0.0, 7.0, 49.0),
            CollisionBox::new(13.0, 10.0, 10.0, 38.0),
        ],
    },
    // 2: PTERODACTYL
    ObstacleType {
        kind: ObstacleKind::Pterodactyl,
        width: 46.0,
        height: 40.0,
        y_pos: &[100.0, 75.0, 50.0],
        multiple_speed: 999.0,
        min_speed: 8.5,
        min_gap: 150.0,
        num_frames: Some(2),
        frame_rate: Some(1000.0 / 6.0),
        speed_offset: Some(0.8),
        collision_boxes: &[
            CollisionBox::new(15.0, 15.0, 16.0, 5.0),
            CollisionBox::new(18.0, 21.0, 24.0, 6.0),
            CollisionBox::new(2.0, 14.0, 4.0, 3.0),
            CollisionBox::new(6.0, 10.0, 4.0, 7.0),
            CollisionBox::new(10.0, 8.0, 6.0, 9.0),
        ],
    },
];

// ==========================================
// 1. Animation States (Replaces Trex.status)
// ==========================================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrexStatus {
    Crashed,
    Ducking,
    Jumping,
    Running,
    Waiting,
}

// ==========================================
// 2. T-Rex Config (Replaces Trex.config)
// ==========================================
#[derive(Clone, Copy, Debug)]
pub struct TrexConfig {
    pub drop_velocity: f32,
    pub gravity: f32,
    pub height: f32,
    pub height_duck: f32,
    pub initial_jump_velocity: f32,
    pub intro_duration: f32,
    pub max_jump_height: f32,
    pub min_jump_height: f32,
    pub speed_drop_coefficient: f32,
    pub sprite_width: f32,
    pub start_x_pos: f32,
    pub width: f32,
    pub width_duck: f32,
}

impl Default for TrexConfig {
    fn default() -> Self {
        Self {
            drop_velocity: -5.0,
            gravity: 0.6,
            height: 47.0,
            height_duck: 25.0,
            initial_jump_velocity: -10.0,
            intro_duration: 1500.0,
            max_jump_height: 30.0,
            min_jump_height: 30.0,
            speed_drop_coefficient: 3.0,
            sprite_width: 262.0,
            start_x_pos: 50.0,
            width: 44.0,
            width_duck: 59.0,
        }
    }
}

// ==========================================
// 3. Trex Struct (Replaces function Trex)
// ==========================================
#[derive(Clone, Debug)]
pub struct Trex {
    pub sprite_pos: Vec2,
    pub x_pos: f32,
    pub y_pos: f32,
    pub ground_y_pos: f32,
    pub current_frame: usize,
    pub current_anim_frames: &'static [f32],
    pub blink_delay: f32,
    pub blink_count: u32,
    pub anim_start_time: f32,
    pub timer: f32,
    pub ms_per_frame: f32,
    pub config: TrexConfig,
    pub status: TrexStatus,

    pub jumping: bool,
    pub ducking: bool,
    pub jump_velocity: f32,
    pub reached_min_height: bool,
    pub speed_drop: bool,
    pub jump_count: u32,
    pub jumpspot_x: f32,
    pub playing_intro: bool,
    pub min_jump_height: f32,
}

impl Trex {
    /// Replaces `Trex.collisionBoxes.DUCKING`
    pub const COLLISION_BOXES_DUCKING: &'static [CollisionBox] =
        &[CollisionBox::new(1.0, 18.0, 55.0, 25.0)];

    /// Replaces `Trex.collisionBoxes.RUNNING`
    pub const COLLISION_BOXES_RUNNING: &'static [CollisionBox] = &[
        CollisionBox::new(22.0, 0.0, 17.0, 16.0),
        CollisionBox::new(1.0, 18.0, 30.0, 9.0),
        CollisionBox::new(10.0, 35.0, 14.0, 8.0),
        CollisionBox::new(1.0, 24.0, 29.0, 5.0),
        CollisionBox::new(5.0, 30.0, 21.0, 4.0),
        CollisionBox::new(9.0, 34.0, 15.0, 4.0),
    ];

    /// Replaces `new Trex(canvas, spritePos)`
    pub fn new(sprite_pos: Vec2) -> Self {
        let config = TrexConfig::default();
        let mut trex = Self {
            sprite_pos,
            x_pos: 0.0,
            y_pos: 0.0,
            ground_y_pos: 0.0,
            current_frame: 0,
            current_anim_frames: &[],
            blink_delay: 0.0,
            blink_count: 0,
            anim_start_time: 0.0,
            timer: 0.0,
            ms_per_frame: 1000.0 / 60.0,
            config,
            status: TrexStatus::Waiting,
            jumping: false,
            ducking: false,
            jump_velocity: 0.0,
            reached_min_height: false,
            speed_drop: false,
            jump_count: 0,
            jumpspot_x: 0.0,
            playing_intro: false,
            min_jump_height: 0.0,
        };

        trex.init();
        trex
    }

    pub fn width(&self) -> f32 {
        self.config.width
    }

    pub fn height(&self) -> f32 {
        self.config.height
    }

    /// Replaces `Trex.prototype.init()`
    pub fn init(&mut self) {
        // DEFAULT_HEIGHT (150.0) - HEIGHT (47.0) - BOTTOM_PAD (10.0) = 93.0
        self.ground_y_pos = 150.0 - self.config.height - 10.0;
        self.y_pos = self.ground_y_pos;
        self.min_jump_height = self.ground_y_pos - self.config.min_jump_height;

        self.update(0.0, Some(TrexStatus::Waiting));
    }

    /// Replaces `Trex.prototype.setJumpVelocity(setting)`
    pub fn set_jump_velocity(&mut self, setting: f32) {
        self.config.initial_jump_velocity = -setting;
        self.config.drop_velocity = -setting / 2.0;
    }

    /// Replaces `Trex.prototype.update(deltaTime, opt_status)`
    pub fn update(&mut self, delta_time: f32, opt_status: Option<TrexStatus>) {
        self.timer += delta_time;

        // Update the status if a new one was passed
        if let Some(status) = opt_status {
            self.status = status;
            self.current_frame = 0;
            let anim = status.anim_frames();
            self.ms_per_frame = anim.ms_per_frame;
            self.current_anim_frames = anim.frames;

            if status == TrexStatus::Waiting {
                self.anim_start_time = (get_time() * 1000.0) as f32;
                self.set_blink_delay();
            }
        }

        // Game intro animation: T-Rex slides in from x = 0 to START_X_POS (50)
        if self.playing_intro && self.x_pos < self.config.start_x_pos {
            self.x_pos += ((self.config.start_x_pos / self.config.intro_duration) * delta_time)
                .round();
        }

        if self.status == TrexStatus::Waiting {
            self.blink((get_time() * 1000.0) as f32);
        }

        // Advance animation frame index
        if self.timer >= self.ms_per_frame {
            self.current_frame = if self.current_frame == self.current_anim_frames.len() - 1 {
                0
            } else {
                self.current_frame + 1
            };
            self.timer = 0.0;
        }

        // Speed drop becomes duck if the down key is still being pressed on landing
        if self.speed_drop && self.y_pos == self.ground_y_pos {
            self.speed_drop = false;
            self.set_duck(true);
        }
    }

    /// Replaces `Trex.prototype.draw(x, y)`
    /// Call this from `Runner::draw()` with `y_offset = 25.0`
    pub fn draw(&self, sprite: &Texture2D, is_hidpi: bool, y_offset: f32) {
        // Pick the current animation frame X offset
        let frame_x = if self.status == TrexStatus::Waiting {
            // While waiting, only show the closed-eye frame (index 0) when blink delay has elapsed
            let now_ms = (macroquad::time::get_time() * 1000.0) as f32;
            if now_ms - self.anim_start_time >= self.blink_delay {
                self.current_anim_frames.get(self.current_frame).copied().unwrap_or(0.0)
            } else {
                0.0 // Eyes open frame
            }
        } else {
            self.current_anim_frames
                .get(self.current_frame)
                .copied()
                .unwrap_or(0.0)
        };

        let mut source_x = frame_x;
        let mut source_y = 0.0;
        let is_ducking_sprite = self.ducking && self.status != TrexStatus::Crashed;
        let mut source_width = if is_ducking_sprite {
            self.config.width_duck
        } else {
            self.config.width
        };
        let mut source_height = self.config.height;

        if is_hidpi {
            source_x *= 2.0;
            source_y *= 2.0;
            source_width *= 2.0;
            source_height *= 2.0;
        }

        // Adjustments for sprite sheet position
        source_x += self.sprite_pos.x;
        source_y += self.sprite_pos.y;

        if is_ducking_sprite {
            // Ducking
            draw_texture_ex(
                sprite,
                self.x_pos,
                self.y_pos + y_offset,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(source_x, source_y, source_width, source_height)),
                    dest_size: Some(vec2(self.config.width_duck, self.config.height)),
                    ..Default::default()
                },
            );
        } else {
            // Crashed whilst ducking: T-Rex stands up so shift X by +1px
            let draw_x = if self.ducking && self.status == TrexStatus::Crashed {
                self.x_pos + 1.0
            } else {
                self.x_pos
            };

            // Standing / running / jumping / crashed
            draw_texture_ex(
                sprite,
                draw_x,
                self.y_pos + y_offset,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(source_x, source_y, source_width, source_height)),
                    dest_size: Some(vec2(self.config.width, self.config.height)),
                    ..Default::default()
                },
            );
        }
    }

    /// Replaces `Trex.prototype.setBlinkDelay()`
    pub fn set_blink_delay(&mut self) {
        self.blink_delay = (gen_range(0.0_f32, 1.0_f32) * BLINK_TIMING).ceil();
    }

    /// Replaces `Trex.prototype.blink(time)`
    pub fn blink(&mut self, time: f32) {
        let delta_time = time - self.anim_start_time;

        if delta_time >= self.blink_delay && self.current_frame == 1 {
            // Set new random delay for the next blink
            self.set_blink_delay();
            self.anim_start_time = time;
            self.blink_count += 1;
        }
    }

    /// Replaces `Trex.prototype.startJump(speed)`
    pub fn start_jump(&mut self, speed: f32) {
        if !self.jumping {
            self.update(0.0, Some(TrexStatus::Jumping));
            // Tweak the jump velocity based on the current speed
            self.jump_velocity = self.config.initial_jump_velocity - (speed / 10.0);
            self.jumping = true;
            self.reached_min_height = false;
            self.speed_drop = false;
        }
    }

    /// Replaces `Trex.prototype.endJump()`
    pub fn end_jump(&mut self) {
        if self.reached_min_height && self.jump_velocity < self.config.drop_velocity {
            self.jump_velocity = self.config.drop_velocity;
        }
    }

    pub fn update_jump(&mut self, delta_time: f64, _speed: f64) {
        let ms_per_frame = self
            .anim_frames
            .get(&self.status)
            .map(|anim| anim.ms_per_frame)
            .unwrap_or(1000.0 / 60.0);

        let frames_elapsed = delta_time / ms_per_frame;

        // Speed drop makes Trex fall faster.
        if self.speed_drop {
            self.y_pos += (self.jump_velocity * self.config.speed_drop_coefficient * frames_elapsed)
                .round() as i32;
        } else {
            self.y_pos += (self.jump_velocity * frames_elapsed).round() as i32;
        }

        self.jump_velocity += self.config.gravity * frames_elapsed;

        // Minimum height has been reached.
        if self.y_pos < self.min_jump_height || self.speed_drop {
            self.reached_min_height = true;
        }

        // Reached max height.
        if self.y_pos < self.config.max_jump_height || self.speed_drop {
            self.end_jump();
        }

        // Back down at ground level. Jump completed.
        if self.y_pos > self.ground_y_pos {
            self.reset();
            self.jump_count += 1;
        }

        self.update(delta_time, None);
    }

    /// Set the speed drop. Immediately cancels the current jump.
    pub fn set_speed_drop(&mut self) {
        self.speed_drop = true;
        self.jump_velocity = 1.0;
    }

    /// Toggle ducking state.
    pub fn set_duck(&mut self, is_ducking: bool) {
        if is_ducking && self.status != TrexStatus::Ducking {
            self.update(0.0, Some(TrexStatus::Ducking));
            self.ducking = true;
        } else if self.status == TrexStatus::Ducking {
            self.update(0.0, Some(TrexStatus::Running));
            self.ducking = false;
        }
    }

    /// Reset the t-rex to running at start of game.
    pub fn reset(&mut self) {
        self.y_pos = self.ground_y_pos;
        self.jump_velocity = 0.0;
        self.jumping = false;
        self.ducking = false;
        self.update(0.0, Some(TrexStatus::Running));
        self.midair = false;
        self.speed_drop = false;
        self.jump_count = 0;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AnimFrames {
    pub frames: &'static [f32],
    pub ms_per_frame: f32,
}

impl TrexStatus {
    /// Replaces `Trex.animFrames`
    pub const fn anim_frames(self) -> AnimFrames {
        match self {
            TrexStatus::Waiting => AnimFrames {
                frames: &[44.0, 0.0],
                ms_per_frame: 1000.0 / 3.0,
            },
            TrexStatus::Running => AnimFrames {
                frames: &[88.0, 132.0],
                ms_per_frame: 1000.0 / 12.0,
            },
            TrexStatus::Crashed => AnimFrames {
                frames: &[220.0],
                ms_per_frame: 1000.0 / 60.0,
            },
            TrexStatus::Jumping => AnimFrames {
                frames: &[0.0],
                ms_per_frame: 1000.0 / 60.0,
            },
            TrexStatus::Ducking => AnimFrames {
                frames: &[264.0, 323.0],
                ms_per_frame: 1000.0 / 8.0,
            },
        }
    }
}

