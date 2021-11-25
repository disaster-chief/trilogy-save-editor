mod app;
pub mod components;
mod mass_effect_1;
mod mass_effect_1_le;
mod mass_effect_2;
mod mass_effect_3;
pub mod raw_ui;
pub mod shared;

pub use self::app::*;

use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{self, Display};
use std::ops::Deref;
use std::rc::Rc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use yew::{html, Html};

// RcUi
#[derive(Clone, Default)]
pub struct RcUi<T>(Rc<RefCell<T>>);

impl<T> RcUi<T> {
    pub fn new(inner: T) -> Self {
        RcUi(Rc::new(RefCell::new(inner)))
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        RefCell::borrow(&self.0)
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        RefCell::borrow_mut(&self.0)
    }
}

impl<T> From<T> for RcUi<T> {
    fn from(from: T) -> Self {
        Self::new(from)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for RcUi<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner: T = Deserialize::deserialize(deserializer)?;
        Ok(inner.into())
    }
}

impl<T: Serialize> serde::Serialize for RcUi<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.borrow().serialize(serializer)
    }
}

impl<T> PartialEq for RcUi<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: Display> Display for RcUi<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.borrow().fmt(f)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Theme {
    MassEffect1,
    MassEffect2,
    MassEffect3,
}

impl Deref for Theme {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Theme::MassEffect1 => "mass-effect-1",
            Theme::MassEffect2 => "mass-effect-2",
            Theme::MassEffect3 => "mass-effect-3",
        }
    }
}

impl From<Theme> for yew::Classes {
    fn from(theme: Theme) -> Self {
        theme.to_string().into()
    }
}

pub fn format_code(text: impl AsRef<str>) -> Html {
    let text = text.as_ref().split('`').enumerate().map(|(i, text)| {
        if i % 2 != 0 {
            html! { <span class="bg-default-border px-1 py-px rounded-sm">{ text }</span>}
        } else {
            html! { text }
        }
    });
    html! { for text }
}
