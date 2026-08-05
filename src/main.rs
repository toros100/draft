use slint::ComponentHandle;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

use slint::{Model, ModelRc, language::PointerEventKind};

use crate::construction::object::ObjectId;

use draft::construction;
use draft::geom;
use draft::model;

fn main() -> Result<(), slint::PlatformError> {
    let main_window = draft::MainWindow::new()?;

    debug_assert!(
        Option::<ObjectId>::from(main_window.global::<draft::Id>().get_NONE()).is_none(),
        "none sentinel mismatch"
    );

    let am = Rc::new(model::ObjectModel::default());

    {
        let mut a = am.arena_mut();
        let p1 = a.add_point(geom::point2(200., 200.));
        let p2 = a.add_point(geom::point2(400., 200.));
        let p3 = a.add_curve(p1, p2);

        let p4 = a.add_relative_point(
            p2,
            construction::expression::ExpressionObj::Length(100.),
            construction::expression::ExpressionObj::Angle(PI / 2.),
        );

        let p5 = a.add_point_midway(p1, p4);
        a.add_line(p1, p4);

        a.calculate_all();
    }

    let ps = PointerState::default();

    main_window.on_pointer_event({
        let mut ps = ps.clone();
        let am = am.clone();
        let w = main_window.as_weak();

        move |e, p| match e {
            PointerEventKind::Up | PointerEventKind::Cancel => ps.up(),
            PointerEventKind::Down => ps.down(p.into()),
            PointerEventKind::Move if ps.is_up() => {
                let pos = p.into();
                let res = am.arena_mut().hit_scan(pos, 12.);

                ps.obj(res);
                w.unwrap().set_hover_id(res.into());
            }
            PointerEventKind::Move if let Some(a) = ps.dragging() => {
                let p = geom::Point2::from(p);
                // TODO: need a way to distinguish object kind
                am.arena_mut().drag_to(a, p);

                // NOTE: just doing a full refresh on any change, should to partial refresh using
                // dirty flag and cache later
                am.arena_mut().calculate_all();
                am.reset();
            }
            _ => {}
        }
    });

    am.reset();

    let point_model = model::points_model(am.clone());
    println!("num points: {}", point_model.row_count());
    let line_model = model::lines_model(am.clone());
    println!("num lines: {}", line_model.row_count());
    let curve_model = model::curves_model(am.clone());
    println!("num curves: {}", curve_model.row_count());
    let curve_controls_mnodel = model::curve_controls_model(am.clone());
    main_window.set_points(ModelRc::new(point_model));
    main_window.set_lines(ModelRc::new(line_model));
    main_window.set_curves(ModelRc::new(curve_model));
    main_window.set_curve_controls(ModelRc::new(curve_controls_mnodel));
    main_window.run()
}

#[derive(Default)]
enum Pointer {
    #[default]
    Up,
    Down,
}

/// i hate this, not cool, not nice
/// need better tracking of state, especially for "tool usage"
/// e.g. adding a line (selecting two points)
#[derive(Default, Clone)]
struct PointerState {
    inner: Rc<RefCell<PointerStateInner>>,
}

#[derive(Default)]
struct PointerStateInner {
    obj: Option<ObjectId>,
    drag_start: Option<geom::Point2>,
    state: Pointer,
}

impl PointerState {
    fn is_up(&self) -> bool {
        matches!(self.inner.borrow().state, Pointer::Up)
    }

    fn dragging(&self) -> Option<ObjectId> {
        let b = self.inner.borrow();
        b.obj
    }

    fn obj(&mut self, opt: Option<ObjectId>) {
        self.inner.borrow_mut().obj = opt;
    }

    fn down(&mut self, pos: geom::Point2) {
        self.inner.borrow_mut().state = Pointer::Down;
        self.inner.borrow_mut().drag_start = Some(pos);
    }

    fn up(&mut self) {
        self.inner.borrow_mut().state = Pointer::Up;
        self.inner.borrow_mut().drag_start = None;
    }
}
