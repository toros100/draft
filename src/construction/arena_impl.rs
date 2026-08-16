use super::*;
use crate::construction::{Object, ObjectId};
use crate::{ObjectKind, TaggedObjectId, core::*};
use crate::{arena::*, geom};

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
    OnCurveAbs {
        curve: CurveId,
        length: LengthExpression,
    },
    // TODO: ...
}

impl Arena<Object> {
    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::DragTo(id, pos) => {
                self.drag_to(id, pos);
            }
            Action::AddPoint(PointDefinition::DistAngle {
                parent,
                dist,
                angle,
            }) => {
                _ = self.add_point_relative(parent, dist, angle);
            }
            Action::AddPoint(PointDefinition::Free { pos }) => _ = self.add_root(pos),
            Action::AddLine(p, q) => {
                _ = self.add_line(p, q);
            }
            Action::AddPoint(PointDefinition::OnLineRel { from, to, frac }) => {
                let exp = expression::dist_between(from, to) * frac;
                self.add_point_on_line(from, to, exp);
            }
            Action::AddPoint(PointDefinition::OnCurveAbs { curve, length }) => {
                self.add_point_on_curve(curve, length);
            }
            Action::AddCurve(p, q) => {
                self.add_curve(p, q);
            }
        }
    }

    // HACK: ...
    pub fn get_tagged_id(&self, id: ObjectId) -> Option<TaggedObjectId> {
        let obj = &self.objs[*self.id_to_idx.get(&id)?].1;
        let kind = match obj {
            Object::Point(PointObj::DistAngle { .. }) => ObjectKind::PointDistAngle,
            Object::Point(PointObj::OnLine { .. }) => ObjectKind::PointOnLine,
            Object::Point(PointObj::OnCurve { .. }) => ObjectKind::PointOnCurve,
            Object::Point(PointObj::Root { .. }) => ObjectKind::PointFree,
            Object::Line(_) => ObjectKind::Line,
            Object::Curve(_) => ObjectKind::Curve,
            Object::CurveControl(_) => ObjectKind::CurveControl,
            _ => None?,
        };

        let raw = usize::from(id) as i32;
        Some(TaggedObjectId { kind, raw })
    }

    pub fn get_object_data(&self, id: ObjectId) -> crate::ObjectDataResponse {
        let obj = &self.objs[*self.id_to_idx.get(&id).unwrap()].1;
        match obj {
            Object::Point(PointObj::DistAngle { dist, angle, .. }) => {
                let mut data = crate::ObjectDataResponse::default();
                let d = self.stringify(dist.inner());
                let a = self.stringify(angle.inner());

                data.ok = true;
                data.id = crate::TaggedObjectId {
                    raw: usize::from(id) as i32,
                    kind: crate::ObjectKind::PointDistAngle,
                };
                data.data.angle = a.into();
                data.data.length = d.into();
                data
            }
            _ => crate::ObjectDataResponse {
                err: "not implemented".into(),
                ..Default::default()
            },
        }
    }

    pub fn apply_object_data(&mut self, update: crate::ObjectDataUpdate) {
        match update.id.kind {
            crate::ObjectKind::PointDistAngle => {
                let new_dist = self.parse::<LengthExpression>(update.data.length);
                let new_angle = self.parse::<AngleExpression>(update.data.angle);
                let id = PointId::from(update.id.raw as usize);
                let obj = self.get_obj_mut::<PointObj>(id).unwrap();
                match obj {
                    PointObj::DistAngle { dist, angle, .. } => {
                        *dist = new_dist;
                        *angle = new_angle;
                    }
                    _ => {
                        println!("PointObj match fell through")
                    }
                }
            }

            _ => {
                println!("fell through: {:?}", update.id)
            }
        }
    }

    pub fn iter_vals(&self) -> impl Iterator<Item = &Option<Value>> {
        self.vals.iter()
    }

    pub fn iter_triples(&self) -> impl Iterator<Item = (ObjectId, &Object, Option<&Value>)> {
        debug_assert_eq!(self.objs.len(), self.vals.len());
        self.objs
            .iter()
            .zip(self.vals.iter())
            .map(|((id, o), v)| (*id, o, v.as_ref()))
    }

    pub fn drag_to(&mut self, id: ObjectId, target: geom::Point2) {
        // need to decide whether i want this to do nothing or explode if anything fails
        // none of the operations should ever fail, unless something else is very wrong

        let idx = *self.id_to_idx.get(&id).unwrap();

        let obj = &mut (self.objs[idx]);
        if let (_, Object::Point(PointObj::Root { pos })) = obj {
            *pos = target
        }
        match obj {
            (_, Object::Point(PointObj::Root { pos })) => *pos = target,
            (_, Object::CurveControl(CurveControlObj { parent, off })) => {
                let parent_idx = self
                    .id_to_idx
                    .get(&ObjectId::from(*parent))
                    .copied()
                    .expect("curve control should have parent");

                let parent: &PointVal = PointVal::project(
                    self.vals[parent_idx]
                        .as_ref()
                        .expect("curve control should have value"),
                )
                .unwrap();

                *off = (target - parent.pos).into()
            }
            _ => {}
        }
    }

    pub fn hit_scan(&self, cursor_pos: Point2, limit: f64) -> Option<ObjectId> {
        fn dist_to(val: &Value, target: Point2, limit: f64) -> Option<f64> {
            match val {
                Value::Point(p) => {
                    let d = p.pos.dist(target);
                    if d >= limit { None } else { Some(d) }
                }
                Value::CurveControl(c) => {
                    let d = c.pos.dist(target);
                    if d >= limit { None } else { Some(d) }
                }
                Value::Curve(c) => c.curve.dist(target, limit),
                Value::Line(l) => {
                    let (p, _) = geom::closest_point_on_line_segment(l.from, l.to, target);
                    let d = p.dist(target);
                    if d >= limit { None } else { Some(d) }
                }
                _ => None,
            }
        }
        // TODO: need to handle curves, the return type will probably need to be more complex
        // especially for curves. e.g. if its a curve that was hit, we will need to know where on
        // the curve that was and not just the curves id
        let mut dist_point = f64::MAX;
        let mut closest_point = None;

        let mut dist_curve = f64::MAX;
        let mut closest_curve = None;

        // HACK: disgusting
        for (ev, (i, _)) in self.vals.iter().zip(self.objs.iter()) {
            if let Some(e) = ev {
                let d = dist_to(e, cursor_pos, limit);
                if let Some(d) = d {
                    match e {
                        Value::Curve(_) => {
                            if d < dist_curve {
                                dist_curve = d;
                                closest_curve = Some(*i)
                            }
                        }
                        _ => {
                            if d < dist_point {
                                dist_point = d;
                                closest_point = Some(*i)
                            }
                        }
                    }
                }
            }
        }

        // HACK: (workaround to make points on curves hoverable)
        // (i do not want to implement real z-order with reordering, maybe a priority-based system would work)
        if closest_point.is_some() {
            closest_point
        } else {
            closest_curve
        }
    }

    pub fn add_root(&mut self, pos: Point2) -> PointId {
        self.try_push_obj(PointObj::Root { pos })
    }

    pub fn add_point_relative(
        &mut self,
        parent: PointId,
        dist: LengthExpression,
        angle: AngleExpression,
    ) -> PointId {
        self.check_expr_dep(dist.inner());
        self.check_expr_dep(angle.inner());

        let p = PointObj::DistAngle {
            parent,
            dist,
            angle,
        };
        self.try_push_obj(p)
    }

    fn check_expr_dep(&mut self, exp: &Expression) {
        self.dep_scratch.clear();
        exp.dependencies(&mut self.dep_scratch);
        for d in self.dep_scratch.iter() {
            assert!(self.id_to_idx.contains_key(d))
        }
    }

    pub fn add_curve(&mut self, from: PointId, to: PointId) -> CurveId {
        let control_1 = self.add_curve_control(from);
        let control_2 = self.add_curve_control(to);
        let o = CurveObj {
            from,
            to,
            from_control: control_1,
            to_control: control_2,
        };

        self.try_push_obj(o)
    }

    pub fn add_point_on_curve(&mut self, curve: CurveId, dist: LengthExpression) -> PointId {
        self.check_expr_dep(dist.inner());
        self.try_push_obj(PointObj::OnCurve { curve, dist })
    }

    pub fn add_point_midway(&mut self, from: PointId, to: PointId) -> PointId {
        // NOTE: could be a primitive instead
        let dist = expression::scalar(0.5) * expression::dist_between(from, to);
        self.add_point_on_line(from, to, dist)
    }

    pub fn add_point_perpendicular(
        &mut self,
        from: PointId,
        to: PointId,
        dist: LengthExpression,
    ) -> PointId {
        // NOTE: could be a primitive instead

        let angle = expression::angle(PI / 2.) + expression::line_angle(from, to);
        self.add_point_relative(from, dist, angle)
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
        self.try_push_obj(p)
    }

    pub fn add_line(&mut self, from: PointId, to: PointId) -> LineId {
        self.try_push_obj(LineObj { from, to })
    }

    pub fn add_point_on_line(
        &mut self,
        from: PointId,
        to: PointId,
        dist: LengthExpression,
    ) -> PointId {
        self.check_expr_dep(dist.inner());
        self.try_push_obj(PointObj::OnLine { from, to, dist })
    }

    pub fn get_value_for<V: Variant<Object>>(&self, id: V::Id) -> Option<&V::Val> {
        let idx = *self.id_to_idx.get(&(id.into()))?;
        V::Val::project(self.vals[idx].as_ref()?)
    }

    pub fn get_obj<V: Variant<Object>>(&self, id: V::Id) -> Option<&V> {
        let idx = *self.id_to_idx.get(&(id.into()))?;
        V::project(&self.objs[idx].1)
    }

    fn get_obj_mut<V: Variant<Object>>(&mut self, id: V::Id) -> Option<&mut V> {
        let idx = *self.id_to_idx.get(&(id.into()))?;
        V::project_mut(&mut self.objs[idx].1)
    }

    pub fn get_obj_gen(&self, id: ObjectId) -> Option<&Object> {
        self.id_to_idx.get(&id).map(|&i| &self.objs[i].1)
    }

    pub fn can_delete(&self, id: ObjectId) -> bool {
        if !self.id_to_idx.contains_key(&id) {
            false
        } else {
            let deps = self.depependents.get(&id).unwrap();
            deps.is_empty()
        }
    }

    pub fn stringify<S>(&self, s: &S) -> String
    where
        S: Stringify,
    {
        s.stringify(self)
    }

    pub fn parse<P>(&self, input: impl AsRef<str>) -> P
    where
        P: Parse,
    {
        P::parse(input, self)
    }

    pub fn delete(&mut self, id: ObjectId) {
        if self.delete_obj(id) {
            self.trim_orphans()
        }
    }

    // HACK: could probably be done more efficiently
    fn delete_obj(&mut self, id: ObjectId) -> bool {
        if !self.can_delete(id) {
            debug_assert!(false, "should not be reached");
            // defensive no-op in release
            return false;
        }

        let Some(&idx) = self.id_to_idx.get(&id) else {
            return false;
        };

        let obj = &self.objs[idx].1;

        // TODO: reuse self.dep_scratch
        let mut v = vec![];

        obj.dependencies_dispatch(&mut v);

        for d in v {
            let deps = self.depependents.get_mut(&d).unwrap();
            debug_assert!(
                deps.remove(&id),
                "dependencies and dependents should be consistent"
            );
        }

        // O(n) but it is what it is
        // can't use swap_remove here
        self.objs.remove(idx);
        self.vals.remove(idx);

        self.id_to_idx.remove(&id).unwrap();
        self.depependents.remove(&id).unwrap();

        for j in idx..self.objs.len() {
            let id = self.objs[j].0;
            let prev_idx = self.id_to_idx.insert(id, j);
            debug_assert_eq!(prev_idx.unwrap(), j + 1);
        }

        true
    }

    fn trim_orphans(&mut self) {
        // orphaned curve controls need to be removed
        for i in (0..self.objs.len()).rev() {
            let (id, obj) = &self.objs[i];
            if matches!(obj, Object::CurveControl(_))
                && self.depependents.get(id).unwrap().is_empty()
            {
                debug_assert!(self.delete_obj(*id), "should have existed and been deleted");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom;
    use approx::assert_relative_eq;

    #[test]
    fn pythagoras() {
        let mut a = Arena::default();

        let p = a.add_root(geom::point2(0., 0.));
        let q = a.add_point_relative(p, expression::length(234.), expression::angle(0.));
        let r = a.add_point_relative(p, expression::length(98.), expression::angle(PI / 2.));

        let obj = VariableObj::Length(expression::dist_between(q, r));

        let e = a.try_push_obj(obj);

        a.evaluate_all();

        let dist = a
            .get_value_for::<VariableObj>(e)
            .expect("should be present");

        let d = dist.val.try_as_length().expect("should be length variant");

        assert_relative_eq!(d, (234f64.powi(2) + 98f64.powi(2)).sqrt());
    }
}
