macro_rules! replace_default {
    ($property_type:ty) => {
        <$property_type>::default()
    };
    ($_:ty, $default_value:expr) => {
        $default_value
    };
}

#[cfg(feature = "skins")]
macro_rules! replace_ty {
    ($x:ty) => {
        $x
    };
    ($_:ty, $x:ty) => {
        $x
    };
}

#[cfg(feature = "skins")]
macro_rules! replace_merge {
    ($name:expr, $value:expr) => {
        $name = $value
    };
    ($name:expr, $value:expr, $__:ty) => {
        $name.merge_in($value)
    };
}

macro_rules! define_options {
    (
        $(#[$struct_doc:meta])*
        $([$copy:meta])?
        $struct_name:ident,

        $(#[$opt_struct_doc:meta])*
        $opt_struct_name:ident {
            $(
                $(#[$property_doc:meta])*
                $property_name:ident: $property_type:ty$([$opt_property_type:ty])? $(=> $property_default_value:expr)?
            ),+ $(,)?
        }
    ) => (
        #[derive(Debug, Clone)]
        $(
        #[$struct_doc]
        )*
        pub struct $struct_name {
            $(
            $(
            #[$property_doc]
            )*
            pub $property_name: $property_type,
            )+
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self {
                    $(
                    $property_name: replace_default!($property_type$(, $property_default_value)?),
                    )+
                }
            }
        }

        #[cfg(feature = "skins")]
        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        $(
        #[$opt_struct_doc]
        )*
        pub struct $opt_struct_name {
            $(
            $(
            #[$property_doc]
            )*
            pub $property_name: Option<replace_ty!($property_type$(, $opt_property_type)?)>,
            )+
        }

        #[cfg(feature = "skins")]
        impl From<$opt_struct_name> for $struct_name {
            fn from(opt: $opt_struct_name) -> Self {
                Self {
                $(
                    $property_name: opt.$property_name.map_or_else(
                        ||replace_default!($property_type$(, $property_default_value)?),
                        |v| v.into(),
                    ),
                )+
                }
            }
        }

        #[cfg(feature = "skins")]
        impl From<$struct_name> for $opt_struct_name {
            fn from(value: $struct_name) -> Self {
                Self {
                $(
                    $property_name: Some(value.$property_name.into()),
                )+
                }
            }
        }

        #[cfg(feature = "skins")]
        impl $struct_name {
            /// Merge a partial configuration into a full configuration
            pub fn merge_in(&mut self, opt: $opt_struct_name) {
                $(
                if let Some(value) = opt.$property_name {
                    replace_merge!(self.$property_name, value$(, $opt_property_type)?);
                }
                )+
            }
        }
    )
}
