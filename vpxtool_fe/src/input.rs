// src/input/shift_handler.rs
use std::time::{Duration, Instant};

const INITIAL_SHIFT_SPEED: f32 = 1.0;
const RAMP_UP_DURATION: f32 = 10.0; // seconds until max speed
const MAX_SHIFT_SPEED: f32 = 200.0; // Maximum jump per second

const DEFAULT_ALPHA_JUMP_DEBOUNCE: Duration = Duration::from_millis(200);

/// Handles shift key presses to change the current table index
/// The shift key can be held down to scroll through the tables
/// The speed of the scroll increases over time
pub struct ShiftHandler {
    shift_start: Option<Instant>,
    shift_applied: f32,
}

impl ShiftHandler {
    pub fn new() -> Self {
        Self {
            shift_start: None,
            shift_applied: 0.0,
        }
    }

    fn handle_shift(&mut self, now: Instant, delta_time: Duration, direction: i16) -> i16 {
        if self.shift_start.is_none() {
            self.shift_start = Some(now);
            return direction;
        }

        let held_duration = now.duration_since(self.shift_start.unwrap()).as_secs_f32();
        let speed = (INITIAL_SHIFT_SPEED
            + (held_duration / RAMP_UP_DURATION) * (MAX_SHIFT_SPEED - INITIAL_SHIFT_SPEED))
            .min(MAX_SHIFT_SPEED);

        self.shift_applied += delta_time.as_secs_f32() * speed * direction as f32;
        if self.shift_applied.abs() >= 1.0 {
            let index_change = self.shift_applied.floor() as i16 * direction;
            self.shift_applied = self.shift_applied.fract();
            return index_change;
        }

        0
    }

    pub fn update(
        &mut self,
        now: Instant,
        delta_time: Duration,
        keystates: &sdl3::keyboard::KeyboardState,
    ) -> i16 {
        if keystates.is_scancode_pressed(sdl3::keyboard::Scancode::LShift) {
            self.handle_shift(now, delta_time, -1)
        } else if keystates.is_scancode_pressed(sdl3::keyboard::Scancode::RShift) {
            self.handle_shift(now, delta_time, 1)
        } else {
            self.shift_start = None;
            self.shift_applied = 0.0;
            0
        }
    }
}

/// Handles alphabetic jumps between tables
pub struct AlphabeticJumper {
    last_jump_time: Instant,
    // Debounce period to prevent double jumps
    debounce_period: Duration,
}

impl AlphabeticJumper {
    pub fn new() -> Self {
        Self {
            last_jump_time: Instant::now(),
            debounce_period: DEFAULT_ALPHA_JUMP_DEBOUNCE,
        }
    }

    pub fn handle_jump<F>(
        &mut self,
        tables: &[vpxtool_shared::indexer::IndexedTable],
        current_index: usize,
        direction: i8, // 1 for forward, -1 for backward
        get_first_letter: F,
    ) -> Option<usize>
    where
        F: Fn(&vpxtool_shared::indexer::IndexedTable) -> char,
    {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_jump_time) < self.debounce_period {
            return None;
        }

        // Get current table's first letter
        let current_table = &tables[current_index];
        let current_letter = get_first_letter(current_table);
        let table_count = tables.len();

        if direction > 0 {
            // Forward jump: Find the first table starting with a different letter
            let mut new_index = current_index;

            for _ in 0..table_count {
                new_index = (new_index + 1) % table_count;
                let new_letter = get_first_letter(&tables[new_index]);

                if new_letter != current_letter {
                    self.last_jump_time = now;
                    return Some(new_index);
                }
            }
        } else {
            // Backward jump: Find the previous letter group and jump to its first occurrence
            // TODO is this the most intuitive behaviour?

            // First, find the previous letter that's different
            let mut prev_letter = None;
            let mut prev_letter_index = None;

            // Search backward through the tables to find the previous letter group
            for i in 1..=table_count {
                let idx = (current_index + table_count - i) % table_count;
                let letter = get_first_letter(&tables[idx]);

                if letter != current_letter {
                    prev_letter = Some(letter);
                    break;
                }
            }

            // If we found a previous letter, find its first occurrence
            if let Some(target_letter) = prev_letter {
                // Find the first table with this letter when going backward from current
                for i in 1..=table_count {
                    let idx = (current_index + table_count - i) % table_count;
                    let letter = get_first_letter(&tables[idx]);

                    if letter == target_letter {
                        prev_letter_index = Some(idx);
                    } else if prev_letter_index.is_some() && letter != target_letter {
                        // We've gone past the group, so use the last found index
                        break;
                    }
                }

                if let Some(idx) = prev_letter_index {
                    self.last_jump_time = now;
                    return Some(idx);
                }
            }
        }

        None
    }
}
