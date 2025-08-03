macro_rules! enum_with_matching_struct {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                $variant($variant)
            ),*
        }
    };
}
