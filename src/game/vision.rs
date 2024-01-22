use array2d::Array2D;

use super::gamedata::GameData;
use super::traverser::Traverser;
use crate::common::{UITile, GameMessage, NORMALSTYLE};

// TODO: No validity checking for two places reaching the same tile with conflicting orientations
//  - This means that one will randomly be chosen and create differential results based on rotation,
//    which is unacceptable.
pub fn generate_tiles(height: usize, width: usize, start: Traverser, data: &GameData) -> Array2D<Traverser> {
    let mut array = Array2D::filled_with(Traverser::none(), height, width);
    let center_col = width/2;
    let center_row = height/2;
    let res1 = array.set(center_row, center_col, start);
    // This should never occur unless the array is 0 size
    if res1.is_err() {
        return array;
    }
    // Scan outwards in lines from the center
    for i in 0..4 {
        let mut x = center_col;
        let mut y = center_row;
        let chx;
        let chy;
        if i == 0 {
            chx =  1;
            chy =  0;
        } else if i == 1 {
            chx =  1;
            chy =  2;
        } else if i == 2 {
            chx =  0;
            chy =  1;
        } else {
            chx =  2;
            chy =  1;
        }
        while x < width && y < height && array[(y,x)].tile_exists(&data) {
            let res = array.set(y+chy-1,x+chx-1, array[(y,x)].travel(i, &data));
            x = x+chx-1;
            y = y+chy-1;
            if x == 0 || y == 0 || res.is_err() {
                break;
            }
        }
    }
    
    // Scan the corners
    for i in 0..4 {
        let chx;
        let chy;
        let dirx;
        let diry;
        if i == 0 {
            chx =  2;
            chy =  0;
            dirx = 3;
            diry = 0;
        } else if i == 1 {
            chx =  2;
            chy =  2;
            dirx = 3;
            diry = 1;
        } else if i == 2 {
            chx =  0;
            chy =  0;
            dirx = 2;
            diry = 0;
        } else {
            chx =  0;
            chy =  2;
            dirx = 2;
            diry = 1;
        }
        let mut start_x = center_col+chx-1;
        let mut start_y = center_row+chy-1;
        let mut do_x;
        let mut do_y;
        while start_x < width {
            do_x = start_x;
            do_y = start_y;
            while do_y != center_row && do_x < width {
                let do_t1 = array[(do_y,do_x-chx+1)].travel(dirx, &data);
                let do_t2 = array[(do_y-chy+1,do_x)].travel(diry, &data);
                if do_t1.tile_exists(&data) && do_t2.tile_exists(&data) && do_t1.same_tile(&do_t2) {
                    let res = array.set(do_y,do_x, do_t1.clone());
                    if res.is_err() {
                        break;
                    }
                }
                if do_x == 0 {
                    break;
                }
                do_x = do_x+chx-1;
                do_y = do_y-chy+1;
            }
            if start_y > 0 && start_y < height-1 {
                start_y = start_y+chy-1;
            } else {
                if start_x == 0 {
                    break;
                }
                start_x = start_x+chx-1;
            }
        }
    }
    return array;
}

pub fn traverser_view_msg(t: Traverser, width: usize, height: usize, data: &GameData) -> Vec<UITile> {
    let tarray = generate_tiles(height, width, t, &data);
    let mut msg_vec: Vec<UITile> = Vec::with_capacity(width*height);
    for row in 0..tarray.num_rows() {
        for col in 0..tarray.num_columns() {
            if tarray[(row, col)].tile_exists(&data) {
                msg_vec.push(data.ui_tile(tarray[(row, col)].tile()));
            } else {
                msg_vec.push(UITile {ch: '#', sty: NORMALSTYLE});
            }
        }
    }
    return msg_vec;
}
