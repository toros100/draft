mod eval;
pub mod expression;
pub mod object;
pub mod value;

use crate::construction::object::*;
use crate::construction::value::*;
use crate::geom::*;
use eval::{Eval, EvalCtx, EvalError};
use expression::ExpressionObj;
use std::collections::HashMap;
use std::f64::consts::PI;
use thiserror::Error;

#[derive(Default)]
pub struct ObjectArena {
    v: Vec<(Object, ObjectId)>,
    cache: Vec<Option<Value>>,
    raw_id_to_idx: HashMap<usize, usize>,
    dep_scratch: Vec<ObjectId>,
}

#[derive(Error, Debug)]
pub enum DependencyError {
    #[error("missing dependency")]
    Missing(ObjectId),
}

impl ObjectArena {
    pub fn len(&self) -> usize {
        self.v.len()
    }

    pub fn is_empty(&self) -> bool {
        self.v.is_empty()
    }

    pub fn get_object(&self, idx: usize) -> (&Object, ObjectId) {
        let (o, id) = &self.v[idx];
        (o, *id)
    }

    pub fn get_value(&self, idx: usize) -> Option<&Value> {
        self.cache[idx].as_ref()
    }

    fn push_object<A>(&mut self, o: A) -> A::Id
    where
        A: ArenaObject,
    {
        let obj = o.into();

        if let Err(e) = self.check_dependencies(&obj) {
            // TODO: handle properly, this is just for debugging
            panic!("while pushing {:?}: {e}", obj)
        }
        let idx = self.v.len();
        let typed_id = A::Id::from_raw(idx);
        let erased_id = typed_id.into();
        self.v.push((obj, erased_id));
        assert!(
            self.raw_id_to_idx
                .insert(erased_id.into_raw(), idx)
                .is_none()
        );
        self.cache.push(None);
        typed_id
    }

    pub fn check_dependencies(&mut self, o: &Object) -> Result<(), DependencyError> {
        self.dep_scratch.clear();
        o.push_dependencies(&mut self.dep_scratch);
        for &d in self.dep_scratch.iter() {
            if !self.raw_id_to_idx.contains_key(&d.into_raw()) {
                return Err(DependencyError::Missing(d));
            }
        }
        Ok(())
    }

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
        for (ev, (_, i)) in self.cache.iter().zip(self.v.iter()) {
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

        let idx = *self.raw_id_to_idx.get(&id.into_raw()).unwrap();
        let obj = &mut (self.v[idx]);
        if let (Object::Point(PointObj::Root { pos }), _) = obj {
            *pos = target
        }
        match obj {
            (Object::Point(PointObj::Root { pos }), _) => *pos = target,
            (Object::CurveControl(CurveControlObj { parent, off }), _) => {
                let parent_idx = self
                    .raw_id_to_idx
                    .get(&parent.into_raw())
                    .copied()
                    .expect("curve control should have parent");

                let parent: &PointVal = self.cache[parent_idx]
                    .as_ref()
                    .expect("curve control should have value")
                    .try_into()
                    .unwrap();

                *off = (target - parent.pos).into()
            }
            _ => {}
        }
    }

    pub fn add_root(&mut self, pos: Point2) -> PointId {
        self.push_object(PointObj::Root { pos })
    }

    pub fn add_relative_point(
        &mut self,
        parent: PointId,
        dist: ExpressionObj,
        angle: ExpressionObj,
    ) -> PointId {
        let dist = self.push_object(dist);
        let angle = self.push_object(angle);

        let p = PointObj::DistAngle {
            parent,
            dist,
            angle,
        };
        self.push_object(p)
    }

    pub fn add_curve(&mut self, from: PointId, to: PointId) -> CurveId {
        let control_1 = self.add_curve_control(from);
        let control_2 = self.add_curve_control(to);
        let o = CurveObj {
            from,
            to,
            control_1,
            control_2,
        };

        self.push_object(o)
    }

    pub fn add_point_on_curve(&mut self, curve: CurveId, dist: ExpressionObj) -> PointId {
        let dist = self.push_object(dist);
        self.push_object(PointObj::OnCurve { curve, dist })
    }

    pub fn add_point_midway(&mut self, from: PointId, to: PointId) -> PointId {
        // NOTE: could be a primitive instead
        let exp = ExpressionObj::Mul(
            ExpressionObj::Scalar(0.5).into(),
            ExpressionObj::Dist(from, to).into(),
        );
        self.add_point_on_line(from, to, exp)
    }

    pub fn add_point_perpendicular(
        &mut self,
        from: PointId,
        to: PointId,
        dist: ExpressionObj,
    ) -> PointId {
        // NOTE: could be a primitive instead
        let angle = ExpressionObj::Add(
            ExpressionObj::Angle(PI / 2.).into(),
            ExpressionObj::LineAngle(to, from).into(),
        );
        self.add_relative_point(from, dist, angle)
    }

    fn add_curve_control(&mut self, parent: PointId) -> CurveControlId {
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
        self.push_object(p)
    }

    pub fn add_line(&mut self, from: PointId, to: PointId) -> LineId {
        // lines are pretty much cosmetic right now
        // 1. placing a point "on a line" is done without involving an actual line object
        // 2. ExpressionObj::Dist also just measures the distance between two points
        //
        // intersecting with a line could also be done with the implied line/beam between two
        // points? tbd
        self.push_object(LineObj { from, to })
    }

    pub fn add_point_on_line(
        &mut self,
        from: PointId,
        to: PointId,
        dist: ExpressionObj,
    ) -> PointId {
        let dist = self.push_object(dist);
        self.push_object(PointObj::OnLine { from, to, dist })
    }

    // NOTE: could do some kind of dirty tracking and only partial recalculation when something
    // updates. but this seems plenty fast enough for any realistic number of points, and partial
    // recalculation would also require tracking what changed and notifying the slint model of which
    // rows changed.
    pub fn evaluate_all(&mut self) {
        for i in 0..self.v.len() {
            // want this to explode rn
            self.evaluate(i).unwrap();
        }
    }

    fn evaluate(&mut self, idx: usize) -> Result<(), EvalError> {
        let (prev, _) = self.cache.split_at(idx);

        let eval_ctx = PrefixCtx {
            id_to_idx: &self.raw_id_to_idx,
            cached: prev,
        };

        let obj = &self.v[idx];

        let val = match &obj.0 {
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
    id_to_idx: &'a HashMap<usize, usize>,
    cached: &'a [Option<Value>],
}

impl<'a> EvalCtx for PrefixCtx<'a> {
    // maybe a specialized error type would be nice
    // but that is approaching overcooked, this should never even error
    // if everything else is correct
    fn try_get(&self, id: impl Into<ObjectId>) -> Result<&Value, EvalError> {
        let idx = self
            .id_to_idx
            .get(&id.into().into_raw())
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
