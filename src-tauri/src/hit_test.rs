use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::Deserialize;
use tauri::{Runtime, WebviewWindow};

pub const POLL_INTERVAL: Duration = Duration::from_millis(33);

#[derive(Debug, Clone, Deserialize)]
pub struct AvatarRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub type AvatarRects = Arc<Mutex<Vec<AvatarRect>>>;
pub type DragFlag = Arc<AtomicBool>;

fn hit(rects: &[AvatarRect], x: f64, y: f64) -> bool {
    rects
        .iter()
        .any(|r| x >= r.x && x <= r.x + r.width && y >= r.y && y <= r.y + r.height)
}

pub fn spawn<R: Runtime>(window: WebviewWindow<R>, rects: AvatarRects, dragging: DragFlag) {
    thread::spawn(move || {
        let mut ignoring = true;

        loop {
            let should_ignore = if dragging.load(Ordering::SeqCst) {
                false
            } else {
                match (
                    window.cursor_position(),
                    window.outer_position(),
                    window.scale_factor(),
                ) {
                    (Ok(cursor), Ok(win_pos), Ok(scale)) => {
                        let rel_x = (cursor.x - win_pos.x as f64) / scale;
                        let rel_y = (cursor.y - win_pos.y as f64) / scale;
                        let current_rects = rects.lock().unwrap();
                        !hit(&current_rects, rel_x, rel_y)
                    }
                    _ => true,
                }
            };

            if should_ignore != ignoring && window.set_ignore_cursor_events(should_ignore).is_ok() {
                ignoring = should_ignore;
            }

            thread::sleep(POLL_INTERVAL);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64, w: f64, h: f64) -> AvatarRect {
        AvatarRect {
            x,
            y,
            width: w,
            height: h,
        }
    }

    #[test]
    fn hits_point_inside_rect() {
        let rects = vec![rect(10.0, 10.0, 64.0, 64.0)];
        assert!(hit(&rects, 20.0, 20.0));
    }

    #[test]
    fn misses_point_outside_all_rects() {
        let rects = vec![rect(10.0, 10.0, 64.0, 64.0)];
        assert!(!hit(&rects, 500.0, 500.0));
    }

    #[test]
    fn hits_within_any_of_multiple_rects() {
        let rects = vec![rect(0.0, 0.0, 10.0, 10.0), rect(100.0, 100.0, 10.0, 10.0)];
        assert!(hit(&rects, 105.0, 105.0));
    }

    #[test]
    fn misses_when_no_rects() {
        let rects: Vec<AvatarRect> = vec![];
        assert!(!hit(&rects, 5.0, 5.0));
    }
}
