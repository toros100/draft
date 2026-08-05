use crate::construction::eval::{Eval, EvalCtx, EvalError};
use crate::construction::expression::{ExpressionObj, ExpressionVal};
use crate::construction::value::*;
use crate::geom::{Point2, Polar};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct ObjectId(usize);

// impl From<ObjectId> for i32 {
//     fn from(value: ObjectId) -> Self {
//         debug_assert!(value.0 <= i32::MAX as usize);
//         value.0 as i32
//     }
// }
//
// impl From<usize> for ObjectId {
//     fn from(value: usize) -> Self {
//         Self(value)
//     }
// }

impl ObjectId {
    pub fn from_raw(val: usize) -> Self {
        ObjectId(val)
    }
    pub fn into_raw(self) -> usize {
        self.0
    }
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
    CurveControl(CurveControlObj),

    Expression(ExpressionObj),
}

#[derive(Clone, Copy, Debug)]
pub enum PointObj {
    // point with absolute position
    Absolute {
        pos: Point2,
    },
    // point at distance and angle from another point
    DistAngle {
        parent: ObjectId, // must refer to Object::Point in arena
        dist: ObjectId,   // must refer to Object::Expression
        angle: ObjectId,  // same
    },
    // point on line between two points
    // deliberately not referring to Object::Line, which are "drawn" lines
    OnLine {
        from: ObjectId, // must refer to Object::Point
        to: ObjectId,   // ...
        dist: ObjectId,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct LineObj {
    pub from: ObjectId,
    pub to: ObjectId,
}

#[derive(Clone, Copy, Debug)]
pub struct CurveObj {
    pub from: ObjectId,
    pub to: ObjectId,
    pub control_1: ObjectId,
    pub control_2: ObjectId,
}

#[derive(Clone, Copy, Debug)]
pub struct CurveControlObj {
    pub parent: ObjectId,
    pub off: Polar,
}

impl Eval for PointObj {
    type Output = PointVal;
    fn eval(&self, ctx: &impl EvalCtx) -> Result<Self::Output, EvalError> {
        match self {
            PointObj::Absolute { pos: p } => Ok(PointVal { pos: *p }),
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
            from: from.pos,
            to: to.pos,
            control_1: control_1.pos,
            control_2: control_2.pos,
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
