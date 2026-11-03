use olivia_window::raw::{KeyCode, MouseButton};

mod sealed {
    use super::{Button, Key, Mouse};

    pub trait Sealed: Copy {
        fn button(self) -> Button;
    }

    impl Sealed for Key {
        fn button(self) -> Button {
            Button::Key(self)
        }
    }

    impl Sealed for Mouse {
        fn button(self) -> Button {
            Button::Mouse(self)
        }
    }
}

pub(crate) use sealed::Sealed;

/// Something that is either down or up: a [`Key`] or a [`Mouse`] button.
pub trait Control: Sealed {}

impl Control for Key {}
impl Control for Mouse {}

/// One [`Control`], whichever kind it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    Key(Key),
    Mouse(Mouse),
}

macro_rules! keys {
    ($($key:ident = $code:ident,)*) => {
        /// A key by its physical position: [`Key::W`] is the key at the QWERTY W position on any
        /// layout.
        #[non_exhaustive]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Key {
            $(
                #[doc = concat!("The key at the QWERTY position of `", stringify!($key), "`.")]
                $key,
            )*
        }

        impl Key {
            pub(crate) fn from_code(code: KeyCode) -> Option<Self> {
                match code {
                    $(KeyCode::$code => Some(Self::$key),)*
                    _ => None,
                }
            }
        }
    };
}

keys! {
    A = KeyA, B = KeyB, C = KeyC, D = KeyD, E = KeyE, F = KeyF, G = KeyG, H = KeyH, I = KeyI,
    J = KeyJ, K = KeyK, L = KeyL, M = KeyM, N = KeyN, O = KeyO, P = KeyP, Q = KeyQ, R = KeyR,
    S = KeyS, T = KeyT, U = KeyU, V = KeyV, W = KeyW, X = KeyX, Y = KeyY, Z = KeyZ,
    Digit0 = Digit0, Digit1 = Digit1, Digit2 = Digit2, Digit3 = Digit3, Digit4 = Digit4,
    Digit5 = Digit5, Digit6 = Digit6, Digit7 = Digit7, Digit8 = Digit8, Digit9 = Digit9,
    Numpad0 = Numpad0, Numpad1 = Numpad1, Numpad2 = Numpad2, Numpad3 = Numpad3,
    Numpad4 = Numpad4, Numpad5 = Numpad5, Numpad6 = Numpad6, Numpad7 = Numpad7,
    Numpad8 = Numpad8, Numpad9 = Numpad9, NumpadAdd = NumpadAdd,
    NumpadSubtract = NumpadSubtract, NumpadMultiply = NumpadMultiply,
    NumpadDivide = NumpadDivide, NumpadDecimal = NumpadDecimal, NumpadEnter = NumpadEnter,
    F1 = F1, F2 = F2, F3 = F3, F4 = F4, F5 = F5, F6 = F6, F7 = F7, F8 = F8, F9 = F9, F10 = F10,
    F11 = F11, F12 = F12,
    Left = ArrowLeft, Right = ArrowRight, Up = ArrowUp, Down = ArrowDown,
    Space = Space, Enter = Enter, Tab = Tab, Backspace = Backspace, Escape = Escape,
    Insert = Insert, Delete = Delete, Home = Home, End = End, PageUp = PageUp,
    PageDown = PageDown,
    ShiftLeft = ShiftLeft, ShiftRight = ShiftRight, ControlLeft = ControlLeft,
    ControlRight = ControlRight, AltLeft = AltLeft, AltRight = AltRight, SuperLeft = SuperLeft,
    SuperRight = SuperRight,
    CapsLock = CapsLock, NumLock = NumLock, ScrollLock = ScrollLock, ContextMenu = ContextMenu,
    PrintScreen = PrintScreen, Pause = Pause,
    Backquote = Backquote, Minus = Minus, Equal = Equal, BracketLeft = BracketLeft,
    BracketRight = BracketRight, Backslash = Backslash, Semicolon = Semicolon, Quote = Quote,
    Comma = Comma, Period = Period, Slash = Slash, IntlBackslash = IntlBackslash,
}

/// A mouse button.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mouse {
    /// The left button.
    Left,
    /// The right button.
    Right,
    /// The middle button, usually the wheel.
    Middle,
    /// The back side button.
    Back,
    /// The forward side button.
    Forward,
}

impl Mouse {
    pub(crate) fn from_button(button: MouseButton) -> Option<Self> {
        match button {
            MouseButton::Left => Some(Self::Left),
            MouseButton::Right => Some(Self::Right),
            MouseButton::Middle => Some(Self::Middle),
            MouseButton::Back => Some(Self::Back),
            MouseButton::Forward => Some(Self::Forward),
            MouseButton::Other(_) => None,
        }
    }
}
