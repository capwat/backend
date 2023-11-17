use error_stack::{Context, Report};

use std::borrow::Cow;
use thiserror::Error;
use validator::ValidateError;

#[derive(Debug, Error)]
#[error("Invalid given data occurred")]
pub struct ValidatorReport;

pub trait IntoValidatorReport<T> {
    fn into_validator_report(self) -> error_stack::Result<T, ValidatorReport>;
}

impl<T> IntoValidatorReport<T> for Result<T, ValidateError> {
    fn into_validator_report(self) -> error_stack::Result<T, ValidatorReport> {
        self.map_err(|v| {
            fn read_errors<'a>(
                err: &'a ValidateError,
                fields_queue: &mut Vec<Cow<'a, str>>,
                mut report: Report<ValidatorReport>,
            ) -> Report<ValidatorReport> {
                match err {
                    ValidateError::Fields(fields) => {
                        for (field, data) in fields {
                            fields_queue.push(Cow::Borrowed(field));
                            report = read_errors(data, fields_queue, report);
                        }
                        report
                    }
                    ValidateError::Messages(messages) => {
                        let field_str = fields_queue.join(".");
                        for message in messages {
                            report = report.attach_printable(format!("{field_str}: {message}"));
                        }
                        report
                    }
                    ValidateError::Slice(slice) => {
                        for element in slice {
                            if let Some(element) = element {
                                fields_queue.push(Cow::Owned(element.to_string()));
                                report = read_errors(element, fields_queue, report)
                            }
                        }
                        report
                    }
                }
            }

            let mut queue = Vec::new();
            let report = Report::new(ValidatorReport);
            read_errors(&v, &mut queue, report)
        })
    }
}

// We need to dissect the error of figment so that
// we can get more info on why server configuration
// fails to parse (from a file or environment vars)
pub(crate) trait FigmentErrorAttachable<T: Context> {
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
