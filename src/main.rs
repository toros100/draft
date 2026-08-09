use crate::construction::variants::expression;
use draft::construction::{self, Object, ObjectId};
use draft::{geom, model};
use slint::{ComponentHandle, language::PointerEventKind};
use std::{cell::RefCell, rc::Rc};

fn main() -> Result<(), slint::PlatformError> {
    let main_window = draft::MainWindow::new()?;

    debug_assert_eq!(
        main_window.global::<draft::Id>().get_NONE(),
        draft::slint_conv::ID_NONE,
        "none sentinel mismatch"
    );

    let a = model::ArenaModel::<Object>::default();
    let am = Rc::new(a);

    #[allow(unused)]
    {
        let mut a = am.arena_mut();

        let n = 15usize;

        let left = a.add_root(geom::point2(300., 300.));
        let right = a.add_root(geom::point2(600., 300.));
        let top = a.add_root(geom::point2(450., 100.));
        let c = a.add_curve(left, right);

        let mut crv = Vec::with_capacity(n);
        let mut lns = Vec::with_capacity(n);
        let mut pol = Vec::with_capacity(n);

        for i in 0..=n {
            let cp =
                a.add_point_on_curve(c, expression::curve_length(c) * ((i as f64) / (n as f64)));
            crv.push(cp);
            lns.push(a.add_line(cp, top));
            pol.push(a.add_point_midway(cp, top))
        }

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
                let res = am.arena_mut().hit_scan(pos, 8.);

                ps.obj(res);
                w.unwrap().set_hover_id(res.into());
            }
            PointerEventKind::Move if let Some(a) = ps.dragging() => {
                // HACK:
                let p = geom::point2(p.x as f64, p.y as f64);
                // TODO: need a way to distinguish object kind
                am.arena_mut().drag_to(a, p);

                // NOTE: just doing a full refresh on any change, should to partial refresh using
                // dirty flag and cache later
                am.arena_mut().evaluate_all();
                am.notify_all();
            }
            _ => {}
        }
    });

    main_window.set_points(model::filter_map_model(am.clone()));
    main_window.set_lines(model::filter_map_model(am.clone()));
    main_window.set_curve_controls(model::filter_map_model(am.clone()));
    main_window.set_curves(model::filter_map_model(am.clone()));

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
