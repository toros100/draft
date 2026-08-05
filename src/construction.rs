mod eval;
pub mod expression;
pub mod object;
pub mod value;

use crate::construction::object::*;
use crate::construction::value::*;
use crate::geom::*;
use eval::{Eval, EvalCtx, EvalError};
use expression::ExpressionObj;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::f64::consts::PI;

#[derive(Default)]
pub struct ObjectArena {
    pub v: Vec<Object>,
    pub ids: Vec<ObjectId>,
    pub cache: Vec<Option<Value>>,
    pub id_to_idx: HashMap<ObjectId, usize>,
}

impl ObjectArena {
    // not using TouchArea in slint, because we will need to detect interaction with curves in the
    // future
    // obviously a lot of room to improve here algorithmically
    pub fn hit_scan(&self, cursor_pos: Point2, tolerance: f64) -> Option<ObjectId> {
        fn dist_to(val: &Value, to: Point2) -> f64 {
            match val {
                Value::Point(p) => p.pos.dist(to),
                Value::CurveControl(c) => c.pos.dist(to),
                // HACK:: returning MAX for unimplemented cases for now
                _ => f64::MAX,
            }
        }
        // TODO: need to handle curves, the return type will probably need to be more complex
        // especially for curves. e.g. if its a curve that was hit, we will need to know where on
        // the curve that was and not just the curves id
        let mut dist = f64::MAX;
        let mut id = None;
        for (ev, i) in self.cache.iter().zip(self.ids.iter()) {
            if let Some(e) = ev {
                let d = dist_to(e, cursor_pos);
                if d <= tolerance && d < dist {
                    dist = d;
                    id = Some(*i)
                }
            }
        }
        id
    }

    pub fn drag_to(&mut self, id: ObjectId, target: Point2) {
        // need to decide whether i want this to do nothing or explode if anything fails
        // none of the operations should ever fail, unless something else is very wrong

        let idx = *self.id_to_idx.get(&id).unwrap();
        let obj = &mut (self.v[idx]);
        if let Object::Point(PointObj::Absolute { pos }) = obj {
            *pos = target
        }
        match obj {
            Object::Point(PointObj::Absolute { pos }) => *pos = target,
            Object::CurveControl(CurveControlObj { parent, off }) => {
                let parent_idx = *self.id_to_idx.get(parent).unwrap();
                let parent: &PointVal =
                    self.cache[parent_idx].as_ref().unwrap().try_into().unwrap();
                *off = (target - parent.pos).into()
            }
            _ => {}
        }
    }

    pub fn add_point(&mut self, pos: Point2) -> ObjectId {
        self.push_obj(PointObj::Absolute { pos })
    }

    pub fn add_relative_point(
        &mut self,
        parent: ObjectId,
        dist: ExpressionObj,
        angle: ExpressionObj,
    ) -> ObjectId {
        let dist = self.push_obj(dist);
        let angle = self.push_obj(angle);

        let p = PointObj::DistAngle {
            parent,
            dist,
            angle,
        };
        self.push_obj(p)
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
        let p = CurveControlObj {
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
        };
        self.push_obj(p)
    }

    pub fn add_line(&mut self, from: ObjectId, to: ObjectId) -> ObjectId {
        self.push_obj(LineObj { from, to })
    }

    fn push_obj(&mut self, o: impl Into<Object>) -> ObjectId {
        // WARN: will break when adding removal of objects (reuse of idx/id)
        // also, this should check if the object is "legal" to add:
        // everything it depends on needs to already be pushed
        let idx = self.v.len();
        let id = ObjectId::from_raw(idx);
        self.v.push(o.into());
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
        let dist = self.push_obj(dist);
        self.push_obj(PointObj::OnLine { from, to, dist })
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
    fn try_get(&self, id: impl Borrow<ObjectId>) -> Result<&Value, EvalError> {
        let idx = self
            .id_to_idx
            .get(id.borrow())
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
