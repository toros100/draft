use draft::construction::object::ObjectId;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

use slint::{Model, ModelRc, language::PointerEventKind};

use crate::construction::expression::ExpressionObj;

use draft::construction;
use draft::geom;
use draft::model;

fn main() -> Result<(), slint::PlatformError> {
    let main_window = draft::MainWindow::new()?;

    debug_assert_eq!(
        main_window.global::<draft::Id>().get_NONE(),
        draft::slint_conv::ID_NONE,
        "none sentinel mismatch"
    );

    let am = Rc::new(model::ObjectModel::default());

    #[allow(unused)]
    {
        let mut a = am.arena_mut();
        let p1 = a.add_root(geom::point2(200., 200.));
        let p2 = a.add_root(geom::point2(400., 200.));
        let p3 = a.add_curve(p1, p2);
        let dist = ExpressionObj::Mul(
            ExpressionObj::Scalar(0.5).into(),
            ExpressionObj::CurveLength(p3).into(),
        );

        let p4 = a.add_point_on_curve(p3, dist);

        let p5 = a.add_relative_point(
            p2,
            construction::expression::ExpressionObj::Length(100.),
            construction::expression::ExpressionObj::Angle(PI / 2.),
        );

        let p6 = a.add_point_midway(p1, p5);
        a.add_line(p1, p6);

        a.evaluate_all();
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
                am.arena_mut().evaluate_all();
                am.reset();
            }
            _ => {}
        }
    });

    am.reset();

    let point_model = model::filter_map(am.clone());
    println!("num points: {}", point_model.row_count());
    let line_model = model::filter_map(am.clone());
    println!("num lines: {}", line_model.row_count());
    let curve_model = model::filter_map(am.clone());
    println!("num curves: {}", curve_model.row_count());
    let curve_controls_model = model::filter_map(am.clone());

    main_window.set_points(ModelRc::new(point_model));
    main_window.set_lines(ModelRc::new(line_model));
    main_window.set_curves(ModelRc::new(curve_model));
    main_window.set_curve_controls(ModelRc::new(curve_controls_model));

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
