use core::fmt;

impl<C> fmt::Debug for crate::Error<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("category", &self.inner.category)
            .field("report", &self.inner.report)
            .field("span", &self.inner.span)
            .finish()
    }
}

impl<C> fmt::Display for crate::Error<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:?}", self.inner.report)
        } else {
            write!(f, "{}: {:#?}", self.inner.category, self.inner.report)
        }
    }
}
