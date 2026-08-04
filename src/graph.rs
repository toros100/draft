// should maybe rename this, the only graph is the implicit dependency graph between objects in the
// arena

use crate::geom::*;
use std::f64::consts::PI;
use std::ops::Mul;
use std::{collections::HashMap, ops::Add};
use thiserror::Error;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct ObjectId(usize);

impl From<ObjectId> for i32 {
    fn from(value: ObjectId) -> Self {
        debug_assert!(value.0 <= i32::MAX as usize);
        value.0 as i32
    }
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
    from: ObjectId,
    to: ObjectId,
}

#[derive(Clone, Copy, Debug)]
pub struct CurveObj {
    from: ObjectId,
    to: ObjectId,
    control_1: ObjectId,
    control_2: ObjectId,
}

#[derive(Clone, Copy, Debug)]
pub struct CurveControlObj {
    parent: ObjectId,
    off: Polar,
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

// want to also have expression for curve length
// but i also want points on curves to produce sub-curves like in seamly
// and then i want to be able to also measure these, and not just the "first-class" curve
//
// i.e.: curve from a to b
// point c placed on curve at 100mm from start (or by intersecting curve and line?)
// -> implicit curve from a to c
//
// will also need this making pattern pieces by contour/path
// points dont accidentally end up on curves i guess, they are all special
// point variants that could know about their curve
// so a path could be "collapsed" with that?
// path (a, curve_a_b, c)
// c is ON curve_a_b, contour detects this?

#[derive(Clone, Debug)]
pub enum ExpressionObj {
    Length(f64),
    Angle(f64),
    Scalar(f64),
    LineAngle(ObjectId, ObjectId),
    Dist(ObjectId, ObjectId),
    // could use ObjectId and put subexpressions into the arena instead of Box<ExpressionObj> for recursion
    // i guess that would be better for peak performance, could deduplicate expressions too
    Mul(Box<ExpressionObj>, Box<ExpressionObj>),
    Add(Box<ExpressionObj>, Box<ExpressionObj>),
    // TODO:
    // division (at least for scalars)
    // curve length including subcurves (oof)
    // further functions? trig? min/max? exponential?
    // small DLS + parser for input
    // what does seamly have? i think it even has conditionals?
}

impl ExpressionObj {
    pub fn type_check(&self) -> Result<(), ExpressionError> {
        todo!()
    }
}

#[derive(Clone, Copy)]
pub enum ExpressionVal {
    Length(f64),
    Angle(f64),
    Scalar(f64),
}

impl ExpressionVal {
    fn try_add(&self, other: &ExpressionVal) -> Result<ExpressionVal, ExpressionError> {
        match (self, other) {
            (ExpressionVal::Length(a), ExpressionVal::Length(b)) => {
                Ok(ExpressionVal::Length(a.add(b)))
            }
            (ExpressionVal::Angle(a), ExpressionVal::Angle(b)) => {
                Ok(ExpressionVal::Angle(a.add(b)))
            }
            (ExpressionVal::Scalar(a), ExpressionVal::Scalar(b)) => {
                Ok(ExpressionVal::Scalar(a.add(b)))
            }
            (ExpressionVal::Angle(a), ExpressionVal::Scalar(b))
            | (ExpressionVal::Scalar(a), ExpressionVal::Angle(b)) => {
                Ok(ExpressionVal::Angle(a.add(b)))
            }
            _ => Err(ExpressionError::UnexpectedOperandType),
        }
    }
    fn try_mul(&self, other: &ExpressionVal) -> Result<ExpressionVal, ExpressionError> {
        match (self, other) {
            (ExpressionVal::Length(a), ExpressionVal::Length(b)) => {
                Ok(ExpressionVal::Length(a.mul(b)))
            }
            (ExpressionVal::Angle(a), ExpressionVal::Angle(b)) => {
                Ok(ExpressionVal::Angle(a.mul(b)))
            }
            (ExpressionVal::Scalar(a), ExpressionVal::Scalar(b)) => {
                Ok(ExpressionVal::Scalar(a.mul(b)))
            }
            (ExpressionVal::Length(a), ExpressionVal::Scalar(b))
            | (ExpressionVal::Scalar(a), ExpressionVal::Length(b)) => {
                Ok(ExpressionVal::Length(a.mul(b)))
            }
            _ => Err(ExpressionError::UnexpectedOperandType),
        }
    }
}

type PointVal = Point2;
#[derive(Clone)]
pub enum Value {
    Point(PointVal),
    Line(LineVal),
    Curve(CurveVal),
    CurveControl(CurveControlVal),
    Expression(ExpressionVal),
}

#[derive(Clone, Copy)]
pub struct LineVal {
    pub from: Point2,
    pub to: Point2,
}

#[derive(Clone, Copy)]
pub struct CurveVal {
    pub from: Point2,
    pub to: Point2,
    pub control_1: Point2,
    pub control_2: Point2,
}

#[derive(Clone, Copy)]
pub struct CurveControlVal {
    pub pos: Point2,
    pub parent: Point2,
}

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("unresolved dependency (order broken)")]
    UnresolvedDependency,
    #[error("unknown dependency")]
    UnknownDependency,
    #[error("unexpected value type")]
    UnexpectedType,
    #[error("expression error: {}", .0)]
    ExpressionError(ExpressionError),
}

impl From<ExpressionError> for EvalError {
    fn from(value: ExpressionError) -> Self {
        EvalError::ExpressionError(value)
    }
}

#[derive(Error, Debug)]
pub enum ExpressionError {
    #[error("unexpected operand type")]
    UnexpectedOperandType,
    #[error("unexpected result type")]
    UnexpectedResultType,
}

impl Value {
    // TODO: janky
    fn dist(&self, to: Point2) -> f64 {
        match self {
            Value::Point(p) => p.dist(to),
            Value::CurveControl(c) => c.pos.dist(to),
            // returning MAX for unimplemented cases for now
            _ => f64::MAX,
        }
    }
}

trait EvalCtx {
    // returning reference here because i anticipate future Value variants that are not Copy
    // (e.g. complex curve with n points)
    fn get_cached(&self, id: ObjectId) -> Result<&Value, EvalError>;
}

// idea: to evaluate obj at index i, split the cache at that index and give the first part as
// context (objects may depend on anything that appears before it in the arena, implicit topological
// order of dependency graph)
struct PrefixCtx<'a> {
    id_to_idx: &'a HashMap<ObjectId, usize>,
    cached: &'a [Option<Value>],
}

impl<'a> EvalCtx for PrefixCtx<'a> {
    // maybe a specialized error type would be nice
    // but that is approaching overcooked, this should never even error
    // if everything else is correct
    fn get_cached(&self, id: ObjectId) -> Result<&Value, EvalError> {
        let idx = self
            .id_to_idx
            .get(&id)
            .copied()
            .ok_or(EvalError::UnknownDependency)?;
        if idx >= self.cached.len() {
            // should not happen, would indicate issue with the ordering of dependencies
            Err(EvalError::UnresolvedDependency)
        } else {
            self.cached[idx]
                .as_ref()
                .ok_or(EvalError::UnresolvedDependency)
        }
    }
}

trait Eval {
    type Output;
    fn eval(&self, ctx: &impl EvalCtx) -> Result<Self::Output, EvalError>;
}

impl Eval for ExpressionObj {
    type Output = ExpressionVal;
    fn eval(&self, ctx: &impl EvalCtx) -> Result<Self::Output, EvalError> {
        match self {
            ExpressionObj::Length(f) => Ok(ExpressionVal::Length(*f)),
            ExpressionObj::Angle(f) => Ok(ExpressionVal::Angle(*f)),
            ExpressionObj::Scalar(f) => Ok(ExpressionVal::Scalar(*f)),
            ExpressionObj::Add(a, b) => {
                let a = a.eval(ctx)?;
                let b = b.eval(ctx)?;
                Ok(a.try_add(&b)?)
            }
            ExpressionObj::Mul(a, b) => {
                let a = a.eval(ctx)?;
                let b = b.eval(ctx)?;
                Ok(a.try_mul(&b)?)
            }
            ExpressionObj::Dist(a, b) => {
                let a = ctx.get_cached(*a)?;
                let b = ctx.get_cached(*b)?;

                let a_pos = if let Value::Point(p) = a {
                    *p
                } else {
                    return Err(EvalError::UnexpectedType);
                };

                let b_pos = if let Value::Point(p) = b {
                    *p
                } else {
                    return Err(EvalError::UnexpectedType);
                };

                Ok(ExpressionVal::Length(a_pos.dist(b_pos)))
            }
            ExpressionObj::LineAngle(a, b) => {
                let a = ctx.get_cached(*a)?;
                let b = ctx.get_cached(*b)?;

                let a_pos = if let Value::Point(p) = a {
                    *p
                } else {
                    return Err(EvalError::UnexpectedType);
                };

                let b_pos = if let Value::Point(p) = b {
                    *p
                } else {
                    return Err(EvalError::UnexpectedType);
                };

                Ok(ExpressionVal::Angle(b_pos.angle(a_pos)))
            }
        }
    }
}

impl Eval for PointObj {
    type Output = PointVal;
    fn eval(&self, ctx: &impl EvalCtx) -> Result<Self::Output, EvalError> {
        match self {
            PointObj::Absolute { pos } => Ok(*pos),
            PointObj::DistAngle {
                parent,
                dist,
                angle,
            } => {
                // this "type checking" branching is very annoying and verbose
                // TODO: implement TryFrom?
                let dist =
                    if let Value::Expression(ExpressionVal::Length(l)) = ctx.get_cached(*dist)? {
                        *l
                    } else {
                        return Err(EvalError::UnexpectedType);
                    };
                let angle =
                    if let Value::Expression(ExpressionVal::Angle(l)) = ctx.get_cached(*angle)? {
                        *l
                    } else {
                        return Err(EvalError::UnexpectedType);
                    };

                let off = Polar { dist, angle };

                let parent_val = ctx.get_cached(*parent)?;
                match parent_val {
                    Value::Point(p) => Ok(*p + off),
                    _ => Err(EvalError::UnexpectedType),
                }
            }
            PointObj::OnLine { from, to, dist } => {
                let from_pos = if let Value::Point(p) = ctx.get_cached(*from)? {
                    *p
                } else {
                    return Err(EvalError::UnexpectedType);
                };
                let to_pos = if let Value::Point(p) = ctx.get_cached(*to)? {
                    *p
                } else {
                    return Err(EvalError::UnexpectedType);
                };

                let dist =
                    if let Value::Expression(ExpressionVal::Length(l)) = ctx.get_cached(*dist)? {
                        *l
                    } else {
                        return Err(EvalError::UnexpectedType);
                    };

                let v = from_pos
                    .vec_to(to_pos)
                    .try_normalize()
                    .map(|v| v.scale(dist))
                    .unwrap_or_default();

                // WARN:
                // if the two points are closer than geom::EPS together, v will be the zero vec
                // and the point "on the line" will end up at "from"

                Ok(from_pos + v)
            }
        }
    }
}

impl Eval for LineObj {
    type Output = LineVal;
    fn eval(&self, ctx: &impl EvalCtx) -> Result<Self::Output, EvalError> {
        let from = if let Value::Point(p) = ctx.get_cached(self.from)? {
            *p
        } else {
            return Err(EvalError::UnexpectedType);
        };
        let to = if let Value::Point(p) = ctx.get_cached(self.to)? {
            *p
        } else {
            return Err(EvalError::UnexpectedType);
        };

        Ok(LineVal { from, to })
    }
}

impl Eval for CurveObj {
    type Output = CurveVal;
    fn eval(&self, ctx: &impl EvalCtx) -> Result<Self::Output, EvalError> {
        let from = if let Value::Point(p) = ctx.get_cached(self.from)? {
            *p
        } else {
            return Err(EvalError::UnexpectedType);
        };

        let to = if let Value::Point(p) = ctx.get_cached(self.to)? {
            *p
        } else {
            return Err(EvalError::UnexpectedType);
        };
        let control_1 = if let Value::CurveControl(p) = ctx.get_cached(self.control_1)? {
            p.pos
        } else {
            return Err(EvalError::UnexpectedType);
        };
        let control_2 = if let Value::CurveControl(p) = ctx.get_cached(self.control_2)? {
            p.pos
        } else {
            return Err(EvalError::UnexpectedType);
        };

        Ok(CurveVal {
            from,
            to,
            control_1,
            control_2,
        })
    }
}

impl Eval for CurveControlObj {
    type Output = CurveControlVal;
    fn eval(&self, ctx: &impl EvalCtx) -> Result<Self::Output, EvalError> {
        let parent = ctx.get_cached(self.parent)?;

        let (pos, parent) = if let Value::Point(p) = parent {
            (*p + self.off, *p)
        } else {
            return Err(EvalError::UnexpectedType);
        };

        Ok(CurveControlVal { pos, parent })
    }
}

#[derive(Default)]
pub struct ObjectArena {
    pub v: Vec<Object>,
    pub ids: Vec<ObjectId>,
    pub cache: Vec<Option<Value>>,
    pub id_to_idx: HashMap<ObjectId, usize>,
}

impl ObjectArena {
    pub fn hit_scan(&self, cursor_pos: Point2, tolerance: f64) -> Option<ObjectId> {
        let mut dist = f64::MAX;
        let mut id = None;
        for (ev, i) in self.cache.iter().zip(self.ids.iter()) {
            if let Some(e) = ev {
                let d = e.dist(cursor_pos);
                if d <= tolerance && d < dist {
                    dist = d;
                    id = Some(*i)
                }
            }
        }
        id
    }

    pub fn drag_to(&mut self, id: ObjectId, target: Point2) {
        if let Some(&idx) = self.id_to_idx.get(&id) {
            let obj = &mut (self.v[idx]);
            if let Object::Point(PointObj::Absolute { pos }) = obj {
                *pos = target
            }
            match obj {
                Object::Point(PointObj::Absolute { pos }) => *pos = target,
                Object::CurveControl(CurveControlObj { parent, off }) => {
                    let parent_idx = self.id_to_idx.get(parent).copied().unwrap();
                    let parent_pos =
                        if let Value::Point(p) = self.cache[parent_idx].clone().unwrap() {
                            p
                        } else {
                            panic!("type error")
                        };
                    *off = (target - parent_pos).into()
                }
                _ => {}
            }
        }
    }

    pub fn add_point(&mut self, pos: Point2) -> ObjectId {
        let idx = self.v.len();
        let id = ObjectId(idx);
        let p = Object::Point(PointObj::Absolute { pos });
        self.v.push(p);
        self.ids.push(id);
        assert!(self.id_to_idx.insert(id, idx).is_none());
        self.cache.push(None);
        id
    }

    pub fn add_relative_point(
        &mut self,
        parent: ObjectId,
        dist: ExpressionObj,
        angle: ExpressionObj,
    ) -> ObjectId {
        let dist = self.push_obj(Object::Expression(dist));
        let angle = self.push_obj(Object::Expression(angle));

        let idx = self.v.len();
        let id = ObjectId(idx);

        let p = Object::Point(PointObj::DistAngle {
            parent,
            dist,
            angle,
        });
        self.v.push(p);
        self.ids.push(id);
        self.id_to_idx.insert(id, idx);
        self.cache.push(None);
        id
    }

    pub fn add_curve(&mut self, from: ObjectId, to: ObjectId) -> ObjectId {
        let control_1 = self.add_curve_control(from);
        let control_2 = self.add_curve_control(to);
        let o = Object::Curve(CurveObj {
            from,
            to,
            control_1,
            control_2,
        });
        self.push_obj(o)
    }

    pub fn add_point_midway(&mut self, from: ObjectId, to: ObjectId) -> ObjectId {
        let exp = ExpressionObj::Mul(
            ExpressionObj::Scalar(0.5).into(),
            ExpressionObj::Dist(from, to).into(),
        );
        self.add_point_on_line(from, to, exp)
    }

    pub fn add_point_perpendicular(
        &mut self,
        from: ObjectId,
        to: ObjectId,
        dist: ExpressionObj,
    ) -> ObjectId {
        let angle = ExpressionObj::Add(
            ExpressionObj::Angle(PI / 2.).into(),
            ExpressionObj::LineAngle(to, from).into(),
        );
        self.add_relative_point(from, dist, angle)
    }

    fn add_curve_control(&mut self, parent: ObjectId) -> ObjectId {
        let p = Object::CurveControl(CurveControlObj {
            parent,
            // TODO: issue with control point not being "selectable" with the hit scan
            // due to being at the exact same position as its parent, but earlier in the order?
            // quick fix: dont initialize it exactly on top of the point
            // this is not good (without touching controls, the curve should be a straight line)
            // could make the curve itself draggable to free the control from being stuck on its
            // parent, have to look into what this even means mathematically
            off: Polar {
                dist: 10.,
                angle: 0.,
            },
        });
        self.push_obj(p)
    }

    pub fn add_line(&mut self, from: ObjectId, to: ObjectId) -> ObjectId {
        let p = Object::Line(LineObj { from, to });
        self.push_obj(p)
    }

    fn push_obj(&mut self, o: Object) -> ObjectId {
        // WARN: will break when adding removal of objects (reuse of idx/id)
        // also, this should check if the object is "legal" to add:
        // everything it depends on needs to already be pushed
        let idx = self.v.len();
        let id = ObjectId(idx);
        self.v.push(o);
        self.ids.push(id);
        assert!(self.id_to_idx.insert(id, idx).is_none());
        self.cache.push(None);
        id
    }

    pub fn add_point_on_line(
        &mut self,
        from: ObjectId,
        to: ObjectId,
        dist: ExpressionObj,
    ) -> ObjectId {
        let dist = self.push_obj(Object::Expression(dist));

        let idx = self.v.len();
        let id = ObjectId(idx);
        let p = Object::Point(PointObj::OnLine { from, to, dist });
        self.v.push(p);
        self.ids.push(id);
        self.id_to_idx.insert(id, idx);
        self.cache.push(None);
        id
    }

    // NOTE: could do some kind of dirty tracking and only partial recalculation when something
    // updates. but this seems plenty fast enough for any realistic number of points, and partial
    // recalculation would also require tracking what changed and notifying the slint model of which
    // rows changed.
    pub fn calculate_all(&mut self) {
        for i in 0..self.v.len() {
            // want this to explode rn
            self.calculate(i).unwrap();
        }
    }

    fn calculate(&mut self, idx: usize) -> Result<(), EvalError> {
        let (prev, _) = self.cache.split_at(idx);

        let eval_ctx = PrefixCtx {
            id_to_idx: &self.id_to_idx,
            cached: prev,
        };

        let obj = &self.v[idx];

        let val = match obj {
            Object::Point(p) => Value::Point(p.eval(&eval_ctx)?),
            Object::Line(l) => Value::Line(l.eval(&eval_ctx)?),
            Object::CurveControl(c) => Value::CurveControl(c.eval(&eval_ctx)?),
            Object::Curve(c) => Value::Curve(c.eval(&eval_ctx)?),
            Object::Expression(e) => Value::Expression(e.eval(&eval_ctx)?),
        };

        self.cache[idx] = Some(val);
        Ok(())
    }
}
