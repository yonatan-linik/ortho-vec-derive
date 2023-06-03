pub trait IntoOrtho {
    type OrthoVec;

    fn into_ortho(self) -> Self::OrthoVec;
}
