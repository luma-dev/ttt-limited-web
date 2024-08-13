use leptos::*;
use rand::{seq::SliceRandom, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use std::rc::Rc;
use ttt_limited::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use web_sys::js_sys::Uint8Array;
use web_sys::{window, Response};

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingPreset {
    C3x3Limit3,
    C3x3Limit4,
    C3x3Normal,
    C3x4Limit4,
}
impl SettingPreset {
    fn to_str(self) -> &'static str {
        match self {
            SettingPreset::C3x3Limit3 => "3x3 Limit 3",
            SettingPreset::C3x3Limit4 => "3x3 Limit 4",
            SettingPreset::C3x3Normal => "3x3 Normal",
            SettingPreset::C3x4Limit4 => "3x4 Limit 4",
        }
    }
    fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "3x3 Limit 3" => SettingPreset::C3x3Limit3,
            "3x3 Limit 4" => SettingPreset::C3x3Limit4,
            "3x3 Normal" => SettingPreset::C3x3Normal,
            "3x4 Limit 4" => SettingPreset::C3x4Limit4,
            _ => return None,
        }
        .into()
    }
    fn values() -> Vec<Self> {
        vec![
            SettingPreset::C3x3Limit3,
            SettingPreset::C3x3Limit4,
            SettingPreset::C3x3Normal,
            SettingPreset::C3x4Limit4,
        ]
    }
    fn to_game_setting(self) -> GameSetting {
        match self {
            SettingPreset::C3x3Limit3 => GameSetting::try_new_normal_limited(3, 3).unwrap(),
            SettingPreset::C3x3Limit4 => GameSetting::try_new_normal_limited(3, 4).unwrap(),
            SettingPreset::C3x3Normal => GameSetting::try_new_normal(3).unwrap(),
            SettingPreset::C3x4Limit4 => GameSetting::try_new(3, 4, 3, 4).unwrap(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShowMode {
    Nothing,
    Last,
    All,
}
impl ShowMode {
    fn to_str(self) -> &'static str {
        match self {
            ShowMode::Nothing => "Show Nothing",
            ShowMode::Last => "Show Last",
            ShowMode::All => "Show All",
        }
    }
    fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "Show Nothing" => ShowMode::Nothing,
            "Show Last" => ShowMode::Last,
            "Show All" => ShowMode::All,
            _ => return None,
        }
        .into()
    }
    fn values() -> Vec<Self> {
        vec![ShowMode::Nothing, ShowMode::Last, ShowMode::All]
    }
}

#[component]
pub fn App() -> impl IntoView {
    //let game_setting = GameSetting::try_new_normal_limited(3, 3).unwrap();
    let (setting_preset, set_setting_preset) = create_signal(SettingPreset::C3x3Limit3);
    let game_setting = move || setting_preset().to_game_setting();
    //let game_setting = GameSetting::try_new(3, 4, 3, 4).unwrap();
    let (game, set_game) = create_signal(Game::new(game_setting()));
    //let analysis0 = analyze(game_setting, Default::default(), 1e5 as usize);
    //let analysis1 = move || analyze(game_setting, game().state().clone(), 1e5 as usize);
    //let analysis = create_memo(move |_| analysis0.merge(analysis1()));
    let (analysis, set_analysis) = create_signal::<Option<Rc<AnalysisDictionary>>>(None);
    let (show_hint_first, set_show_hint_first) = create_signal(false);
    let (show_hint_second, set_show_hint_second) = create_signal(false);
    let (mode, set_mode) = create_signal(ShowMode::Last);
    let (highlight_last, set_highlight_last) = create_signal(true);

    let (downloading, set_downloading) = create_signal(false);

    create_effect(move |_| {
        set_game(Game::new(game_setting()));
        set_analysis(None);
        set_downloading(false);
        if matches!(
            setting_preset(),
            SettingPreset::C3x3Limit3 | SettingPreset::C3x3Limit4 | SettingPreset::C3x3Normal
        ) {
            set_analysis(Some(Rc::new(analyze(
                game_setting(),
                Default::default(),
                usize::MAX,
            ))));
        }
    });

    let result_view = move || {
        let r = game().result();
        let s = match r {
            GameResult::FirstWin => "First Win",
            GameResult::SecondWin => "Second Win",
            GameResult::Continue => "",
        };
        view! { <div>{s}</div> }
    };

    let next_player_view = move || {
        let s = if game().is_next_first() {
            "First(O)"
        } else {
            "Second(X)"
        };
        view! { <div>Next: {s}</div> }
    };

    let board_view = move || {
        let get_game = game;
        let game = game();
        let v = game
            .to_cells()
            .into_iter()
            .enumerate()
            .map(|(y, row)| {
                let v = row
                    .into_iter()
                    .enumerate()
                    .map(|(x, cell)| {
                        let take = Take {
                            x: x as u8,
                            y: y as u8,
                        };
                        let is_valid = game.validate_take(take).is_ok();
                        let analysis = analysis().clone();
                        let analysis = {
                            if let Some(analysis) = analysis {
                                let mut game = game.clone();
                                if !game.is_finished() && is_valid {
                                    game.add_take(take);
                                    let game = game.normalize();
                                    let analysis = analysis.analysis().get(game.state()).cloned();
                                    analysis
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        };
                        let analysis_str = if (game.is_next_first() && show_hint_first())
                            || (!game.is_next_first() && show_hint_second())
                        {
                            match analysis.unwrap_or_default() {
                                GameAnalysis::Winning(t) => format!("<L{}>", t),
                                GameAnalysis::Losing(t) => format!("<W{}>", t),
                                GameAnalysis::Neutral => " ".to_string(),
                            }
                        } else {
                            "".to_string()
                        };
                        let format_piece = |is_first: bool, i: usize| {
                            let piece = if is_first { "O" } else { "X" };
                            let num = match mode() {
                                ShowMode::Nothing => "".to_string(),
                                ShowMode::Last => {
                                    if i == 0 && (game.is_next_first() == is_first) {
                                        i.to_string()
                                    } else {
                                        "".to_string()
                                    }
                                }
                                ShowMode::All => i.to_string(),
                            };
                            format!("{}{}", piece, num)
                        };
                        let s = match cell {
                            CellView::None => "".to_string(),
                            CellView::First(i) => format_piece(true, i),
                            CellView::Second(i) => format_piece(false, i),
                        };
                        let is_last = game.is_last_take(take);
                        let base_color = match cell {
                            CellView::First(_) => "blue",
                            CellView::Second(_) => "red",
                            CellView::None => "black",
                        };
                        view! {
                            <button
                                style:width="80px"
                                style:height="80px"
                                style:font-size="22px"
                                style:font-weight="bold"
                                style:color=base_color
                                style:border=if is_last && highlight_last() {
                                    Some("5px solid")
                                } else {
                                    None
                                }
                                style:border-color=if is_last && highlight_last() {
                                    Some(format!("rgba({}, 0.5)", base_color))
                                } else {
                                    None
                                }
                                disabled=game.is_finished() || !is_valid
                                on:click=move |_ev| {
                                    if get_game().validate_take(take).is_ok() {
                                        set_game
                                            .update(|game| {
                                                game.add_take(take);
                                            });
                                    }
                                }
                            >
                                <span>{s}</span>
                                <br />
                                <span>{analysis_str}</span>
                            </button>
                        }
                    })
                    .collect::<Vec<_>>();
                view! { <div style:display="flex">{v}</div> }
            })
            .collect::<Vec<_>>();
        view! { <div>{v}</div> }
    };

    let download_analysis = move || {
        move || {
            set_downloading(true);
            let p = future_to_promise(async move {
                let window = window().unwrap();
                let res = JsFuture::from(window.fetch_with_str("/analyzed_3x4_4.bin"))
                    .await
                    .unwrap();
                let res = res.dyn_into::<Response>().unwrap();
                let buf = JsFuture::from(res.array_buffer().unwrap()).await.unwrap();
                let buf = Uint8Array::new(&buf);
                let buf = buf.to_vec();
                let ad = postcard::from_bytes::<AnalysisDictionary>(&buf).unwrap();
                set_analysis(Some(Rc::new(ad)));
                Ok(JsValue::NULL)
            });
            let _ = p.catch(&Closure::once(move |err: JsValue| {
                logging::log!("err {:?}", err);
                set_downloading(false);
            }));
        }
    };

    let analyzed_view = move || {
        if analysis().is_none() {
            view! {
                <div>
                    <button
                        disabled=downloading
                        on:click=move |_ev| {
                            download_analysis()();
                        }
                    >
                        {"Download Analysis"}
                    </button>
                </div>
            }
        } else {
            view! {
                <div>
                    <div>
                        <button
                            disabled=move || game().is_finished() || analysis().is_none()
                            on:click=move |_ev| {
                                let analysis = analysis().unwrap();
                                let a = analysis.analysis();
                                let mut best_way = GameAnalysis::min();
                                let mut best_takes = vec![];
                                for y in 0..game_setting().board_height() {
                                    for x in 0..game_setting().board_width() {
                                        let take = Take { x: x as u8, y: y as u8 };
                                        let game = game();
                                        if game.validate_take(take).is_ok() {
                                            let mut game = game;
                                            game.add_take(take);
                                            let game = game.normalize();
                                            let analysis = a
                                                .get(game.state())
                                                .cloned()
                                                .unwrap_or_default();
                                            match analysis.cmp(&best_way) {
                                                std::cmp::Ordering::Less => {}
                                                std::cmp::Ordering::Equal => {
                                                    best_takes.push(take);
                                                }
                                                std::cmp::Ordering::Greater => {
                                                    best_way = analysis;
                                                    best_takes.clear();
                                                    best_takes.push(take);
                                                }
                                            }
                                        }
                                    }
                                }
                                let mut seed = [0; 32];
                                window()
                                    .unwrap()
                                    .crypto()
                                    .unwrap()
                                    .get_random_values_with_u8_array(&mut seed)
                                    .unwrap();
                                let mut rng = Xoshiro256PlusPlus::from_seed(seed);
                                let one_best_take = best_takes.choose(&mut rng).copied().unwrap();
                                set_game
                                    .update(|game| {
                                        game.add_take(one_best_take);
                                    });
                            }
                        >
                            {"Take Best"}
                        </button>
                    </div>
                    <div>
                        <label>
                            <input
                                type="checkbox"
                                checked=show_hint_first()
                                on:input=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    set_show_hint_first(checked);
                                }
                            />
                            {"Show Hint First"}
                        </label>
                        <label>
                            <input
                                type="checkbox"
                                checked=show_hint_second()
                                on:input=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    set_show_hint_second(checked);
                                }
                            />
                            {"Show Hint Second"}
                        </label>
                    </div>
                </div>
            }
        }
    };

    view! {
        <div>
            <div
                style:display="inline-flex"
                style:justify-content="space-between"
                style:align-items="baseline"
                style:gap="10px"
            >
                <h1>{"Tic Tac Toe Limited"}</h1>
                <a href="https://github.com/luma-dev/ttt-limited-web" target="_blank">
                    {"Source Code (GitHub)"}
                </a>
            </div>
            <div>{board_view}</div>
            <div>{next_player_view}</div>
            <div>{result_view}</div>
            <div>
                <button on:click=move |_ev| {
                    set_game
                        .update(|game| {
                            *game = Game::new(game_setting());
                        });
                }>{"Reset"}</button>
            </div>
            {analyzed_view}
            <div>
                <label>
                    {"Mode: "}
                    <select
                        value=move || mode().to_str().to_string()
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            let mode = ShowMode::try_from_str(&value).unwrap();
                            set_mode(mode);
                        }
                    >
                        {move || {
                            ShowMode::values()
                                .into_iter()
                                .map(|opt| {
                                    view! {
                                        <option
                                            value=opt.to_str().to_string()
                                            selected=opt == mode()
                                        >
                                            {opt.to_str().to_string()}
                                        </option>
                                    }
                                })
                                .collect::<Vec<_>>()
                        }}
                    </select>
                </label>
            </div>
            <div>
                <label>
                    <input
                        type="checkbox"
                        checked=highlight_last()
                        on:input=move |ev| {
                            let checked = event_target_checked(&ev);
                            set_highlight_last(checked);
                        }
                    />
                    {"Highlight Last"}
                </label>
            </div>
            <div>
                <label>
                    {"Game Preset: "}
                    <select
                        value=move || setting_preset().to_str().to_string()
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            let setting_preset = SettingPreset::try_from_str(&value).unwrap();
                            set_setting_preset(setting_preset);
                        }
                    >
                        {move || {
                            SettingPreset::values()
                                .into_iter()
                                .map(|opt| {
                                    view! {
                                        <option
                                            value=opt.to_str().to_string()
                                            selected=opt == setting_preset()
                                        >
                                            {opt.to_str().to_string()}
                                        </option>
                                    }
                                })
                                .collect::<Vec<_>>()
                        }}
                    </select>
                </label>
            </div>
        </div>
    }
}
