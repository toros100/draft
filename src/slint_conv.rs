use crate::construction::{
    CurveControlId, CurveId, LengthVariableId, LineId, ObjectId, PointDistAngleId, PointFreeId,
    PointOnCurveId, PointOnLineId,
};
use crate::geom::Point2;
use crate::slint_gen;

impl From<slint_gen::TaggedObjectId> for ObjectId {
    fn from(value: slint_gen::TaggedObjectId) -> Self {
        match value.kind {
            slint_gen::ObjectKind::PointFree => PointFreeId::from(value.raw as usize).into(),
            slint_gen::ObjectKind::PointDistAngle => {
                PointDistAngleId::from(value.raw as usize).into()
            }
            slint_gen::ObjectKind::PointOnLine => PointOnLineId::from(value.raw as usize).into(),
            slint_gen::ObjectKind::PointOnCurve => PointOnCurveId::from(value.raw as usize).into(),
            slint_gen::ObjectKind::Line => LineId::from(value.raw as usize).into(),
            slint_gen::ObjectKind::Curve => CurveId::from(value.raw as usize).into(),
            slint_gen::ObjectKind::CurveControl => CurveControlId::from(value.raw as usize).into(),
            slint_gen::ObjectKind::LengthVariable => {
                LengthVariableId::from(value.raw as usize).into()
            }
        }
    }
}

impl From<ObjectId> for slint_gen::TaggedObjectId {
    fn from(value: ObjectId) -> Self {
        match value {
            ObjectId::PointFree(i) => slint_gen::TaggedObjectId {
                kind: slint_gen::ObjectKind::PointFree,
                raw: i.0 as i32,
            },
            ObjectId::PointDistAngle(i) => slint_gen::TaggedObjectId {
                kind: slint_gen::ObjectKind::PointDistAngle,
                raw: i.0 as i32,
            },
            ObjectId::PointOnLine(i) => slint_gen::TaggedObjectId {
                kind: slint_gen::ObjectKind::PointOnLine,
                raw: i.0 as i32,
            },
            ObjectId::PointOnCurve(i) => slint_gen::TaggedObjectId {
                kind: slint_gen::ObjectKind::PointOnCurve,
                raw: i.0 as i32,
            },
            ObjectId::Line(i) => slint_gen::TaggedObjectId {
                kind: slint_gen::ObjectKind::Line,
                raw: i.0 as i32,
            },
            ObjectId::Curve(i) => slint_gen::TaggedObjectId {
                kind: slint_gen::ObjectKind::Curve,
                raw: i.0 as i32,
            },
            ObjectId::CurveControl(i) => slint_gen::TaggedObjectId {
                kind: slint_gen::ObjectKind::CurveControl,
                raw: i.0 as i32,
            },
            ObjectId::LengthVariable(i) => slint_gen::TaggedObjectId {
                kind: slint_gen::ObjectKind::LengthVariable,
                raw: i.0 as i32,
            },
        }
    }
}

impl From<slint_gen::WorldPos> for Point2 {
    fn from(value: slint_gen::WorldPos) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
        }
    }
}

impl From<Point2> for slint_gen::WorldPos {
    fn from(value: Point2) -> Self {
        Self {
            x: value.x as f32,
            y: value.y as f32,
        }
    }
}
