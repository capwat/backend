use std::borrow::Cow;

use error_stack::Report;
use thiserror::Error;
use validator::ValidateError;

#[derive(Debug, Error)]
#[error("Invalid given data occurred")]
pub struct Wrapper;

pub trait IntoValidatorReport<T> {
    fn into_validator_report(self) -> error_stack::Result<T, Wrapper>;
}

impl<T> IntoValidatorReport<T> for Result<T, ValidateError> {
    fn into_validator_report(self) -> error_stack::Result<T, Wrapper> {
        self.map_err(|v| {
            fn read_errors<'a>(
                err: &'a ValidateError,
                fields_queue: &mut Vec<Cow<'a, str>>,
                mut report: Report<Wrapper>,
            ) -> Report<Wrapper> {
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
            let report = Report::new(Wrapper);
            read_errors(&v, &mut queue, report)
        })
    }
}
