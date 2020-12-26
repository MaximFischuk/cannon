pub mod mime_type {
    use mime::Mime;
    use serde::Serializer;

    pub fn serialize<S>(mime: &Mime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mime_string = mime.to_string();
        serializer.serialize_str(mime_string.as_str())
    }
}
