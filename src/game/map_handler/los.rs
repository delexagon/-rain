use super::{Traverser, GameData, UITile};
use crate::common::{Rgb, Fg, Array2D};

// Assumes the arrays are the same size
pub fn transform_uitile(ui_array: &mut Array2D<UITile>, array: &Array2D<Option<Traverser>>, data: &mut GameData) {
    for c in ui_array.coord_iter() {
        match array[c] {
            Some(traverser) => ui_array[c] = data.ui.ui_tile(traverser.tile(), &data.world, &data.entities),
            None => ui_array[c] = UITile {fg: Fg {ch: ' ', bold: false, ital: false, color: Rgb(255,255,255)}, bg: Rgb(0,0,0)}
        }
    }
}

pub fn advance(t: Option<Traverser>, d: u8, data: &mut GameData) -> Option<Traverser> {
    let t = t?;
    if data.object_see_through(t.tile) {
        return data.load_travel(t, d);
    }
    None
}

#[derive(Copy, Clone)]
struct Ray {
    numer: u16,
    denom: u16,
    count: u16
}

#[derive(Copy, Clone)]
struct Cutoff {
    x: Ray,
    y: Ray,
    just_sidled: bool
}
impl Default for Cutoff {
    fn default() -> Self {
        Self {
            x: Ray {numer: 0, denom: 0, count: 0},
            y: Ray {numer: 0, denom: 0, count: 0},
            just_sidled: false
        }
    }
}

// x >= y. Along is incoming on x direction, sidle is incoming on y direction.
fn next_val(x: u16, y: u16, along_in: Option<Cutoff>, sidle_in: Option<Cutoff>, same_tile: bool) -> Option<(bool, Cutoff)> {
    // TODO: This doesn't work right yet. pls to fix
    let sidle_out = if let Some(sidle) = sidle_in {
        let next_x = if sidle.x.count + sidle.x.numer > sidle.x.denom {
            Ray {numer: sidle.x.numer, denom: sidle.x.denom, count: sidle.x.count+sidle.x.numer}
        } else {
            Ray {numer: 0, denom: 0, count: 0}
        };
        if sidle.y.denom == 0 {
            Some(Cutoff {
                x: next_x,
                y: Ray {numer: y, denom: x, count: 0},
                just_sidled: true
            })
        } else if sidle.y.count >= sidle.y.denom {
            Some(Cutoff {
                x: next_x,
                y: Ray {numer: sidle.y.numer, denom: sidle.y.denom, count: sidle.y.count - sidle.y.denom},
                just_sidled: true
            })
        } else {
            None
        }
    } else {
        None
    };
    let along_out = if let Some(along) = along_in {
        // We have one tile along with a greater/equal count, which spawns the next sidle.
        // After, we go back to any
        let next_y = if along.y.denom == 0 || along.y.count >= along.y.denom {
            Ray {numer: 0, denom: 0, count: 0}
        } else {
            Ray {numer: along.y.numer, denom: along.y.denom, count: along.y.count+along.y.numer}
        };
        if same_tile && sidle_out.is_some() {
            Some(Cutoff {
                x: Ray {numer: 0, denom: 0, count: 0},
                y: next_y,
                just_sidled: false
            })
        } else if along.x.denom == 0 {
            Some(Cutoff {
                x: Ray {numer: y, denom: x, count: 0},
                y: next_y,
                just_sidled: false
            })
        } else if along.x.count + along.x.numer <= along.x.denom {
            Some(Cutoff {
                x: Ray {numer: along.x.numer, denom: along.x.denom, count: along.x.count+along.x.numer},
                y: next_y,
                just_sidled: false
            })
        } else if along.just_sidled && along.x.count > along.x.denom {
            Some(Cutoff {
                x: Ray {numer: along.x.numer, denom: along.x.denom, count: along.x.count%along.x.denom},
                y: next_y,
                just_sidled: false
            })
        } else {
            None
        }
    } else {
        None
    };
    if let Some(a) = along_out {
        return Some((true, a));
    } else if let Some(s) = sidle_out {
        return Some((false, s));
    } else {
        return None;
    }
}

pub fn los_scan(t_arr: &mut Array2D<Option<Traverser>>, t: Traverser, data: &mut GameData) {
    t_arr.fill(None);
    let height = t_arr.height();
    let width = t_arr.width();
    let mut c_arr = Array2D::new_sized(width,height,None);
    let center = t_arr.center();
    t_arr[center] = Some(t);
    c_arr[center] = Some(Cutoff::default());
    
    for (here, prev, dir) in t_arr.linear_iter(t_arr.center()) {
        t_arr[here] = advance(t_arr[prev], dir, data);
        if t_arr[here].is_some() {
            c_arr[here] = Some(Cutoff::default());
        }
    }
    for (here, offset, prev_a, dir_a, prev_s, dir_s) in t_arr.quad_iter(t_arr.center()) {
        let a = advance(t_arr[prev_a], dir_a, data);
        let s = advance(t_arr[prev_s], dir_s, data);
        let same_tile = if let (Some(t1), Some(t2)) = (a,s) {t1.same_tile(&t2)} else {false};
        let v = next_val(
            offset.0 as u16, offset.1 as u16, 
            if a.is_some() {c_arr[prev_a]} else {None},
            if s.is_some() {c_arr[prev_s]} else {None},
            same_tile
        );
        if let Some((is_along, cutoff)) = v {
            if is_along {
                t_arr[here] = a;
            } else {
                t_arr[here] = s;
            }
            c_arr[here] = Some(cutoff);
        } else {
            t_arr[here] = None;
            c_arr[here] = None;
        }
    }
}
