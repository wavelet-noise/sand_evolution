use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use cgmath::{InnerSpace, Matrix, Matrix2, Matrix3, SquareMatrix, Vector2, Vector3};
use rhai::EvalAltResult;
use crate::CellTypeNotFound;
use crate::shared_state::SharedState;

pub fn register_rhai(rhai: &mut rhai::Engine, scope: &mut rhai::Scope, shared_state_rc: Rc<RefCell<SharedState>>, id_dict: HashMap<String, u8>) {
    //rhai_scope.push_constant("RES_X", dimensions.0);
    //rhai_scope.push_constant("RES_Y", dimensions.1);

    {
        let moved_clone = shared_state_rc.clone();
        rhai.register_fn("set_cell", move |x: i64, y: i64, t: i64| {
            moved_clone
                .borrow_mut()
                .set_pixel(x as i32, y as i32, t as u8);
        });
    }
    {
        let moved_clone = shared_state_rc.clone();
        rhai.register_fn("set_cell", move |x: f64, y: f64, t: i64| {
            moved_clone
                .borrow_mut()
                .set_pixel(x as i32, y as i32, t as u8);
        });
    }
    {
        let moved_clone = shared_state_rc.clone();
        rhai.register_fn("set_cell", move |v: Vector2::<f64>, t: i64| {
            moved_clone
                .borrow_mut()
                .set_pixel(v.x as i32, v.y as i32, t as u8);
        });
    }
    {
        rhai.register_fn("draw_line", move |v1: Vector2::<f64>, v2: Vector2::<f64>, t: i64| {
            draw_line(v1, v2, t as u8, shared_state_rc.clone());
        });
    }
    rhai.register_fn("type_id", move |name: &str| -> Result<i64, Box<rhai::EvalAltResult>> {
        if id_dict.contains_key(name) {
            Ok(id_dict[name] as i64)
        } else {
            Err(EvalAltResult::ErrorSystem(
                "SystemError".into(),
                Box::new(CellTypeNotFound{name: name.to_string()})
            ).into())
        }
    });
    rhai.register_fn("fract", move |v: f64| { v.fract() });
    rhai.register_fn("rand", move || -> i64 { crate::random::my_rand() });
    scope.push("time", 0f64);

    rhai
        .register_type::<Vector2<f64>>()
        .register_fn("vec2", Vector2::<f64>::new)
        .register_set("x", |v: &mut Vector2<f64>, x: f64| v.x = x)
        .register_set("y", |v: &mut Vector2<f64>, y: f64| v.y = y)
        .register_get("x", |v: &mut Vector2<f64>| v.x)
        .register_get("y", |v: &mut Vector2<f64>| v.y)
        .register_fn("+", |a: Vector2<f64>, b: Vector2<f64>| a + b)
        .register_fn("-", |a: Vector2<f64>, b: Vector2<f64>| a - b)
        .register_fn("*", |v: Vector2<f64>, scalar: f64| v * scalar)
        .register_fn("magnitude", |v: &mut Vector2<f64>| v.magnitude())
        .register_fn("normalize", |v: &mut Vector2<f64>| v.normalize());

    rhai
        .register_type::<Vector3<f64>>()
        .register_fn("vec3", Vector3::<f64>::new)
        .register_set("x", |v: &mut Vector3<f64>, x: f64| v.x = x)
        .register_set("y", |v: &mut Vector3<f64>, y: f64| v.y = y)
        .register_set("z", |v: &mut Vector3<f64>, z: f64| v.z = z)
        .register_get("x", |v: &mut Vector3<f64>| v.x)
        .register_get("y", |v: &mut Vector3<f64>| v.y)
        .register_get("z", |v: &mut Vector3<f64>| v.z)
        .register_fn("+", |a: Vector3<f64>, b: Vector3<f64>| a + b)
        .register_fn("-", |a: Vector3<f64>, b: Vector3<f64>| a - b)
        .register_fn("*", |v: Vector3<f64>, scalar: f64| v * scalar)
        .register_fn("magnitude", |v: &mut Vector3<f64>| v.magnitude())
        .register_fn("normalize", |v: &mut Vector3<f64>| v.normalize())
        .register_fn("dot", |a: Vector3<f64>, b: Vector3<f64>| a.dot(b))
        .register_fn("cross", |a: Vector3<f64>, b: Vector3<f64>| a.cross(b));

    rhai
        .register_type::<Matrix2<f64>>()
        .register_fn("mat2", Matrix2::<f64>::new)
        .register_fn("*", |m: Matrix2<f64>, v: Vector2<f64>| m * v)
        .register_fn("*", |a: Matrix2<f64>, b: Matrix2<f64>| a * b)
        .register_fn("transpose", |a: Matrix2<f64>| a.transpose())
        .register_fn("invert", |a: Matrix2<f64>| a.invert());

    rhai
        .register_type::<Matrix3<f64>>()
        .register_fn("mat3", Matrix3::<f64>::new)
        .register_fn("*", |m: Matrix3<f64>, v: Vector3<f64>| m * v)
        .register_fn("*", |a: Matrix3<f64>, b: Matrix3<f64>| a * b)
        .register_fn("transpose", |a: Matrix3<f64>| a.transpose())
        .register_fn("invert", |a: Matrix3<f64>| a.invert());
}

fn draw_line(start: Vector2<f64>, end: Vector2<f64>, t: u8, state: Rc<RefCell<SharedState>>)
{
    let mut x0 = start.x as i64;
    let mut y0 = start.y as i64;
    let x1 = end.x as i64;
    let y1 = end.y as i64;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();

    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    let mut err = dx - dy;

    loop {
        state.borrow_mut().set_pixel(x0 as i32, y0 as i32, t);

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;

        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }
}