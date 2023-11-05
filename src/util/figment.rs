use error_stack::{Context, Report};

// We need to dissect the error of figment so that
// we can get more info on why server configuration
// fails to parse (from a file or environment vars)
pub trait FigmentErrorAttachable<T: Context> {
    fn attach_figment_error(self, err: figment::Error) -> Report<T>;
}

impl<T: Context> FigmentErrorAttachable<T> for Report<T> {
    fn attach_figment_error(self, e: figment::Error) -> Report<T> {
        let mut this = self.attach_printable(format!("{}", e.kind));

        if let (Some(profile), Some(md)) = (&e.profile, &e.metadata) {
            if !e.path.is_empty() {
                let key = md.interpolate(profile, &e.path);
                this = this.attach_printable(format!("for key {:?}", key));
            }
        }

        if let Some(md) = &e.metadata {
            if let Some(source) = &md.source {
                this = this.attach_printable(format!("in {} {}", source, md.name));
            } else {
                this = this.attach_printable(format!(" in {}", md.name));
            }
        }

        // TODO: Implement chain of errors happening with figment
        // if let Some(prev) = &e.prev {
        //   this = this.attach_printable(prev);
        // }

        this
    }
}
