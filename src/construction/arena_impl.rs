use super::*;
use crate::construction::{Object, ObjectId};
use crate::core::*;
use crate::{arena::*, geom};

impl Arena<Object> {
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
        let dist = self.try_push_obj(ExpressionObj::from(dist));
        let angle = self.try_push_obj(ExpressionObj::from(angle));

        let p = PointObj::DistAngle {
            parent,
            dist,
            angle,
        };
        self.try_push_obj(p)
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
        let dist = self.try_push_obj(ExpressionObj::from(dist));
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
        // lines are pretty much cosmetic right now
        // 1. placing a point "on a line" is done without involving an actual line object
        // 2. ExpressionObj::Dist also just measures the distance between two points
        //
        // intersecting with a line could also be done with the implied line/beam between two
        // points? tbd
        self.try_push_obj(LineObj { from, to })
    }

    pub fn add_point_on_line(
        &mut self,
        from: PointId,
        to: PointId,
        dist: LengthExpression,
    ) -> PointId {
        let dist = self.try_push_obj(ExpressionObj::from(dist));
        self.try_push_obj(PointObj::OnLine { from, to, dist })
    }

    pub fn get_value_for<V: Variant<Object>>(&self, id: V::Id) -> Option<&V::Val> {
        let idx = *self.id_to_idx.get(&(id.into()))?;
        V::Val::project(self.vals[idx].as_ref()?)
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

        let exp = ExpressionObj::from(expression::dist_between(q, r));

        let e = a.try_push_obj(exp);

        a.evaluate_all();

        let dist = a
            .get_value_for::<ExpressionObj>(e)
            .expect("should be present");

        let d = dist.try_as_length().expect("should be length variant");

        assert_relative_eq!(d, (234f64.powi(2) + 98f64.powi(2)).sqrt());
    }
}
