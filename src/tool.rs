use crate::arena::Arena;
use crate::construction::{
    AngleExpression, LengthExpression, Object, PointId, PointObj, expression,
};
use crate::{construction::ObjectId, geom::Point2};

static_assertions::assert_obj_safe!(Tool);

#[derive(Default, Debug, Clone)]
pub struct ToolResponse {
    pub done: bool,
    pub action: Option<Action>,
    pub overlay_changed: bool,
}

// TODO: add modifier (ctrl/shift)
// (to for example implement snapping behaviour for some tools, like choice of angle)
#[derive(Debug, Clone, Copy)]
pub enum ToolInput {
    Press { obj: Option<ObjectId>, pos: Point2 },
    Release { obj: Option<ObjectId>, pos: Point2 },
    Move { obj: Option<ObjectId>, pos: Point2 },
}

// "vocabulary" used to describe a tool overlay, will be rendered in a different scene to avoid
// having to rebuild the main scene any time the tool overlay changes (which could be very often,
// e.g. with a virtual line from some selected point to the cursor position)
// renderer can decide how exactly to style these
#[derive(Debug, Clone, Copy)]
pub enum PathPrimitive {
    Line(Point2, Point2),
    Point(Point2),
    Curve(Point2, Point2, Point2, Point2),
    // would be nice to have stuff like
    // HighlightPoint(PointId)
    // to let the renderer know to highlight a particular point
}

#[derive(Debug, Clone)]
pub enum Action {
    AddPoint(PointDefinition),
    AddLine(PointId, PointId),
    AddCurve(PointId, PointId),
    DragTo(ObjectId, Point2),
    // TODO ...
}

#[derive(Debug, Clone)]
pub enum PointDefinition {
    // the dist/angle expressions would probably be constant initially (as produced by the tool)
    // then a more complex expression could be entered with a dialog?
    DistAngle {
        parent: PointId,
        dist: LengthExpression,
        angle: AngleExpression,
    },
    Free {
        pos: Point2,
    },
    OnLineRel {
        from: PointId,
        to: PointId,
        frac: f64,
    },
    // TODO: ...
}

pub trait Tool {
    // tool receives events, responds with state that may contain an action that should be applied
    // to the Arena<Object>
    fn submit(&mut self, input: ToolInput, arena: &Arena<Object>) -> ToolResponse;

    // the tool may produce a description of some overlay paths that should be renderer
    // (e.g. when adding a point at dist/angle of another point: a virtual line from the position of
    // the first point to the cursor position)
    // this should be polled after every call to submit
    // between calls of submit, the previous overlay can be reused?
    fn overlay(&self) -> &[PathPrimitive];

    fn reset(&mut self);

    // TODO: report if tool can interact with something? (for cursor variant)
    // fn applicable(&self, arena: &Arena<Object>, id: ObjectId) -> bool;
}

pub fn default_boxed<T: Tool + Default + 'static>() -> Box<dyn Tool> {
    Box::new(T::default())
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Drag {
    holding: Option<ObjectId>,
}

impl Tool for Drag {
    fn submit(&mut self, input: ToolInput, _: &Arena<Object>) -> ToolResponse {
        match input {
            ToolInput::Press { obj: Some(o), .. } if self.holding.is_none() => {
                match o {
                    ObjectId::Point(_) | ObjectId::CurveControl(_) => {
                        self.holding = Some(o);
                    }
                    _ => {}
                }
                ToolResponse {
                    done: false,
                    action: None,
                    overlay_changed: false,
                }
            }
            ToolInput::Move { pos: p, .. } if let Some(holding) = self.holding => ToolResponse {
                action: Some(Action::DragTo(holding, p)),
                done: false,
                overlay_changed: false,
            },

            ToolInput::Release { pos: p, .. } if let Some(holding) = self.holding => {
                self.holding = None;
                ToolResponse {
                    done: true,
                    action: Some(Action::DragTo(holding, p)),
                    overlay_changed: false,
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
        &[]
    }

    fn reset(&mut self) {
        self.holding = None
    }
}

#[derive(Debug, Clone, Default)]
pub struct AddPointDistAngle {
    parent: Option<PointId>,
    overlay: Option<[PathPrimitive; 2]>,
}

impl Tool for AddPointDistAngle {
    fn submit(&mut self, input: ToolInput, arena: &Arena<Object>) -> ToolResponse {
        match input {
            ToolInput::Press {
                obj: Some(ObjectId::Point(id)),
                pos,
            } if self.parent.is_none() => {
                self.parent = Some(id);
                let parent_pos = arena.get_value_for::<PointObj>(id).expect("TODO").pos;
                self.overlay = Some([
                    PathPrimitive::Line(pos, parent_pos),
                    PathPrimitive::Point(pos),
                ]);
                ToolResponse {
                    done: false,
                    action: None,
                    overlay_changed: true,
                }
            }
            ToolInput::Press { pos, .. } if let Some(parent) = self.parent => {
                let parent_pos = arena.get_value_for::<PointObj>(parent).expect("TODO").pos;
                let ang = parent_pos.angle(pos);
                let dist = parent_pos.dist(pos);
                ToolResponse {
                    done: true,
                    action: Some(Action::AddPoint(PointDefinition::DistAngle {
                        parent,
                        dist: expression::length(dist),
                        angle: expression::angle(ang),
                    })),
                    overlay_changed: false,
                }
            }
            ToolInput::Move { pos, .. } if let Some(parent) = self.parent => {
                let parent_pos = arena.get_value_for::<PointObj>(parent).expect("TODO").pos;
                self.overlay = Some([
                    PathPrimitive::Line(pos, parent_pos),
                    PathPrimitive::Point(pos),
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
}

#[derive(Debug, Default, Clone)]
pub struct Line {
    first: Option<PointId>,
    overlay: Option<[PathPrimitive; 1]>,
}

impl Tool for Line {
    fn submit(&mut self, input: ToolInput, arena: &Arena<Object>) -> ToolResponse {
        match input {
            ToolInput::Press { obj, .. }
                if self.first.is_none()
                    && let Some(ObjectId::Point(id)) = obj =>
            {
                self.first = Some(id);
                ToolResponse {
                    done: false,
                    action: None,
                    overlay_changed: false,
                }
            }
            ToolInput::Move { obj, pos } if let Some(first) = self.first => {
                let first_pos = arena.get_value_for::<PointObj>(first).expect("TODO").pos;
                if let Some(ObjectId::Point(id)) = obj {
                    let second_pos = arena.get_value_for::<PointObj>(id).expect("TODO").pos;
                    self.overlay = Some([PathPrimitive::Line(first_pos, second_pos)]);
                } else {
                    self.overlay = Some([PathPrimitive::Line(first_pos, pos)]);
                };
                ToolResponse {
                    done: false,
                    action: None,
                    overlay_changed: true,
                }
            }
            ToolInput::Press { obj: target, .. }
                if let Some(p) = self.first
                    && let Some(ObjectId::Point(q)) = target =>
            {
                self.reset();
                ToolResponse {
                    done: true,
                    action: Some(Action::AddLine(p, q)),
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
}

#[derive(Debug, Clone, Default)]
pub struct Free {
    overlay: Option<[PathPrimitive; 1]>,
}

impl Tool for Free {
    fn submit(&mut self, input: ToolInput, _: &Arena<Object>) -> ToolResponse {
        match input {
            ToolInput::Move { pos, .. } => {
                self.overlay = Some([PathPrimitive::Point(pos)]);
                ToolResponse {
                    done: false,
                    action: None,
                    overlay_changed: true,
                }
            }
            ToolInput::Press { pos, .. } => {
                self.overlay = None;
                ToolResponse {
                    done: true,
                    action: Some(Action::AddPoint(PointDefinition::Free { pos })),
                    overlay_changed: true,
                }
            }
            _ => ToolResponse::default(),
        }
    }
    fn reset(&mut self) {
        self.overlay = None
    }
    fn overlay(&self) -> &[PathPrimitive] {
        match self.overlay.as_ref() {
            Some(o) => o.as_slice(),
            None => &[],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OnLine {
    overlay: Option<[PathPrimitive; 2]>,
    first: Option<PointId>,
    second: Option<PointId>,
}

impl Tool for OnLine {
    fn submit(&mut self, input: ToolInput, arena: &Arena<Object>) -> ToolResponse {
        match input {
            ToolInput::Press {
                obj: Some(ObjectId::Point(l)),
                ..
            } if self.first.is_none() => {
                self.first = Some(l);
                ToolResponse::default()
            }
            ToolInput::Press {
                obj: Some(ObjectId::Point(l)),
                ..
            } if self.first.is_some() && self.second.is_none() => {
                self.second = Some(l);
                ToolResponse::default()
            }

            ToolInput::Move { pos, obj: target }
                if let Some(first) = self.first
                    && self.second.is_none() =>
            {
                let first = arena.get_value_for::<PointObj>(first).unwrap().pos;

                let target = if let Some(ObjectId::Point(p)) = target {
                    arena.get_value_for::<PointObj>(p).unwrap().pos
                } else {
                    pos
                };

                let midway = first + ((target - first) * 0.5);

                self.overlay = Some([
                    PathPrimitive::Line(first, target),
                    PathPrimitive::Point(midway),
                ]);

                ToolResponse {
                    done: false,
                    action: None,
                    overlay_changed: true,
                }
            }

            ToolInput::Move { pos, .. }
                if let Some(first) = self.first
                    && let Some(second) = self.second =>
            {
                let first = arena.get_value_for::<PointObj>(first).unwrap().pos;
                let second = arena.get_value_for::<PointObj>(second).unwrap().pos;
                let (p, _) = closest_point_on_line(first, second, pos);

                self.overlay = Some([PathPrimitive::Line(first, second), PathPrimitive::Point(p)]);

                ToolResponse {
                    done: false,
                    action: None,
                    overlay_changed: true,
                }
            }

            ToolInput::Press { pos, .. }
                if let Some(first) = self.first
                    && let Some(second) = self.second =>
            {
                let first_pos = arena.get_value_for::<PointObj>(first).unwrap().pos;
                let second_pos = arena.get_value_for::<PointObj>(second).unwrap().pos;
                let (_, t) = closest_point_on_line(first_pos, second_pos, pos);

                ToolResponse {
                    done: true,
                    action: Some(Action::AddPoint(PointDefinition::OnLineRel {
                        from: first,
                        to: second,
                        frac: t,
                    })),
                    overlay_changed: true,
                }
            }

            _ => ToolResponse::default(),
        }
    }

    fn overlay(&self) -> &[PathPrimitive] {
        match self.overlay.as_ref() {
            Some(o) => o,
            None => &[],
        }
    }

    fn reset(&mut self) {
        self.overlay = None;
        self.first = None;
        self.second = None;
    }
}

fn closest_point_on_line(a: Point2, b: Point2, q: Point2) -> (Point2, f64) {
    let ab = b - a;
    let aq = q - a;
    let t = (aq.dot(ab) / ab.dot(ab)).clamp(0.0, 1.0);
    (a + t * ab, t)
}
