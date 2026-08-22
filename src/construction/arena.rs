use crate::expression::{ExpressionError, Length, Parse, ParseError, Stringify, Symbols};
use std::collections::{HashMap, HashSet};

use crate::construction::*;
use crate::expression::{self, AngleExpression, LengthExpression};
use crate::geom::{self, Point2};

use crate::slint_gen::{ObjectData, ObjectDataUpdate};

// TODO: CLEANUP

pub trait Variant {
    type Id: Into<ObjectId> + From<usize> + Copy;
    type Value: Default;

    fn into_entry(self, id: Self::Id) -> Entry;
    fn eval(&self, dst: &mut Self::Value, ctx: &EvalCtx) -> Result<(), EvalError>;
    fn dependencies(&self, dst: &mut impl Extend<ObjectId>);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, derive_more::From)]
pub enum ObjectId {
    PointFree(PointFreeId),
    PointDistAngle(PointDistAngleId),
    PointOnLine(PointOnLineId),
    PointOnCurve(PointOnCurveId),
    Line(LineId),
    Curve(CurveId),
    CurveControl(CurveControlId),
    LengthVariable(LengthVariableId),
}

impl ObjectId {
    pub fn into_raw(self) -> usize {
        match self {
            ObjectId::PointFree(id) => id.0,
            ObjectId::PointDistAngle(id) => id.0,
            ObjectId::PointOnLine(id) => id.0,
            ObjectId::PointOnCurve(id) => id.0,
            ObjectId::CurveControl(id) => id.0,
            ObjectId::Curve(id) => id.0,
            ObjectId::Line(id) => id.0,
            ObjectId::LengthVariable(id) => id.0,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EvalError {
    #[error("unresolved dependency")]
    UnresolvedDependency,
    #[error("unknown dependency")]
    UnknownDependency,
    #[error("unexpected type")]
    UnexpectedType,
    #[error("expression error: {}", .0)]
    ExpressionError(ExpressionError),
    #[error("illegal dependency")]
    IllegalDependency,
}

// (preparing to implement undo/redo)
#[derive(Debug, Clone)]
pub enum Action {
    Add(Add),
    Update(Update),
    Delete(ObjectId),
}

#[derive(Debug, Clone)]
pub enum Add {
    PointFree(Point2),
    PointOnCurve(CurveId, f64),
    PointOnLine(PointId, PointId, f64),
    PointDistAngle(PointId, LengthExpression, AngleExpression),
    Line(PointId, PointId),
    Curve(PointId, PointId),
}

#[derive(Debug, Clone)]
pub enum Update {
    PointDistAngle {
        target: PointDistAngleId,
        parent: Option<PointId>,
        angle: Option<AngleExpression>,
        dist: Option<LengthExpression>,
        lock_dist: Option<bool>,
        lock_angle: Option<bool>,
    },
    PointFree {
        target: PointFreeId,
        pos: Point2,
    },
    PointOnLine {
        target: PointOnLineId,
        from: Option<PointId>,
        to: Option<PointId>,
        dist: Option<LengthExpression>,
    },
    PointOnCurve {
        target: PointOnCurveId,
        curve: Option<CurveId>,
        dist: Option<LengthExpression>,
    },
    CurveControl {
        target: CurveControlId,
        off: geom::Polar,
    },
}

pub struct EvalCtx<'a> {
    id_to_idx: &'a HashMap<ObjectId, usize>,
    prev_entries: &'a [Entry],
}

impl EvalCtx<'_> {
    pub fn get_entry(&self, id: impl Into<ObjectId>) -> Result<&Entry, EvalError> {
        let idx = *self
            .id_to_idx
            .get(&id.into())
            .ok_or(EvalError::UnknownDependency)?;

        self.prev_entries
            .get(idx)
            .ok_or(EvalError::IllegalDependency)
        // should never actually error in practice
    }

    pub fn get_point_position(&self, id: PointId) -> Result<Point2, EvalError> {
        match self.get_entry(id)? {
            Entry::PointFree(_, _, p) => Ok(p.pos),
            Entry::PointDistAngle(_, _, p) => Ok(p.pos),
            Entry::PointOnLine(_, _, p) => Ok(p.pos),
            Entry::PointOnCurve(_, _, p) => Ok(p.pos),
            _ => Err(EvalError::UnexpectedType),
        }
    }

    pub fn get_line(&self, id: LineId) -> Result<&LineVal, EvalError> {
        match self.get_entry(id)? {
            Entry::Line(_, _, v) => Ok(v),
            _ => Err(EvalError::UnexpectedType),
        }
    }

    pub fn get_curve_control(&self, id: CurveControlId) -> Result<Point2, EvalError> {
        match self.get_entry(id)? {
            Entry::CurveControl(_, _, v) => Ok(v.pos),
            _ => Err(EvalError::UnexpectedType),
        }
    }

    pub fn get_curve(&self, id: CurveId) -> Result<&CurveVal, EvalError> {
        match self.get_entry(id)? {
            Entry::Curve(_, _, c) => Ok(c),
            _ => Err(EvalError::UnexpectedType),
        }
    }

    pub fn get_length_var(&self, id: LengthVariableId) -> Result<Length, EvalError> {
        match self.get_entry(id)? {
            Entry::LengthVariable(_, _, c) => Ok(*c),
            _ => Err(EvalError::UnexpectedType),
        }
    }
}

impl Entry {
    pub fn as_point_pos(&self) -> Option<(PointId, Point2)> {
        match *self {
            Entry::PointDistAngle(id, _, v) => Some((id.into(), v.pos)),
            Entry::PointOnLine(id, _, v) => Some((id.into(), v.pos)),
            Entry::PointOnCurve(id, _, v) => Some((id.into(), v.pos)),
            Entry::PointFree(id, _, v) => Some((id.into(), v.pos)),
            _ => None,
        }
    }

    pub fn geometry_kind(&self) -> Option<GeometryKind> {
        match self {
            Entry::PointFree(..) => Some(GeometryKind::Point),
            Entry::PointDistAngle(..) => Some(GeometryKind::Point),
            Entry::PointOnLine(..) => Some(GeometryKind::Point),
            Entry::PointOnCurve(..) => Some(GeometryKind::Point),
            Entry::CurveControl(..) => Some(GeometryKind::CurveControl),
            Entry::Curve(..) => Some(GeometryKind::Curve),
            Entry::Line(..) => Some(GeometryKind::Line),
            _ => None,
        }
    }

    fn dist_limit(&self, target: Point2, limit: f64) -> Option<f64> {
        match self {
            Entry::PointFree(_, _, val) => val.pos.dist_limit(target, limit),
            Entry::PointDistAngle(_, _, val) => val.pos.dist_limit(target, limit),
            Entry::PointOnLine(_, _, val) => val.pos.dist_limit(target, limit),
            Entry::PointOnCurve(_, _, val) => val.pos.dist_limit(target, limit),
            Entry::CurveControl(_, _, val) => val.pos.dist_limit(target, limit),
            Entry::Line(_, _, val) => {
                let d = geom::closest_point_on_line_segment(val.from, val.to, target)
                    .0
                    .dist(target);
                if d < limit { Some(d) } else { None }
            }
            Entry::Curve(_, _, val) => val.curve.dist_limit(target, limit),
            _ => None,
        }
    }

    pub fn id(&self) -> ObjectId {
        match *self {
            Entry::PointFree(id, ..) => id.into(),
            Entry::PointDistAngle(id, ..) => id.into(),
            Entry::PointOnLine(id, ..) => id.into(),
            Entry::PointOnCurve(id, ..) => id.into(),
            Entry::Line(id, ..) => id.into(),
            Entry::Curve(id, ..) => id.into(),
            Entry::CurveControl(id, ..) => id.into(),
            Entry::LengthVariable(id, ..) => id.into(),
        }
    }

    pub fn dependencies(&self, dst: &mut impl Extend<ObjectId>) {
        match self {
            Entry::PointFree(_, obj, ..) => obj.dependencies(dst),
            Entry::PointDistAngle(_, obj, ..) => obj.dependencies(dst),
            Entry::PointOnLine(_, obj, ..) => obj.dependencies(dst),
            Entry::PointOnCurve(_, obj, ..) => obj.dependencies(dst),
            Entry::Line(_, obj, ..) => obj.dependencies(dst),
            Entry::Curve(_, obj, ..) => obj.dependencies(dst),
            Entry::CurveControl(_, obj, ..) => obj.dependencies(dst),
            Entry::LengthVariable(_, obj, ..) => obj.dependencies(dst),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Entry {
    PointFree(PointFreeId, PointFree, PointFreeVal),
    PointDistAngle(PointDistAngleId, PointDistAngle, PointDistAngleVal),
    PointOnLine(PointOnLineId, PointOnLine, PointOnLineVal),
    PointOnCurve(PointOnCurveId, PointOnCurve, PointOnCurveVal),
    Line(LineId, Line, LineVal),
    Curve(CurveId, Curve, CurveVal),
    CurveControl(CurveControlId, CurveControl, CurveControlVal),
    LengthVariable(LengthVariableId, LengthVariable, Length),
}

#[derive(Default)]
pub struct ObjectArena {
    entries: Vec<Entry>,
    id_to_idx: HashMap<ObjectId, usize>,
    reverse_dep: HashMap<ObjectId, HashSet<ObjectId>>,
    dep_scratch: Vec<ObjectId>,
    next_raw_id: usize,
    // NOTE: maybe do better dirty tracking? could track a max_dirty_idx as well
    // but for a realistic number of entries, its almost certainly fine to just do a full eval each time
    min_dirty_idx: usize,
}

// what a mess
#[derive(Debug, thiserror::Error)]
pub enum ArenaError {
    #[error("missing dependency: {:?}", .0)]
    MissingDependency(ObjectId),
    #[error("eval error: {:?}", .0)]
    EvalError(#[from] EvalError),
    #[error("unexpected entry")]
    UnexpectedEntry,
    #[error("failed to delete: {:?}", .0)]
    DeletionError(ObjectId),
    #[error("no entry for id {:?}", .0)]
    EntryNotFound(ObjectId),
    #[error("entry for id {:?} not evaluated", .0)]
    EntryNotEvaluated(ObjectId),
    #[error("not implemented yet")]
    Unimplemented,
    #[error("failed to parse expression: {}", .0)]
    ParseError(#[from] ParseError),
}

impl ObjectArena {
    pub fn get_data_for(&self, id: ObjectId) -> Result<ObjectData, ArenaError> {
        let idx = *self
            .id_to_idx
            .get(&id)
            .ok_or(ArenaError::EntryNotFound(id))?;

        match &self.entries[idx] {
            #[expect(clippy::needless_update)]
            Entry::PointDistAngle(_, o, _) => Ok(ObjectData {
                str_1: o.dist.inner().stringify(&Symbols {}).into(),
                str_2: o.angle.inner().stringify(&Symbols {}).into(),
                bool_1: o.lock_dist,
                bool_2: o.lock_angle,
                ..Default::default()
            }),
            _ => Err(ArenaError::Unimplemented),
        }
    }

    pub fn try_apply_update(&mut self, update: ObjectDataUpdate) -> Result<(), ArenaError> {
        let id = ObjectId::from(update.id);
        let data = update.data;

        let idx = *self
            .id_to_idx
            .get(&id)
            .ok_or(ArenaError::EntryNotFound(id))?;

        // NOTE: maybe the update kinds should be named types and this translation between the
        // weird union struct i pass from slint should be done outside the arena.
        // should also pass the "original" ObjectData along so i can diff the two versions,
        // to avoid re-parsing the same expression when it did not change (and actually make use of
        // the update fields being Option<_> for partial updates)
        let update = {
            let entry = &self.entries[idx];
            match entry {
                Entry::PointDistAngle(id, ..) => {
                    let dist = LengthExpression::parse(data.str_1, &Symbols {});
                    let angle = AngleExpression::parse(data.str_2, &Symbols {});

                    Update::PointDistAngle {
                        target: *id,
                        dist: Some(dist?),
                        angle: Some(angle?),
                        parent: None,
                        lock_dist: Some(data.bool_1),
                        lock_angle: Some(data.bool_2),
                    }
                }
                _ => return Err(ArenaError::Unimplemented),
            }
        };

        self.apply_action(Action::Update(update))
    }

    pub fn apply_action(&mut self, action: Action) -> Result<(), ArenaError> {
        // WARN: need to do way more here
        match action {
            Action::Add(Add::PointOnCurve(id, t)) => {
                self.add_point_on_curve(id, expression::length(t))?;
            }
            Action::Add(Add::PointFree(pos)) => {
                self.add_point_free(pos)?;
            }
            Action::Add(Add::PointOnLine(from, to, dist)) => {
                self.add_point_on_line(from, to, expression::length(dist))?;
            }
            Action::Add(Add::PointDistAngle(parent, dist, angle)) => {
                self.add_point_dist_angle(parent, dist, angle)?;
            }
            Action::Add(Add::Line(from, to)) => {
                self.add_line(from, to)?;
            }
            Action::Add(Add::Curve(from, to)) => {
                self.add_curve(from, to, geom::Polar::default(), geom::Polar::default())?;
            }
            Action::Update(Update::PointDistAngle {
                target,
                angle,
                dist,
                lock_dist,
                lock_angle,
                ..
            }) => {
                let id = target.into();
                let Some(&idx) = self.id_to_idx.get(&id) else {
                    return Err(ArenaError::EntryNotFound(id));
                };

                let Entry::PointDistAngle(_, p, _) = &mut self.entries[idx] else {
                    return Err(ArenaError::UnexpectedEntry);
                };
                if let Some(angle) = angle {
                    p.angle = angle;
                }
                if let Some(dist) = dist {
                    p.dist = dist;
                }
                if let Some(lock_dist) = lock_dist {
                    p.lock_dist = lock_dist
                }
                if let Some(lock_angle) = lock_angle {
                    p.lock_angle = lock_angle
                }
                self.min_dirty_idx = self.min_dirty_idx.min(idx);
            }
            Action::Update(Update::PointFree { target, pos, .. }) => {
                let id = target.into();
                let Some(&idx) = self.id_to_idx.get(&id) else {
                    return Err(ArenaError::EntryNotFound(id));
                };
                let Entry::PointFree(_, p, _) = &mut self.entries[idx] else {
                    return Err(ArenaError::UnexpectedEntry);
                };

                p.pos = pos;
                self.min_dirty_idx = self.min_dirty_idx.min(idx);
            }
            Action::Update(Update::PointOnLine { target, dist, .. }) => {
                let id = target.into();
                let Some(&idx) = self.id_to_idx.get(&id) else {
                    return Err(ArenaError::EntryNotFound(id));
                };
                let Entry::PointOnLine(_, p, _) = &mut self.entries[idx] else {
                    return Err(ArenaError::UnexpectedEntry);
                };
                if let Some(dist) = dist {
                    // WARN: CHECK DEPENDENCIES
                    p.dist = dist;
                }
                self.min_dirty_idx = self.min_dirty_idx.min(idx);
            }
            Action::Update(Update::PointOnCurve { target, dist, .. }) => {
                let id = target.into();
                let Some(&idx) = self.id_to_idx.get(&id) else {
                    return Err(ArenaError::EntryNotFound(id));
                };
                let Entry::PointOnCurve(_, p, _) = &mut self.entries[idx] else {
                    return Err(ArenaError::UnexpectedEntry);
                };
                if let Some(dist) = dist {
                    // WARN: CHECK DEPENDENCIES
                    p.dist = dist;
                }
                self.min_dirty_idx = self.min_dirty_idx.min(idx);
            }
            Action::Update(Update::CurveControl { target, off, .. }) => {
                let id = target.into();
                let Some(&idx) = self.id_to_idx.get(&id) else {
                    return Err(ArenaError::EntryNotFound(id));
                };
                let Entry::CurveControl(_, p, _) = &mut self.entries[idx] else {
                    return Err(ArenaError::UnexpectedEntry);
                };
                p.off = off;
                self.min_dirty_idx = self.min_dirty_idx.min(idx);
            }
            Action::Delete(id) => self.try_delete(id)?,
        }
        Ok(())
    }

    fn try_push<V>(&mut self, obj: V) -> Result<V::Id, ArenaError>
    where
        V: Variant,
    {
        self.check_dependencies(&obj)?;

        let id = V::Id::from(self.next_raw_id);
        self.next_raw_id = self.next_raw_id.wrapping_add(1);

        let idx = self.entries.len();
        self.entries.push(obj.into_entry(id));

        let obj_id = id.into();

        for d in self.dep_scratch.iter() {
            self.reverse_dep.entry(*d).or_default().insert(obj_id);
        }

        let prev = self.id_to_idx.insert(obj_id, idx);
        self.min_dirty_idx = self.min_dirty_idx.min(idx);

        debug_assert!(prev.is_none());
        debug_assert_eq!(self.entries.len(), self.id_to_idx.len());

        Ok(id)
    }

    fn check_dependencies<V>(&mut self, obj: &V) -> Result<(), ArenaError>
    where
        V: Variant,
    {
        self.dep_scratch.clear();
        obj.dependencies(&mut self.dep_scratch);
        for d in self.dep_scratch.iter() {
            if !self.id_to_idx.contains_key(d) {
                return Err(ArenaError::MissingDependency(*d));
            };
        }
        Ok(())
    }

    pub fn can_delete(&self, id: ObjectId) -> bool {
        self.reverse_dep.get(&id).is_none_or(|s| s.is_empty())
    }

    pub fn try_delete(&mut self, id: ObjectId) -> Result<(), ArenaError> {
        let Some(&idx) = self.id_to_idx.get(&id) else {
            return Err(ArenaError::EntryNotFound(id));
        };

        if let Some(h) = self.reverse_dep.get(&id)
            && !h.is_empty()
        {
            return Err(ArenaError::DeletionError(id));
        }

        self.dep_scratch.clear();
        self.entries[idx].dependencies(&mut self.dep_scratch);

        for d in self.dep_scratch.iter() {
            self.reverse_dep.entry(*d).and_modify(|s| _ = s.remove(&id));
        }

        let e = self.entries.remove(idx);
        self.reverse_dep.remove(&id);
        self.id_to_idx.remove(&id);

        for j in idx..self.entries.len() {
            self.id_to_idx.insert(self.entries[j].id(), j);
        }

        if let Entry::Curve(_, c, _) = e {
            self.try_delete(c.from_control.into()).unwrap();
            self.try_delete(c.to_control.into()).unwrap();
        }

        self.min_dirty_idx = self.min_dirty_idx.min(self.entries.len());

        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.min_dirty_idx < self.entries.len()
    }

    pub fn evaluate_all(&mut self) -> Result<(), ArenaError> {
        for i in self.min_dirty_idx..self.entries.len() {
            let (pref, rest) = self.entries.split_at_mut(i);

            let ctx = &EvalCtx {
                id_to_idx: &self.id_to_idx,
                prev_entries: pref,
            };
            let dst = &mut rest[0];
            match dst {
                Entry::PointFree(_, o, v) => o.eval(v, ctx)?,
                Entry::PointDistAngle(_, o, v) => o.eval(v, ctx)?,
                Entry::PointOnLine(_, o, v) => o.eval(v, ctx)?,
                Entry::PointOnCurve(_, o, v) => o.eval(v, ctx)?,
                Entry::Line(_, o, v) => o.eval(v, ctx)?,
                Entry::Curve(_, o, v) => o.eval(v, ctx)?,
                Entry::CurveControl(_, o, v) => o.eval(v, ctx)?,
                Entry::LengthVariable(_, o, v) => o.eval(v, ctx)?,
            }
            self.min_dirty_idx = i + 1;
        }
        Ok(())
    }

    pub fn iter_evaluated(&self) -> impl Iterator<Item = &Entry> {
        self.entries[..self.min_dirty_idx].iter()
    }

    pub fn hit_scan(&self, target: impl Into<Point2>, limit: f64) -> Option<&Entry> {
        let target = target.into();
        self.iter_evaluated()
            .filter_map(|e| {
                _ = e.dist_limit(target, limit)?;
                Some(e)
            })
            .min_by_key(|e| e.geometry_kind())
        // ordering by kind among things that are within the limit
        // (slightly hacky way to make for example points on a curve take precedence over the
        // curve they are on, even though they have the same distance to the cursor.
        // alternatively, i could give points an actual area with a radius and not just a single
        // geom::Point2)
    }

    pub fn get_entry(&self, id: ObjectId) -> Option<&Entry> {
        // WARN: might not be evaluated
        self.id_to_idx.get(&id).map(|j| &self.entries[*j])
    }

    pub fn add_length_variable(
        &mut self,
        expr: LengthExpression,
    ) -> Result<LengthVariableId, ArenaError> {
        self.try_push(LengthVariable { expr })
    }

    pub fn add_point_on_curve(
        &mut self,
        curve: CurveId,
        dist: LengthExpression,
    ) -> Result<PointOnCurveId, ArenaError> {
        self.try_push(PointOnCurve { curve, dist })
    }

    pub fn add_point_on_line(
        &mut self,
        from: impl Into<PointId>,
        to: impl Into<PointId>,
        dist: LengthExpression,
    ) -> Result<PointOnLineId, ArenaError> {
        self.try_push(PointOnLine {
            from: from.into(),
            to: to.into(),
            dist,
        })
    }

    pub fn add_point_free(&mut self, pos: Point2) -> Result<PointFreeId, ArenaError> {
        self.try_push(PointFree { pos })
    }

    pub fn add_point_dist_angle(
        &mut self,
        parent: impl Into<PointId>,
        dist: LengthExpression,
        angle: AngleExpression,
    ) -> Result<PointDistAngleId, ArenaError> {
        self.try_push(PointDistAngle {
            parent: parent.into(),
            dist,
            angle,
            lock_dist: false,
            lock_angle: false,
        })
    }

    pub fn add_line(
        &mut self,
        from: impl Into<PointId>,
        to: impl Into<PointId>,
    ) -> Result<LineId, ArenaError> {
        let from = from.into();
        let to = to.into();
        self.try_push(Line { from, to })
    }

    pub fn add_curve(
        &mut self,
        from: impl Into<PointId>,
        to: impl Into<PointId>,
        control_1_off: geom::Polar,
        control_2_off: geom::Polar,
    ) -> Result<CurveId, ArenaError> {
        let from = from.into();
        let to = to.into();

        let c1 = CurveControl {
            parent: from,
            off: control_1_off,
        };
        let c2 = CurveControl {
            parent: to,
            off: control_2_off,
        };

        self.check_dependencies(&c1)?;
        self.check_dependencies(&c2)?;

        // NOTE: handle this in a better way
        let c1 = self.try_push(c1).unwrap();
        let c2 = self.try_push(c2).unwrap();

        self.try_push(Curve {
            from,
            to,
            from_control: c1,
            to_control: c2,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum GeometryKind {
    CurveControl,
    Point,
    Curve,
    Line,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom;
    use approx::assert_relative_eq;
    use std::f64::consts::PI;

    #[test]
    fn pythagoras() {
        let mut a = ObjectArena::default();

        let p = a.add_point_free(geom::point2(0., 0.)).unwrap();
        let q = a
            .add_point_dist_angle(p, expression::length(234.), expression::angle(0.))
            .unwrap();
        let r = a
            .add_point_dist_angle(p, expression::length(98.), expression::angle(PI / 2.))
            .unwrap();

        let v = a
            .add_length_variable(expression::dist_between(q.into(), r.into()))
            .unwrap();

        a.evaluate_all().unwrap();

        let entry = a.get_entry(v.into()).unwrap();

        let Entry::LengthVariable(id, _, val) = entry else {
            panic!("unexpected entry variant")
        };

        assert_eq!(*id, v);

        let d = f64::from(*val);

        assert_relative_eq!(d, (234f64.powi(2) + 98f64.powi(2)).sqrt());
    }
}
