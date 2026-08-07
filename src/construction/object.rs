use crate::construction::eval::{Eval, EvalCtx, EvalError};
use crate::construction::expression::{ExpressionObj, ExpressionVal};
use crate::construction::value::*;
use crate::geom::{self, Point2, Polar};

mod id;
pub use id::*;

pub trait ArenaObject: Into<Object> + Eval<Output = Self::Val> {
    type Id: Id;
    type Val: TryProject<Value>;
}

impl ArenaObject for PointObj {
    type Id = PointId;
    type Val = PointVal;
}
impl ArenaObject for LineObj {
    type Id = LineId;
    type Val = LineVal;
}
impl ArenaObject for CurveControlObj {
    type Id = CurveControlId;
    type Val = CurveControlVal;
}
impl ArenaObject for CurveObj {
    type Id = CurveId;
    type Val = CurveVal;
}
impl ArenaObject for ExpressionObj {
    type Id = ExpressionId;
    type Val = ExpressionVal;
}

#[derive(Clone, Debug)]
pub enum Object {
    // a point
    Point(PointObj),

    // line between two points
    Line(LineObj),

    // cubic bezier between two points
    Curve(CurveObj),

    // bezier control point
    // functionally similar to Point::DistAngle, but i did not want make it a "first-class point",
    // because i do not want stuff like curves or other points to connect to these synthetic control
    // points.
    // this is kind of a "leaf" point, opposite to PointObj::Root:
    // may depend on other points, but no other point may depend on it?
    // might still want to allow its measurements (dist/angle) to be used in expressions?
    CurveControl(CurveControlObj),

    Expression(ExpressionObj),
}

impl PointObj {
    fn push_dependencies(&self, dep: &mut Vec<ObjectId>) {
        match self {
            PointObj::OnLine { from, to, dist } => {
                dep.extend::<[ObjectId; _]>([(*from).into(), (*to).into(), (*dist).into()])
            }
            PointObj::DistAngle {
                parent,
                dist,
                angle,
            } => dep.extend::<[ObjectId; _]>([parent.into(), dist.into(), angle.into()]),
            PointObj::OnCurve { curve, dist } => {
                dep.extend::<[ObjectId; _]>([curve.into(), dist.into()])
            }
            // writing this case out explicitly rather than using a wildcard so it will break when i
            // add variants to this enum rather than produce garbage
            PointObj::Root { .. } => {}
        }
    }
}

impl Object {
    pub fn push_dependencies(&self, dep: &mut Vec<ObjectId>) {
        // this should probably be done with a method on the ArenaObject trait
        // but i need to rethink the entire dependency tracking thing if/when i actually do smarter
        // caching than full refresh on every change in the arena
        match self {
            Object::Line(l) => dep.extend::<[ObjectId; _]>([l.to.into(), l.from.into()]),
            Object::Point(p) => p.push_dependencies(dep),
            Object::Curve(c) => dep.extend::<[ObjectId; _]>([
                c.from.into(),
                c.to.into(),
                c.control_1.into(),
                c.control_2.into(),
            ]),
            Object::CurveControl(c) => dep.push(c.parent.into()),
            Object::Expression(e) => e.push_dependencies(dep),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PointObj {
    // point with absolute position
    Root {
        pos: Point2,
    },

    // point at distance and angle from another point
    DistAngle {
        parent: PointId,     // must refer to Object::Point in arena
        dist: ExpressionId,  // must refer to Object::Expression
        angle: ExpressionId, // same
    },

    // point on line between two points
    // deliberately not referring to Object::Line, which are "drawn" lines
    OnLine {
        from: PointId, // must refer to Object::Point
        to: PointId,   // ...
        dist: ExpressionId,
    },

    // point on a curve
    OnCurve {
        curve: CurveId,
        dist: ExpressionId,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct LineObj {
    pub from: PointId,
    pub to: PointId,
}

#[derive(Clone, Copy, Debug)]
pub struct CurveObj {
    pub from: PointId,
    pub to: PointId,
    pub control_1: CurveControlId,
    pub control_2: CurveControlId,
}

#[derive(Clone, Copy, Debug)]
pub struct CurveControlObj {
    pub parent: PointId,
    pub off: Polar,
}

impl Eval for PointObj {
    type Output = PointVal;
    fn eval(&self, ctx: &impl EvalCtx) -> Result<Self::Output, EvalError> {
        match self {
            PointObj::Root { pos: p } => Ok(PointVal { pos: *p }),
            PointObj::DistAngle {
                parent,
                dist,
                angle,
            } => {
                let d = ctx.try_get_as::<&ExpressionVal>(dist)?.try_as_length()?;
                let a = ctx.try_get_as::<&ExpressionVal>(angle)?.try_as_angle()?;
                let off = Polar::new(d, a);
                let p = ctx.try_get_as::<&PointVal>(parent)?;

                Ok(PointVal { pos: p.pos + off })
            }
            PointObj::OnLine { from, to, dist } => {
                let from_pos = ctx.try_get_as::<&PointVal>(from)?;
                let to_pos = ctx.try_get_as::<&PointVal>(to)?;
                let dist = ctx.try_get_as::<&ExpressionVal>(dist)?.try_as_length()?;

                let v = from_pos
                    .pos
                    .vec_to(to_pos.pos)
                    .try_normalize()
                    .map(|v| v.scale(dist))
                    .unwrap_or_default();

                // WARN:
                // if the two points are closer than geom::EPS together, v will be the zero vec
                // and the point "on the line" will end up at "from"

                Ok(PointVal {
                    pos: from_pos.pos + v,
                })
            }
            PointObj::OnCurve { curve, dist } => {
                let curve = ctx.try_get_as::<&CurveVal>(curve)?;
                let dist = ctx.try_get_as::<&ExpressionVal>(dist)?.try_as_length()?;
                Ok(PointVal {
                    pos: curve.curve.point_on(dist),
                })
            }
        }
    }
}

impl Eval for LineObj {
    type Output = LineVal;
    fn eval(&self, ctx: &impl EvalCtx) -> Result<Self::Output, EvalError> {
        let from = ctx.try_get_as::<&PointVal>(self.from)?;
        let to = ctx.try_get_as::<&PointVal>(self.to)?;

        Ok(LineVal {
            from: from.pos,
            to: to.pos,
        })
    }
}

impl Eval for CurveObj {
    type Output = CurveVal;
    fn eval(&self, ctx: &impl EvalCtx) -> Result<Self::Output, EvalError> {
        let from = ctx.try_get_as::<&PointVal>(self.from)?;
        let to = ctx.try_get_as::<&PointVal>(self.to)?;
        let control_1 = ctx.try_get_as::<&CurveControlVal>(self.control_1)?;
        let control_2 = ctx.try_get_as::<&CurveControlVal>(self.control_2)?;

        Ok(CurveVal {
            curve: geom::curve(from.pos, control_1.pos, control_2.pos, to.pos),
        })
    }
}

impl Eval for CurveControlObj {
    type Output = CurveControlVal;
    fn eval(&self, ctx: &impl EvalCtx) -> Result<Self::Output, EvalError> {
        let parent = ctx.try_get_as::<&PointVal>(self.parent)?;

        Ok(CurveControlVal {
            pos: parent.pos + self.off,
            parent: parent.pos,
        })
    }
}

impl From<PointObj> for Object {
    fn from(value: PointObj) -> Self {
        Object::Point(value)
    }
}
impl From<LineObj> for Object {
    fn from(value: LineObj) -> Self {
        Object::Line(value)
    }
}
impl From<CurveObj> for Object {
    fn from(value: CurveObj) -> Self {
        Object::Curve(value)
    }
}
impl From<CurveControlObj> for Object {
    fn from(value: CurveControlObj) -> Self {
        Object::CurveControl(value)
    }
}
impl From<ExpressionObj> for Object {
    fn from(value: ExpressionObj) -> Self {
        Object::Expression(value)
    }
}
