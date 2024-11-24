use super::ImfsPathBuf;
use crate::Permissions;

use std::time::SystemTime;

// RwLock is placed here to avoid race conditions with other processes
#[derive(Debug)]
pub enum ImfsEntry {
    File {
        permissions: Permissions,
        times: ImfsEntryTimes,
    },
    Directory {
        children: Vec<ImfsPathBuf>,
        permissions: Permissions,
        times: ImfsEntryTimes,
    },
}

// impl ImfsEntry {
//     #[must_use]
//     pub fn permissions(&self) -> Permissions {
//         match self {
//             Self::Directory { permissions, .. } => *permissions,
//             Self::File { permissions, .. } => *permissions,
//         }
//     }

//     #[must_use]
//     pub fn times(&self) -> ImfsEntryTimes {
//         match self {
//             Self::Directory { times, .. } => *times,
//             Self::File { times, .. } => *times,
//         }
//     }
// }

#[derive(Debug, Clone, Copy)]
pub struct ImfsEntryTimes {
    pub accessed: SystemTime,
    pub modified: SystemTime,
    pub created: SystemTime,
}

impl ImfsEntryTimes {
    #[must_use]
    pub fn now() -> Self {
        Self {
            accessed: SystemTime::now(),
            modified: SystemTime::now(),
            created: SystemTime::now(),
        }
    }
}
