use web_sys::HtmlInputElement;
use yew::{prelude::*, utils::NeqAssign};

use super::CallbackType;
use crate::gui::{components::Helper, RcUi};

#[derive(Clone)]
pub enum NumberType {
    Byte(RcUi<u8>),
    Integer(RcUi<i32>),
    Float(RcUi<f32>),
}

impl PartialEq for NumberType {
    fn eq(&self, other: &NumberType) -> bool {
        match (self, other) {
            (NumberType::Byte(byte), NumberType::Byte(other)) => byte == other,
            (NumberType::Integer(integer), NumberType::Integer(other)) => integer == other,
            (NumberType::Float(float), NumberType::Float(other)) => float == other,
            _ => false,
        }
    }
}

pub enum Msg {
    Change(Event),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub label: String,
    pub value: NumberType,
    pub helper: Option<&'static str>,
    pub onchange: Option<Callback<CallbackType>>,
}

pub struct InputNumber {
    props: Props,
    link: ComponentLink<Self>,
}

impl Component for InputNumber {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        InputNumber { props, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Change(event) => {
                let input: HtmlInputElement = event.target_unchecked_into();
                let value = input.value_as_number();

                if value.is_nan() {
                    return true;
                }

                match self.props.value {
                    NumberType::Byte(ref mut byte) => {
                        let value: u8 = value as u8;
                        *byte.borrow_mut() = value;

                        if let Some(ref callback) = self.props.onchange {
                            callback.emit(CallbackType::Byte(value));
                        }
                    }
                    NumberType::Integer(ref mut integer) => {
                        let value = value as i32;
                        *integer.borrow_mut() = value;

                        if let Some(ref callback) = self.props.onchange {
                            callback.emit(CallbackType::Integer(value));
                        }
                    }
                    NumberType::Float(ref mut float) => {
                        let value = value.clamp(f32::MIN as f64, f32::MAX as f64) as f32;
                        *float.borrow_mut() = value;

                        if let Some(ref callback) = self.props.onchange {
                            callback.emit(CallbackType::Float(value));
                        }
                    }
                }
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let (value, placeholder) = match self.props.value {
            NumberType::Byte(ref byte) => (byte.borrow().to_string(), "<byte>"),
            NumberType::Integer(ref integer) => (integer.borrow().to_string(), "<integer>"),
            NumberType::Float(ref float) => {
                let mut ryu = ryu::Buffer::new();
                (ryu.format(*float.borrow()).trim_end_matches(".0").to_owned(), "<float>")
            }
        };

        let helper = self.props.helper.as_ref().map(|&helper| {
            html! {
                <Helper text={helper} />
            }
        });

        html! {
            <label class="flex items-center gap-1">
                <input type="number" class="input w-[120px]" step="any"
                    {placeholder}
                    {value}
                    onchange={self.link.callback(Msg::Change)}
                />
                { &self.props.label }
                { for helper }
            </label>
        }
    }
}
