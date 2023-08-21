#[macro_export]
macro_rules! css_file {
    ($path:literal) => {
        {
            use std::str::FromStr;
            use stylist::StyleSource;
            use stylist::ast::Sheet;
            let content = Sheet::from_str(include_str!($path));
            StyleSource::from(content.unwrap())
        }
    };
}