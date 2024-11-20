use colored::{ColoredString, Colorize};
use std::{fmt, ops::Deref, sync::OnceLock};

pub struct LazyColoredString {
    inner: OnceLock<ColoredString>,
    initializer: fn() -> ColoredString,
}

impl LazyColoredString {
    const fn new(initializer: fn() -> ColoredString) -> Self { LazyColoredString { inner: OnceLock::new(), initializer } }
    fn get(&self) -> &ColoredString { self.inner.get_or_init(self.initializer) }
}

impl Deref for LazyColoredString {
    type Target = ColoredString;
    fn deref(&self) -> &Self::Target { self.get() }
}

impl fmt::Display for LazyColoredString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { fmt::Display::fmt(self.get(), f) }
}

macro_rules! create_symbols {
    ($($name:ident: $style:ident->$text:expr),* $(,)?) => {$(
        pub static $name: LazyColoredString = LazyColoredString::new(|| $text.$style());
    )*};
}

create_symbols! {
    SEP: white->":",
    ARROW: white->"»",
    ADD: green->"+",
    WARN: yellow->"!",
    FAIL: red->"✖",
    OK: green->"✔",
}
