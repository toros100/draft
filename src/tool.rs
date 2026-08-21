use crate::construction::{
    Action, Add, CurveControlId, CurveId, Entry, ObjectArena, PointDistAngleId, PointFreeId,
    PointId, PointOnCurveId, PointOnLineId, Update,
};
use crate::expression;
use crate::geom::{self, CubicBezier, Point2};
use crate::render::PathPrimitive;
use slint::language::KeyboardModifiers;
use std::{f64, fmt::Debug};

static_assertions::assert_obj_safe!(Tool);

pub trait Tool {
    // NOTE: either needs ref to arena to retrieve positions of things, or the implementation
    // needs to carefully use cached values to ensure they are still consistent
    // e.g. when adding a point at dist and angle to another point, the parent points position can
    // be stored in self to position the phantom line overlay, but structurally there is nothing
    // preventing the parents position from changing since it got picked up
    fn submit(&mut self, input: ToolInput, arena: &ObjectArena) -> ToolResponse;

    // maybe returning an iterator would be more convenient than a slice
    fn overlay(&self) -> &[PathPrimitive];

    fn reset(&mut self);

    // maybe it would be better to include the cursor kind in the ToolResponse
    fn can_interact(&self, g: Option<&Entry>) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct ToolInput<'a> {
    // position of cursor in world space
    pub cursor: Point2,

    // hovered item, if any. will only ever be an Entry variant that is "geometry"
    // (not a LengthVariable or anything else i might add)
    pub hover: Option<&'a Entry>,

    // mouse action
    pub mouse: Mouse,

    // modifiers that can be used to influence tool behaviour, e.g. snapping angles
    pub modifiers: KeyboardModifiers,
}

#[derive(Default, Debug, Clone)]
pub struct ToolResponse {
    pub done: bool,

    // action to apply to the arena
    // this is technically very powerful, could be anything
    // TODO: when i implement undo/redo, i will have to deal with tools producing a lot of actions
    // (in particular the move tools), will probably end up merging subsequent actions on the same
    // object, e.g. if a position gets updated 1000 times in a row, only the last one matters
    pub action: Option<Action>,

    // notify caller that the tools overlay changed, caller will poll tool.overlay() for rendering
    pub overlay_changed: bool,
}

// why do i use my own struct for the mouse action but use slints KeyboardModifiers?
#[derive(Debug, Clone, Copy)]
pub enum Mouse {
    Press,
    Release,
    Move,
}

pub fn default_boxed<T: Tool + Default + 'static>() -> Box<dyn Tool> {
    Box::new(T::default())
}

fn snap_angle(angle: f64) -> f64 {
    ((angle / std::f64::consts::FRAC_PI_4).round() * std::f64::consts::FRAC_PI_4)
        .rem_euclid(std::f64::consts::TAU)
}

#[derive(Debug, Default)]
pub struct MovePointFree {
    holding: Option<PointFreeId>,
}

impl Tool for MovePointFree {
    fn reset(&mut self) {
        self.holding = None;
    }

    fn overlay(&self) -> &[PathPrimitive] {
        &[]
    }

    fn can_interact(&self, g: Option<&Entry>) -> bool {
        self.holding.is_none() && g.is_some_and(|e| matches!(e, Entry::PointFree(..)))
    }

    fn submit(&mut self, input: ToolInput, _: &ObjectArena) -> ToolResponse {
        match input {
            ToolInput {
                hover: Some(Entry::PointFree(id, ..)),
                mouse: Mouse::Press,
                ..
            } if self.holding.is_none() => {
                _ = self.holding.insert(*id);
                ToolResponse::default()
            }
            ToolInput {
                mouse: Mouse::Release,
                ..
            } => {
                self.holding = None;
                ToolResponse::default()
            }
            ToolInput {
                mouse: Mouse::Move,
                cursor: target,
                ..
            } if let Some(id) = self.holding => ToolResponse {
                action: Some(Action::Update(Update::PointFree {
                    target: id,
                    pos: target,
                })),
                ..Default::default()
            },
            _ => ToolResponse::default(),
        }
    }
}

#[derive(Default)]
pub struct MovePointDistAngle {
    holding: Option<PointDistAngleId>,
}

impl Tool for MovePointDistAngle {
    fn overlay(&self) -> &[PathPrimitive] {
        &[]
    }
    fn reset(&mut self) {}
    fn submit(&mut self, input: ToolInput, arena: &ObjectArena) -> ToolResponse {
        match input {
            ToolInput {
                mouse: Mouse::Press,
                hover: Some(Entry::PointDistAngle(id, o, ..)),
                ..
            } => {
                self.reset();
                if o.dist.is_const() {
                    _ = self.holding.insert(*id);
                }
                ToolResponse::default()
            }
            ToolInput {
                mouse: Mouse::Release,
                ..
            } => {
                let overlay_changed = self.holding.is_some();
                self.reset();
                ToolResponse {
                    overlay_changed,
                    ..Default::default()
                }
            }
            ToolInput {
                mouse: Mouse::Move,
                cursor: target,
                ..
            } if let Some(id) = self.holding => {
                let Some(Entry::PointDistAngle(.., o, v)) = arena.get_entry(id.into()) else {
                    return ToolResponse::default();
                };

                let angle_free = o.angle.is_const() && !o.lock_angle;
                let dist_free = o.dist.is_const() && !o.lock_dist;

                let new_dist = if dist_free {
                    Some(expression::length(v.parent.dist(target)))
                } else {
                    None
                };

                let new_angle = if angle_free {
                    let ang = if input.modifiers.shift {
                        snap_angle(v.parent.angle(target))
                    } else {
                        v.parent.angle(target)
                    };
                    Some(expression::angle(ang))
                } else {
                    None
                };

                ToolResponse {
                    action: Some(Action::Update(Update::PointDistAngle {
                        target: id,
                        dist: new_dist,
                        angle: new_angle,
                        parent: None,
                        lock_angle: None,
                        lock_dist: None,
                    })),
                    ..Default::default()
                }
            }
            _ => ToolResponse::default(),
        }
    }
    fn can_interact(&self, g: Option<&Entry>) -> bool {
        self.holding.is_some()
            || g.is_some_and(|e| {
                let Entry::PointDistAngle(_, o, _) = e else {
                    return false;
                };
                o.dist.is_const() || o.angle.is_const()
            })
    }
}

#[derive(Default)]
pub struct MovePointOnLine {
    holding: Option<PointOnLineId>,
}

impl Tool for MovePointOnLine {
    fn overlay(&self) -> &[PathPrimitive] {
        &[]
    }

    fn reset(&mut self) {
        self.holding = None;
    }

    fn submit(&mut self, input: ToolInput, arena: &ObjectArena) -> ToolResponse {
        match input {
            ToolInput {
                mouse: Mouse::Press,
                hover: Some(Entry::PointOnLine(id, o, ..)),
                ..
            } => {
                self.reset();
                if o.dist.is_const() {
                    self.holding = Some(*id);
                }
                ToolResponse::default()
            }
            ToolInput {
                mouse: Mouse::Release,
                ..
            } => {
                let overlay_changed = self.holding.is_some();
                self.reset();
                ToolResponse {
                    overlay_changed,
                    ..Default::default()
                }
            }
            ToolInput {
                mouse: Mouse::Move,
                cursor: target,
                ..
            } if let Some(id) = self.holding => {
                let Some(Entry::PointOnLine(.., v)) = arena.get_entry(id.into()) else {
                    return ToolResponse::default();
                };

                // TODO: decide whether to clamp a point on a line to the actual line segment
                // between the two points or not
                // clamping would be consistent with the behaviour of points on curves
                let closest = geom::closest_point_on_line_segment(v.from, v.to, target).0;
                let new_dist = expression::length(v.from.dist(closest));

                ToolResponse {
                    action: Some(Action::Update(Update::PointOnLine {
                        target: id,
                        dist: Some(new_dist),
                        from: None,
                        to: None,
                    })),
                    ..Default::default()
                }
            }
            _ => ToolResponse::default(),
        }
    }

    fn can_interact(&self, g: Option<&Entry>) -> bool {
        self.holding.is_some()
            || g.is_some_and(|e| {
                let Entry::PointOnLine(_, o, _) = e else {
                    return false;
                };
                o.dist.is_const()
            })
    }
}

#[derive(Default)]
pub struct MovePointOnCurve {
    holding: Option<PointOnCurveId>,
}

impl Tool for MovePointOnCurve {
    fn overlay(&self) -> &[PathPrimitive] {
        &[]
    }

    fn reset(&mut self) {
        self.holding = None;
    }

    fn submit(&mut self, input: ToolInput, arena: &ObjectArena) -> ToolResponse {
        match input {
            ToolInput {
                mouse: Mouse::Press,
                hover: Some(Entry::PointOnCurve(id, o, ..)),
                ..
            } => {
                self.reset();
                if o.dist.is_const() {
                    self.holding = Some(*id);
                }
                ToolResponse::default()
            }
            ToolInput {
                mouse: Mouse::Release,
                ..
            } => {
                let overlay_changed = self.holding.is_some();
                self.reset();
                ToolResponse {
                    overlay_changed,
                    ..Default::default()
                }
            }
            ToolInput {
                mouse: Mouse::Move,
                cursor: target,
                ..
            } if let Some(id) = self.holding => {
                let Some(Entry::PointOnCurve(.., v)) = arena.get_entry(id.into()) else {
                    return ToolResponse::default();
                };

                let t_closest = geom::closest_point_on_curve(v.curve, target).1;
                let new_dist = expression::length(v.curve.split_at(t_closest).0.approx_length());

                ToolResponse {
                    action: Some(Action::Update(Update::PointOnCurve {
                        target: id,
                        dist: Some(new_dist),
                        curve: None,
                    })),
                    ..Default::default()
                }
            }
            _ => ToolResponse::default(),
        }
    }

    fn can_interact(&self, g: Option<&Entry>) -> bool {
        self.holding.is_some()
            || g.is_some_and(|e| {
                let Entry::PointOnCurve(_, o, _) = e else {
                    return false;
                };
                o.dist.is_const()
            })
    }
}
#[derive(Default)]
pub struct MoveCurveControl {
    holding: Option<CurveControlId>,
}

impl Tool for MoveCurveControl {
    fn overlay(&self) -> &[PathPrimitive] {
        &[]
    }

    fn reset(&mut self) {
        self.holding = None;
    }

    fn submit(&mut self, input: ToolInput, arena: &ObjectArena) -> ToolResponse {
        match input {
            ToolInput {
                mouse: Mouse::Press,
                hover: Some(Entry::CurveControl(id, ..)),
                ..
            } => {
                self.reset();
                _ = self.holding.insert(*id);
                ToolResponse::default()
            }
            ToolInput {
                mouse: Mouse::Release,
                ..
            } => {
                let overlay_changed = self.holding.is_some();
                self.reset();
                ToolResponse {
                    overlay_changed,
                    ..Default::default()
                }
            }
            ToolInput {
                mouse: Mouse::Move,
                cursor,
                modifiers,
                ..
            } if let Some(id) = self.holding => {
                let Some(Entry::CurveControl(.., v)) = arena.get_entry(id.into()) else {
                    return ToolResponse::default();
                };

                let mut off: geom::Polar = (cursor - v.parent).into();
                if modifiers.shift {
                    off.angle = snap_angle(off.angle)
                }

                ToolResponse {
                    action: Some(Action::Update(Update::CurveControl { target: id, off })),
                    ..Default::default()
                }
            }
            _ => ToolResponse::default(),
        }
    }

    fn can_interact(&self, g: Option<&Entry>) -> bool {
        self.holding.is_some() || g.is_some_and(|e| matches!(e, Entry::CurveControl(..)))
    }
}

#[derive(Default, Clone, Copy)]
enum MoveToolKind {
    #[default]
    None,
    PointFree,
    PointDistAngle,
    PointOnLine,
    PointOnCurve,
    CurveControl,
}

#[derive(Default)]
pub struct Move {
    // slightly unhinged
    // the inner tools could actually be used directly, but i mainly did this to avoid having one
    // tool with hundreds of branches. (will get even more complicated because i want to add special
    // overlays based on degrees of freedom, e.g. if a point at dist and angle has a locked dist,
    // display the circle on which the point can move)
    //
    // in seamly, you can only move the (singular) root/free point with the cursor. this can even
    // move "dependent" points, wow! (only if the value being modified is constant, and not explicitly
    // locked. if it determined by some non-constant formula, then you probably would not even
    // want to drag it around freely)
    point_free: MovePointFree,
    point_dist_angle: MovePointDistAngle,
    point_on_line: MovePointOnLine,
    point_on_curve: MovePointOnCurve,
    curve_control: MoveCurveControl,
    active: MoveToolKind,
}

impl Tool for Move {
    fn reset(&mut self) {
        self.point_free.reset();
        self.point_on_line.reset();
        self.point_on_curve.reset();
        self.point_dist_angle.reset();
        self.curve_control.reset();
        self.active = MoveToolKind::None;
    }

    fn overlay(&self) -> &[PathPrimitive] {
        match self.active {
            MoveToolKind::None => &[],
            MoveToolKind::PointFree => self.point_free.overlay(),
            MoveToolKind::PointDistAngle => self.point_dist_angle.overlay(),
            MoveToolKind::PointOnLine => self.point_on_line.overlay(),
            MoveToolKind::PointOnCurve => self.point_on_curve.overlay(),
            MoveToolKind::CurveControl => self.curve_control.overlay(),
        }
    }

    fn submit(&mut self, input: ToolInput, arena: &ObjectArena) -> ToolResponse {
        match input {
            ToolInput {
                mouse: Mouse::Release,
                ..
            } => {
                self.reset();
                ToolResponse {
                    overlay_changed: true,
                    ..Default::default()
                }
            }
            ToolInput {
                mouse: Mouse::Press,
                hover: Some(Entry::PointDistAngle(..)),
                ..
            } => {
                self.active = MoveToolKind::PointDistAngle;
                self.point_dist_angle.submit(input, arena)
            }
            ToolInput {
                mouse: Mouse::Press,
                hover: Some(Entry::PointFree(..)),
                ..
            } => {
                self.active = MoveToolKind::PointFree;
                self.point_free.submit(input, arena)
            }
            ToolInput {
                mouse: Mouse::Press,
                hover: Some(Entry::PointOnLine(..)),
                ..
            } => {
                self.active = MoveToolKind::PointOnLine;
                self.point_on_line.submit(input, arena)
            }
            ToolInput {
                mouse: Mouse::Press,
                hover: Some(Entry::PointOnCurve(..)),
                ..
            } => {
                self.active = MoveToolKind::PointOnCurve;
                self.point_on_curve.submit(input, arena)
            }
            ToolInput {
                mouse: Mouse::Press,
                hover: Some(Entry::CurveControl(..)),
                ..
            } => {
                self.active = MoveToolKind::CurveControl;
                self.curve_control.submit(input, arena)
            }
            ToolInput {
                mouse: Mouse::Move, ..
            } => match self.active {
                MoveToolKind::None => ToolResponse::default(),
                MoveToolKind::PointFree => self.point_free.submit(input, arena),
                MoveToolKind::PointDistAngle => self.point_dist_angle.submit(input, arena),
                MoveToolKind::PointOnLine => self.point_on_line.submit(input, arena),
                MoveToolKind::PointOnCurve => self.point_on_curve.submit(input, arena),
                MoveToolKind::CurveControl => self.curve_control.submit(input, arena),
            },
            _ => ToolResponse::default(),
        }
    }

    fn can_interact(&self, g: Option<&Entry>) -> bool {
        if let Some(e) = g {
            match e {
                Entry::PointFree(..) => self.point_free.can_interact(g),
                Entry::PointDistAngle(..) => self.point_dist_angle.can_interact(g),
                Entry::PointOnLine(..) => self.point_on_line.can_interact(g),
                Entry::PointOnCurve(..) => self.point_on_curve.can_interact(g),
                Entry::CurveControl(..) => self.curve_control.can_interact(g),
                _ => false,
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Default)]
pub struct AddPointFree {
    overlay: Option<[PathPrimitive; 1]>,
}

impl Tool for AddPointFree {
    fn reset(&mut self) {
        self.overlay = None;
    }
    fn overlay(&self) -> &[PathPrimitive] {
        self.overlay.as_ref().map_or(&[], |f| f.as_slice())
    }
    fn submit(&mut self, input: ToolInput, _: &ObjectArena) -> ToolResponse {
        match input {
            ToolInput {
                cursor,
                mouse: Mouse::Press,
                ..
            } => {
                _ = self.overlay.insert([PathPrimitive::Point(input.cursor)]);
                ToolResponse {
                    action: Some(Action::Add(crate::construction::Add::PointFree(cursor))),
                    done: true,
                    ..Default::default()
                }
            }

            ToolInput {
                cursor,
                mouse: Mouse::Move,
                ..
            } => {
                _ = self.overlay.insert([PathPrimitive::Point(cursor)]);
                ToolResponse {
                    overlay_changed: true,
                    ..Default::default()
                }
            }
            _ => ToolResponse::default(),
        }
    }

    fn can_interact(&self, _: Option<&Entry>) -> bool {
        true
    }
}

#[derive(Default, Debug)]
pub struct AddPointOnCurve {
    curve: Option<(CurveId, CubicBezier)>,
    overlay: Option<[PathPrimitive; 2]>,
}

impl Tool for AddPointOnCurve {
    fn reset(&mut self) {
        self.curve = None;
        self.overlay = None;
    }
    fn overlay(&self) -> &[PathPrimitive] {
        self.overlay.as_ref().map_or(&[], |f| f.as_slice())
    }
    fn submit(&mut self, input: ToolInput, _: &ObjectArena) -> ToolResponse {
        match input {
            ToolInput {
                cursor,
                mouse: Mouse::Move,
                ..
            } if let Some((_, curve)) = self.curve => {
                let (q, _) = geom::closest_point_on_curve(curve, cursor);
                _ = self
                    .overlay
                    .insert([PathPrimitive::Point(q), PathPrimitive::Curve(curve)]);
                ToolResponse {
                    overlay_changed: true,
                    ..Default::default()
                }
            }
            ToolInput {
                cursor,
                mouse: Mouse::Press,
                ..
            } if let Some((id, curve)) = self.curve => {
                let (_, t) = geom::closest_point_on_curve(curve, cursor);
                let dist = curve.split_at(t).0.approx_length();
                ToolResponse {
                    done: true,
                    action: Some(Action::Add(Add::PointOnCurve(id, dist))),
                    ..Default::default()
                }
            }
            ToolInput {
                cursor: target,
                hover: Some(Entry::Curve(i, c, v)),
                mouse: Mouse::Press,
                ..
            } if self.curve.is_none() => {
                _ = self.curve.insert((*i, v.curve));
                let (q, _) = geom::closest_point_on_curve(v.curve, target);
                _ = self
                    .overlay
                    .insert([PathPrimitive::Point(q), PathPrimitive::Curve(v.curve)]);
                ToolResponse {
                    overlay_changed: true,
                    ..Default::default()
                }
            }
            _ => ToolResponse::default(),
        }
    }

    fn can_interact(&self, g: Option<&Entry>) -> bool {
        self.curve.is_some() || g.is_some_and(|e| matches!(e, Entry::Curve(..)))
    }
}

#[derive(Default, Debug)]
pub struct AddPointOnLine {
    from: Option<(PointId, Point2)>,
    to: Option<(PointId, Point2)>,
    overlay: Option<[PathPrimitive; 2]>,
}

impl Tool for AddPointOnLine {
    fn reset(&mut self) {
        self.from = None;
        self.to = None;
        self.overlay = None;
    }

    fn overlay(&self) -> &[PathPrimitive] {
        self.overlay.as_ref().map_or(&[], |f| f.as_slice())
    }

    fn submit(&mut self, input: ToolInput, _: &ObjectArena) -> ToolResponse {
        match input {
            ToolInput {
                hover: Some(g),
                mouse: Mouse::Press,
                ..
            } if self.from.is_none()
                && let Some((id, pos)) = g.as_point_pos() =>
            {
                // extra defensive, should not have anything in it a this point
                self.to = None;
                _ = self.from.insert((id, pos));
                ToolResponse::default()
            }

            ToolInput {
                hover: Some(g),
                mouse: Mouse::Press,
                ..
            } if self.from.is_some()
                && self.to.is_none()
                && let Some((id, pos)) = g.as_point_pos() =>
            {
                _ = self.to.insert((id, pos));
                ToolResponse::default()
            }

            ToolInput {
                cursor: target,
                mouse: Mouse::Press,
                ..
            } if let Some((from_id, from_pos)) = self.from
                && let Some((to_id, to_pos)) = self.to =>
            {
                let t = geom::closest_point_on_line_segment(from_pos, to_pos, target).1;
                let dist = t * (from_pos.dist(to_pos));
                ToolResponse {
                    done: true,
                    action: Some(Action::Add(Add::PointOnLine(from_id, to_id, dist))),
                    ..Default::default()
                }
            }
            ToolInput {
                cursor,
                hover,
                mouse: Mouse::Move,
                ..
            } if let Some((_, from)) = self.from
                && self.to.is_none() =>
            {
                // snapping to point position if one is hovered
                let to = hover
                    .and_then(Entry::as_point_pos)
                    .map(|x| x.1)
                    .unwrap_or(cursor);

                let midway = from + (to - from) * (0.5);

                _ = self
                    .overlay
                    .insert([PathPrimitive::Line(from, to), PathPrimitive::Point(midway)]);

                ToolResponse {
                    overlay_changed: true,
                    ..Default::default()
                }
            }

            ToolInput {
                cursor: target,
                mouse: Mouse::Move,
                ..
            } if let Some((_, from)) = self.from
                && let Some((_, to)) = self.to =>
            {
                let on_line = geom::closest_point_on_line_segment(from, to, target).0;

                _ = self
                    .overlay
                    .insert([PathPrimitive::Line(from, to), PathPrimitive::Point(on_line)]);

                ToolResponse {
                    overlay_changed: true,
                    ..Default::default()
                }
            }
            _ => ToolResponse::default(),
        }
    }
    fn can_interact(&self, g: Option<&Entry>) -> bool {
        match (self.from, self.to) {
            (Some(_), Some(_)) => true,
            _ => g.is_some_and(|e| e.as_point_pos().is_some()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AddPointDistAngle {
    parent: Option<(PointId, Point2)>,
    overlay: Option<[PathPrimitive; 2]>,
}

impl Tool for AddPointDistAngle {
    fn submit(&mut self, input: ToolInput, _: &ObjectArena) -> ToolResponse {
        match input {
            ToolInput {
                hover: Some(e),
                mouse: Mouse::Press,
                ..
            } if self.parent.is_none()
                && let Some((id, pos)) = e.as_point_pos() =>
            {
                self.parent = Some((id, pos));

                ToolResponse {
                    done: false,
                    action: None,
                    overlay_changed: true,
                }
            }
            ToolInput {
                mouse: Mouse::Press,
                cursor,
                modifiers,
                ..
            } if let Some((parent_id, parent_pos)) = self.parent => {
                let dist = parent_pos.dist(cursor);
                let angle = if modifiers.shift {
                    snap_angle(parent_pos.angle(cursor))
                } else {
                    parent_pos.angle(cursor)
                };

                //
                ToolResponse {
                    done: true,
                    action: Some(Action::Add(Add::PointDistAngle(
                        parent_id,
                        expression::length(dist),
                        expression::angle(angle),
                    ))),
                    overlay_changed: true,
                }
            }
            ToolInput {
                cursor,
                mouse: Mouse::Move,
                modifiers,
                ..
            } if let Some((_, parent_pos)) = self.parent => {
                let target = if modifiers.shift {
                    let d = parent_pos.dist(cursor);
                    let a = snap_angle(parent_pos.angle(cursor));
                    parent_pos + geom::polar(d, a)
                } else {
                    cursor
                };

                self.overlay = Some([
                    PathPrimitive::Line(parent_pos, target),
                    PathPrimitive::Point(target),
                ]);
                ToolResponse {
                    done: false,
                    action: None,
                    overlay_changed: true,
                }
            }
            _ => ToolResponse {
                done: false,
                action: None,
                overlay_changed: false,
            },
        }
    }

    fn overlay(&self) -> &[PathPrimitive] {
        match self.overlay.as_ref() {
            Some(o) => o.as_slice(),
            None => &[],
        }
    }

    fn reset(&mut self) {
        self.parent = None;
        self.overlay = None;
    }

    fn can_interact(&self, g: Option<&Entry>) -> bool {
        self.parent.is_some() || g.is_some_and(|e| e.as_point_pos().is_some())
    }
}

#[derive(Debug, Default, Clone)]
pub struct AddLine {
    first: Option<(PointId, Point2)>,
    overlay: Option<[PathPrimitive; 1]>,
}

impl Tool for AddLine {
    fn submit(&mut self, input: ToolInput, _: &ObjectArena) -> ToolResponse {
        match input {
            ToolInput {
                hover: Some(e),
                mouse: Mouse::Press,
                ..
            } if self.first.is_none()
                && let Some(p) = e.as_point_pos() =>
            {
                self.first = Some(p);
                ToolResponse {
                    done: false,
                    action: None,
                    overlay_changed: false,
                }
            }
            ToolInput {
                hover,
                mouse: Mouse::Move,
                cursor: target,
                ..
            } if let Some((_, first_pos)) = self.first => {
                let to = hover.and_then(Entry::as_point_pos).map_or(target, |e| e.1);

                _ = self.overlay.insert([PathPrimitive::Line(first_pos, to)]);

                ToolResponse {
                    done: false,
                    action: None,
                    overlay_changed: true,
                }
            }
            ToolInput {
                hover: Some(e),
                mouse: Mouse::Press,
                ..
            } if let Some((from_id, _)) = self.first
                && let Some((to_id, _)) = e.as_point_pos() =>
            {
                self.reset();
                ToolResponse {
                    done: true,
                    action: Some(Action::Add(Add::Line(from_id, to_id))),
                    overlay_changed: true,
                }
            }
            _ => ToolResponse::default(),
        }
    }
    fn reset(&mut self) {
        self.first = None;
        self.overlay = None;
    }
    fn overlay(&self) -> &[PathPrimitive] {
        match self.overlay.as_ref() {
            Some(o) => o.as_slice(),
            None => &[],
        }
    }
    fn can_interact(&self, g: Option<&Entry>) -> bool {
        g.is_some_and(|e| e.as_point_pos().is_some())
    }
}

#[derive(Debug, Default, Clone)]
pub struct AddCurve {
    first: Option<(PointId, Point2)>,
    overlay: Option<[PathPrimitive; 1]>,
}

impl Tool for AddCurve {
    fn submit(&mut self, input: ToolInput, _: &ObjectArena) -> ToolResponse {
        match input {
            ToolInput {
                hover: Some(e),
                mouse: Mouse::Press,
                ..
            } if self.first.is_none()
                && let Some(p) = e.as_point_pos() =>
            {
                self.first = Some(p);
                ToolResponse {
                    done: false,
                    action: None,
                    overlay_changed: false,
                }
            }
            ToolInput {
                hover,
                mouse: Mouse::Move,
                cursor,
                ..
            } if let Some((_, from)) = self.first => {
                let to = hover.and_then(Entry::as_point_pos).map_or(cursor, |e| e.1);

                // slightly curvy phantom curve to make it look visually distinct from the line tool
                let l = from.dist(to);
                let control_dist = l / 4.;
                let control_1_angle = from.angle(to) + f64::consts::FRAC_PI_4;
                let control_2_angle = to.angle(from) + f64::consts::FRAC_PI_4;
                let control_1 = from + geom::polar(control_dist, control_1_angle);
                let control_2 = to + geom::polar(control_dist, control_2_angle);
                let c = geom::curve(from, control_1, control_2, to);

                self.overlay = Some([PathPrimitive::Curve(c)]);

                ToolResponse {
                    done: false,
                    action: None,
                    overlay_changed: true,
                }
            }
            ToolInput {
                hover: Some(e),
                mouse: Mouse::Press,
                ..
            } if let Some((from_id, _)) = self.first
                && let Some((to_id, _)) = e.as_point_pos() =>
            {
                self.reset();
                ToolResponse {
                    done: true,
                    action: Some(Action::Add(Add::Curve(from_id, to_id))),
                    overlay_changed: true,
                }
            }
            _ => ToolResponse::default(),
        }
    }
    fn reset(&mut self) {
        self.first = None;
        self.overlay = None;
    }
    fn overlay(&self) -> &[PathPrimitive] {
        match self.overlay.as_ref() {
            Some(o) => o.as_slice(),
            None => &[],
        }
    }
    fn can_interact(&self, g: Option<&Entry>) -> bool {
        g.is_some_and(|e| e.as_point_pos().is_some())
    }
}
