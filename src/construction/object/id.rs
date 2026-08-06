use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdError {
    #[error("unexpected id variant")]
    UnexpectedVariant,
}

pub trait Id:
    IdFromRaw + IdIntoRaw + Into<ObjectId> + TryFrom<ObjectId, Error = IdError> + Copy
{
}

impl<T> Id for T where
    T: IdFromRaw + IdIntoRaw + Into<ObjectId> + TryFrom<ObjectId, Error = IdError> + Copy
{
}

pub trait IdFromRaw {
    fn from_raw(raw: usize) -> Self;
}

pub trait IdIntoRaw {
    fn into_raw(self) -> usize;
}

impl<T: IdIntoRaw + Copy> IdIntoRaw for &T {
    fn into_raw(self) -> usize {
        (*self).into_raw()
    }
}

impl<T: IdIntoRaw + Copy> IdIntoRaw for &mut T {
    fn into_raw(self) -> usize {
        (*self).into_raw()
    }
}

impl IdFromRaw for PointId {
    fn from_raw(raw: usize) -> Self {
        Self(raw)
    }
}
impl IdFromRaw for LineId {
    fn from_raw(raw: usize) -> Self {
        Self(raw)
    }
}
impl IdFromRaw for CurveControlId {
    fn from_raw(raw: usize) -> Self {
        Self(raw)
    }
}
impl IdFromRaw for CurveId {
    fn from_raw(raw: usize) -> Self {
        Self(raw)
    }
}
impl IdFromRaw for ExpressionId {
    fn from_raw(raw: usize) -> Self {
        Self(raw)
    }
}

impl IdIntoRaw for PointId {
    fn into_raw(self) -> usize {
        self.0
    }
}
impl IdIntoRaw for LineId {
    fn into_raw(self) -> usize {
        self.0
    }
}
impl IdIntoRaw for CurveId {
    fn into_raw(self) -> usize {
        self.0
    }
}
impl IdIntoRaw for CurveControlId {
    fn into_raw(self) -> usize {
        self.0
    }
}
impl IdIntoRaw for ExpressionId {
    fn into_raw(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObjectId {
    Point(PointId),
    Line(LineId),
    Curve(CurveId),
    CurveControl(CurveControlId),
    Expression(ExpressionId),
}

impl ObjectId {
    pub fn into_raw(self) -> usize {
        match self {
            ObjectId::Line(i) => i.into_raw(),
            ObjectId::Point(i) => i.into_raw(),
            ObjectId::CurveControl(i) => i.into_raw(),
            ObjectId::Curve(i) => i.into_raw(),
            ObjectId::Expression(i) => i.into_raw(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PointId(usize);
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LineId(usize);
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CurveId(usize);
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CurveControlId(usize);
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ExpressionId(usize);

impl From<PointId> for ObjectId {
    fn from(value: PointId) -> Self {
        Self::Point(value)
    }
}
impl From<LineId> for ObjectId {
    fn from(value: LineId) -> Self {
        Self::Line(value)
    }
}
impl From<CurveId> for ObjectId {
    fn from(value: CurveId) -> Self {
        Self::Curve(value)
    }
}
impl From<CurveControlId> for ObjectId {
    fn from(value: CurveControlId) -> Self {
        Self::CurveControl(value)
    }
}
impl From<ExpressionId> for ObjectId {
    fn from(value: ExpressionId) -> Self {
        Self::Expression(value)
    }
}

impl TryFrom<ObjectId> for PointId {
    type Error = IdError;
    fn try_from(value: ObjectId) -> Result<Self, Self::Error> {
        match value {
            ObjectId::Point(id) => Ok(id),
            _ => Err(IdError::UnexpectedVariant),
        }
    }
}
impl TryFrom<ObjectId> for LineId {
    type Error = IdError;
    fn try_from(value: ObjectId) -> Result<Self, Self::Error> {
        match value {
            ObjectId::Line(id) => Ok(id),
            _ => Err(IdError::UnexpectedVariant),
        }
    }
}
impl TryFrom<ObjectId> for CurveControlId {
    type Error = IdError;
    fn try_from(value: ObjectId) -> Result<Self, Self::Error> {
        match value {
            ObjectId::CurveControl(id) => Ok(id),
            _ => Err(IdError::UnexpectedVariant),
        }
    }
}
impl TryFrom<ObjectId> for CurveId {
    type Error = IdError;
    fn try_from(value: ObjectId) -> Result<Self, Self::Error> {
        match value {
            ObjectId::Curve(id) => Ok(id),
            _ => Err(IdError::UnexpectedVariant),
        }
    }
}
impl TryFrom<ObjectId> for ExpressionId {
    type Error = IdError;
    fn try_from(value: ObjectId) -> Result<Self, Self::Error> {
        match value {
            ObjectId::Expression(id) => Ok(id),
            _ => Err(IdError::UnexpectedVariant),
        }
    }
}

impl<T: Copy + IdFromRaw + Into<ObjectId>> From<&T> for ObjectId {
    fn from(value: &T) -> Self {
        (*value).into()
    }
}
impl<T: Copy + IdFromRaw + Into<ObjectId>> From<&mut T> for ObjectId {
    fn from(value: &mut T) -> Self {
        (*value).into()
    }
}
