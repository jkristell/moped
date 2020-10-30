use seed::{prelude::*, *};
use crate::Msg;

// ------ ------
//     Init
// ------ ------

pub fn init(url: Url, model: &mut Model) -> Option<()> {
    model.mpdhost = "localhost:6600".to_string();
    Some(())
}

// ------ ------
//     Model
// ------ ------

#[derive(Default)]
pub struct Model {
    pub(crate) mpdhost: String,
}

// ------ ------
//     View
// ------ ------

pub fn view<MS: 'static>(model: &Model) -> Node<MS> {
    div![
        C!["counter"],
        "This is a counter: ",

        input![
            attrs! {
                At::Value => model.mpdhost
            },
            input_ev(Ev::Input, |_| Msg::ServerAddress)
        ],
    ]
}
