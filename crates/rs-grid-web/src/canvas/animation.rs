use rs_grid_core::commands::GridCommand;
use rs_grid_scene::builder::FlashHint;

use super::{ActiveDrag, GridCanvas};

impl GridCanvas {
    /// Apply one frame of scroll momentum and decay the velocity.
    ///
    /// Returns `true` while the velocity is still significant
    /// (caller should schedule another rAF frame).
    pub(super) fn step_scroll_momentum(&self) -> bool {
        let vx = self.0.scroll_vx.get();
        let vy = self.0.scroll_vy.get();
        if vx.abs() < 0.5 && vy.abs() < 0.5 {
            self.0.scroll_vx.set(0.0);
            self.0.scroll_vy.set(0.0);
            return false;
        }
        // ~300 ms deceleration at 60 fps.
        const FRICTION: f64 = 0.88;
        self.0.scroll_vx.set(vx * FRICTION);
        self.0.scroll_vy.set(vy * FRICTION);
        // dispatch triggers page fetches for async data sources.
        self.dispatch(GridCommand::ScrollBy { dx: vx, dy: vy });
        true
    }

    /// Lerp `drag_col_offsets` one step toward the preview target.
    ///
    /// Returns `true` while columns have not yet reached their
    /// target positions (caller should schedule another rAF).
    pub(super) fn step_drag_animation(&self) -> bool {
        // Only active during a ColumnDrag.
        let (src, vx) = {
            let drag = self.0.drag.borrow();
            match *drag {
                Some(ActiveDrag::ColumnDrag {
                    col_idx,
                    current_vx,
                    ..
                }) => (col_idx, current_vx),
                _ => return false,
            }
        };

        let ins = self.insertion_index(vx);

        // Compute target offsets for the prospective drop position.
        let target: Vec<f64> = {
            let state = self.0.state.borrow();
            let cols = &state.model.columns;
            if src >= cols.len() {
                return false;
            }
            let dst = if ins > src {
                ins.saturating_sub(1)
            } else {
                ins
            };
            let mut order: Vec<usize> =
                (0..cols.len()).filter(|&i| i != src).collect();
            let at = dst.min(order.len());
            order.insert(at, src);
            let mut offs = vec![0.0_f64; cols.len()];
            let mut cum = 0.0_f64;
            for &ci in &order {
                offs[ci] = cum;
                cum += cols[ci].width;
            }
            offs
        };

        let mut anim = self.0.drag_col_offsets.borrow_mut();
        if anim.len() != target.len() {
            // Offsets not yet initialised — use target as starting
            // point (events.rs should have initialised from real
            // positions, but guard against races).
            *anim = target;
            return false;
        }

        // Exponential smoothing — alpha from the theme CSS var
        // `--rs-grid-drag-anim-alpha` (default 0.30 ≈ 200 ms).
        let alpha = self.0.builder.borrow().theme.drag_anim_alpha;
        let mut settled = true;
        for (a, &t) in anim.iter_mut().zip(target.iter()) {
            let diff = t - *a;
            if diff.abs() < 0.5 {
                *a = t;
            } else {
                *a += diff * alpha;
                settled = false;
            }
        }
        !settled
    }

    /// Compute the current flash alpha factor, clearing the flash
    /// state when expired. Returns `None` when no flash is active.
    pub(super) fn compute_flash_hint(&self) -> Option<FlashHint> {
        let mut flash = self.0.flash.borrow_mut();
        let f = flash.as_ref()?;
        let now = web_sys::window()
            .expect("no window")
            .performance()
            .expect("no performance")
            .now();
        let elapsed = now - f.start_ms;
        if elapsed >= f.duration_ms {
            *flash = None;
            return None;
        }
        let alpha_factor = 1.0 - elapsed / f.duration_ms;
        Some(FlashHint { alpha_factor })
    }
}
