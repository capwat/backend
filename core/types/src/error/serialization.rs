use super::Error;
use serde::Serialize;

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        struct SerializeMessage<'a>(&'a Error);

        impl<'a> std::fmt::Display for SerializeMessage<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0._make_message(f)
            }
        }

        impl<'a> Serialize for SerializeMessage<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.collect_str(self)
            }
        }

        let mut map = serializer.serialize_map(Some(
            2 + if self._has_subcode() { 1 } else { 0 }
                + if self._has_data() { 1 } else { 0 },
        ))?;

        map.serialize_entry("code", &self.code())?;
        if let Some(subcode) = self.subcode() {
            map.serialize_entry("subcode", &subcode)?;
        }
        map.serialize_entry("message", &SerializeMessage(self))?;
        if self._has_data() {
            map.serialize_key("data")?;
            self._serialize_data::<S>(&mut map)?;
        }

        map.end()
    }
}
