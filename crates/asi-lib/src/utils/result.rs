pub trait ResultExt<T, E> {
    fn map_err_msg(self, msg: &str) -> Result<T, String>;
    fn ok_or_default(self) -> Option<T>
    where
        T: Default;
}

impl<T, E: std::fmt::Display> ResultExt<T, E> for Result<T, E> {
    fn map_err_msg(self, msg: &str) -> Result<T, String> {
        self.map_err(|e| format!("{}: {}", msg, e))
    }

    fn ok_or_default(self) -> Option<T>
    where
        T: Default,
    {
        self.ok()
    }
}
