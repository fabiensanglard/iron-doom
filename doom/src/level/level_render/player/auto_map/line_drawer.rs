use super::map_geometry::{FrameLine, FramePoint, MapLine};
use super::{AutoMap, AM_HEIGHT, AM_WIDTH};

// Outcodes used in Cohen–Sutherland clipping algorithm
const INSIDE: i32 = 0;
const LEFT: i32 = 1;
const RIGHT: i32 = 2;
const BOTTOM: i32 = 4;
const TOP: i32 = 8;

pub fn draw_mline(auto_map: &mut AutoMap, ml: MapLine, color: u8) {
    let mut fl = FrameLine::default();

    if clip_mline(auto_map, ml, &mut fl) {
        draw_fline(auto_map, fl, color);
    }
}

fn clip_mline(auto_map: &AutoMap, ml: MapLine, fl: &mut FrameLine) -> bool {
    if is_trivially_outside(auto_map, ml) {
        return false;
    }

    fl.a.x = auto_map.cxmtof(ml.a.x);
    fl.a.y = auto_map.cymtof(ml.a.y);
    fl.b.x = auto_map.cxmtof(ml.b.x);
    fl.b.y = auto_map.cymtof(ml.b.y);

    let mut outcode1 = compute_outcode(fl.a.x, fl.a.y);
    let mut outcode2 = compute_outcode(fl.b.x, fl.b.y);

    if outcode1 & outcode2 != 0 {
        return false;
    }

    while outcode1 | outcode2 != 0 {
        let outside = if outcode1 != 0 { outcode1 } else { outcode2 };
        let mut tmp = FramePoint::default();
        let dx;
        let dy;

        if outside & TOP != 0 {
            dy = fl.a.y - fl.b.y;
            dx = fl.b.x - fl.a.x;
            tmp.x = fl.a.x + (dx * (fl.a.y)) / dy;
            tmp.y = 0;
        } else if outside & BOTTOM != 0 {
            dy = fl.a.y - fl.b.y;
            dx = fl.b.x - fl.a.x;
            tmp.x = fl.a.x + (dx * (fl.a.y.wrapping_sub_unsigned(AM_HEIGHT))) / dy;
            tmp.y = (AM_HEIGHT - 1) as i32;
        } else if outside & RIGHT != 0 {
            dy = fl.b.y - fl.a.y;
            dx = fl.b.x - fl.a.x;
            tmp.x = (AM_WIDTH - 1) as i32;
            tmp.y = fl.a.y + (dy * (tmp.x - fl.a.x)) / dx;
        } else if outside & LEFT != 0 {
            dy = fl.b.y - fl.a.y;
            dx = fl.b.x - fl.a.x;
            tmp.y = fl.a.y + (dy * (-fl.a.x)) / dx;
            tmp.x = 0;
        } else {
            tmp.x = 0;
            tmp.y = 0;
        }

        if outside == outcode1 {
            fl.a = tmp;
            outcode1 = compute_outcode(fl.a.x, fl.a.y);
        } else {
            fl.b = tmp;
            outcode2 = compute_outcode(fl.b.x, fl.b.y);
        }

        if outcode1 & outcode2 != 0 {
            return false;
        }
    }

    true
}

fn is_trivially_outside(auto_map: &AutoMap, ml: MapLine) -> bool {
    let mut outcode1 = INSIDE;
    let mut outcode2 = INSIDE;

    if ml.a.y > auto_map.m_y2 {
        outcode1 |= TOP;
    } else if ml.a.y < auto_map.m_y {
        outcode1 |= BOTTOM;
    }
    if ml.b.y > auto_map.m_y2 {
        outcode2 |= TOP;
    } else if ml.b.y < auto_map.m_y {
        outcode2 |= BOTTOM;
    }

    if outcode1 & outcode2 != 0 {
        return true;
    }

    if ml.a.x < auto_map.m_x {
        outcode1 |= LEFT;
    } else if ml.a.x > auto_map.m_x2 {
        outcode1 |= RIGHT;
    }
    if ml.b.x < auto_map.m_x {
        outcode2 |= LEFT;
    } else if ml.b.x > auto_map.m_x2 {
        outcode2 |= RIGHT;
    }

    if outcode1 & outcode2 != 0 {
        return true;
    }

    false
}

fn compute_outcode(x: i32, y: i32) -> i32 {
    let mut outcode = INSIDE;

    if y < 0 {
        outcode |= TOP;
    } else if (y as u32) >= AM_HEIGHT {
        outcode |= BOTTOM;
    }
    if x < 0 {
        outcode |= LEFT;
    } else if (x as u32) >= AM_WIDTH {
        outcode |= RIGHT;
    }

    outcode
}

fn draw_fline(auto_map: &mut AutoMap, fl: FrameLine, color: u8) {
    let dx = fl.b.x - fl.a.x;
    let ax = 2 * dx.abs();
    let sx = if dx < 0 { -1 } else { 1 };

    let dy = fl.b.y - fl.a.y;
    let ay = 2 * dy.abs();
    let sy = if dy < 0 { -1 } else { 1 };

    let mut x = fl.a.x;
    let mut y = fl.a.y;

    if ax > ay {
        let mut d = ay - ax / 2;

        loop {
            if x < 0 || (x as u32) >= AM_WIDTH || y < 0 || (y as u32) >= AM_HEIGHT {
                println!("invalid negative value");
                return;
            }
            auto_map.buf.draw_pixel(x as u32, y as u32, color);
            if x == fl.b.x {
                return;
            }
            if d >= 0 {
                y += sy;
                d -= ax;
            }
            x += sx;
            d += ay;
        }
    } else {
        let mut d = ax - ay / 2;

        loop {
            if x < 0 || (x as u32) >= AM_WIDTH || y < 0 || (y as u32) >= AM_HEIGHT {
                println!("invalid negative value");
                return;
            }
            auto_map.buf.draw_pixel(x as u32, y as u32, color);
            if y == fl.b.y {
                return;
            }
            if d >= 0 {
                x += sx;
                d -= ay;
            }
            y += sy;
            d += ax;
        }
    }
}
