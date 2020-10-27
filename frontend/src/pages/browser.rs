use seed::{prelude::*, *};
use crate::Msg;

// ------ ------
//     Init
// ------ ------

pub fn init(url: Url, model: &mut Model) -> Option<()> {
    model.current_dir = "/".into();
    Some(())
}

// ------ ------
//     Model
// ------ ------

#[derive(Default)]
pub struct Model {
    pub(crate) current_dir: String,
    pub(crate) dbdirs: Vec<String>,
}

// ------ ------
//     View
// ------ ------

pub fn view<MS: 'static>(model: &Model) -> Node<MS> {
    let mut current_dir = model.current_dir.clone();

    log!(current_dir);

    if !current_dir.ends_with('/') {
        current_dir.push('/');
    }

    // Folders in the current dir
    let dirs = dirs_in_path(&current_dir, &model.dbdirs);

    let fullpath_display = current_dir.as_str()
        .split('/')
        .filter(|elem| !elem.is_empty())
        .scan(String::new(), |state, part| {
            state.push('/');
            state.push_str(part);
            Some((state.clone(), part.to_string()))
        });

    log!(fullpath_display.clone().collect::<Vec<(String, String)>>());

    div![
        C!["ui one column grid"],
        div![
            C!["column"],
            div![
                C!["ui breadcrumb"],

                a![
                    C!["section"],
                    "Home",
                    ev(Ev::Click, |_| Msg::ChangePath("/".to_string())),
                ],

                fullpath_display
                    .map(|(fullpath, disp)| vec![
                        div![
                            C!["divider"],
                            "/"
                        ],
                        a![
                            C!["section"],
                            disp,
                            ev(Ev::Click, |_| Msg::ChangePath(fullpath)),
                        ],
                    ])
            ],
        ],

        div![
            C!["column"],
            div![
                C!["ui four cards"],
                view_folders(&current_dir, &dirs)
            ]
        ]
    ]
}


fn view_folders<MS: 'static>(current_dir: &str, folders: &[String]) -> Vec<Node<MS>> {
    folders
        .iter()
        .map(|name| view_folder(current_dir, name))
        .collect()
}

// Folder as card
fn view_folder<MS: 'static>(basepath: &str, name: &str) -> Node<MS> {

    let full = format!("{}{}", basepath, name);

    let fp1 = full.to_string();
    let fp2 = fp1.clone();
    let fp3 = fp1.clone();

    div![
        C!["ui card"],
        div![
            C!["image"],
            img![
                attrs!{
                    At::Src => format!("/api/v1/artwork?path={}", full),
                },
                ev(Ev::Click, |_| Msg::PlayqueueAdd(fp1)),
            ],
        ],
        div![
            C!["content"],
            a![
                C!["header"],
                name,
                ev(Ev::Click, |_| Msg::ChangePath(fp2)),
            ]
        ],
        div![
            C!["ui bottom attached button"],
            i![
                C!["add icon"],
            ],
            "Add to playlist",
            ev(Ev::Click, |_| Msg::PlayqueueAdd(fp3)),
        ]
    ]
}



pub fn dirs_in_path(path: &str, dirs: &[String]) -> Vec<String> {

    dirs.iter()
        .filter_map(|s| {
            if s.starts_with(path) {
                let t = &s[path.len()..];
                if !t.contains('/') {
                    Some(t.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}
