use core::fmt;
use std::io;

use error_stack::{Context, Report};

pub fn install_middleware() {
    crate::Error::install_middleware::<io::Error>(
        #[track_caller]
        |error, location| {
            use std::io::ErrorKind;

            #[derive(Debug)]
            struct IoError;

            impl fmt::Display for IoError {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str("I/O error occurred")
                }
            }
            impl Context for IoError {}

            // sometimes custom becomes its source, we need to trim it (if possible)
            let error_kind = error.kind();
            let is_os_error = error.raw_os_error().is_some();

            let mut report = if error.raw_os_error().is_none() {
                Report::new_without_location(error)
                    .attach_location(Some(location))
                    .erase_context()
            } else {
                Report::new_without_location(error)
                    .attach_location(Some(location))
                    .erase_context()
            };

            let show_io_error = !matches!(
                error_kind,
                ErrorKind::NotFound | ErrorKind::PermissionDenied | ErrorKind::AlreadyExists
            );

            if show_io_error && is_os_error {
                report = report.change_context_retain_location(IoError).as_any();
            }

            report
        },
    );
}
