use core::fmt;

impl<C> fmt::Debug for crate::Error<C> {
    #[allow(unused_mut)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("Error");
        let mut f = f
            .field("category", &self.inner.category)
            .field("report", &self.inner.report);

        #[cfg(feature = "server")]
        {
            f = f.field("span", &self.inner.span);
        }
        f.finish()
    }
}

impl<C> fmt::Display for crate::Error<C> {
    #[cfg(not(feature = "server"))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner.report)
    }

    #[cfg(feature = "server")]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:?}", self.inner.report)
        } else {
            write!(f, "{}: {:#?}", self.inner.category, self.inner.report)
        }
    }
}
