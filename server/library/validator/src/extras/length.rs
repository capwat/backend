use crate::HasLength;

#[must_use]
pub fn validate_length<T: HasLength>(
    value: &T,
    min: Option<usize>,
    max: Option<usize>,
    equal: Option<usize>,
) -> bool {
    let length = value.length();
    if let Some(equal) = equal {
        return length == equal;
    } else {
        if let Some(m) = min {
            if length < m {
                return false;
            }
        }
        if let Some(m) = max {
            if length > m {
                return false;
            }
        }
    }

    true
}
