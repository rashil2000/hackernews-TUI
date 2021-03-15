use super::event_view;
use super::search_view;
use super::text_view;
use super::theme::*;
use super::utils::*;
use crate::prelude::*;

/// StoryView is a View displaying a list stories corresponding
/// to a particular category (top stories, newest stories, most popular stories, etc).
pub struct StoryView {
    raw_command: String,
    view: LinearLayout,
    pub stories: Vec<hn_client::Story>,
}

/// Get the description text summarizing basic information about a story
pub fn get_story_text(story: &hn_client::Story) -> StyledString {
    let mut story_text = StyledString::plain(format!("{}", story.title));
    if story.url.len() > 0 {
        story_text.append_styled(format!("\n({})", story.url), ColorStyle::from(LINK_COLOR));
    }
    story_text.append_styled(
        format!(
            "\n{} points | by {} | {} ago | {} comments",
            story.points,
            story.author,
            get_elapsed_time_as_text(story.time),
            story.num_comments,
        ),
        ColorStyle::from(DESC_COLOR),
    );
    story_text
}

impl StoryView {
    pub fn new(stories: Vec<hn_client::Story>) -> Self {
        let view = LinearLayout::vertical().with(|s| {
            stories.iter().enumerate().for_each(|(i, story)| {
                let mut story_text = StyledString::plain(format!("{}. ", i + 1));
                story_text.append(get_story_text(story));
                s.add_child(text_view::TextView::new(story_text));
            })
        });
        StoryView {
            raw_command: String::new(),
            view,
            stories,
        }
    }

    crate::raw_command!();

    inner_getters!(self.view: LinearLayout);
}

impl ViewWrapper for StoryView {
    wrap_impl!(self.view: LinearLayout);
}

/// Return a main view of a StoryView displaying the story list.
/// The main view of a StoryView is a View without status bar or footer.
pub fn get_story_main_view(
    stories: Vec<hn_client::Story>,
    client: &hn_client::HNClient,
) -> impl View {
    event_view::construct_list_event_view(StoryView::new(stories))
        .on_pre_event_inner(Key::Enter, {
            let client = client.clone();
            move |s, _| {
                let id = s.get_inner().get_focus_index();
                // the story struct hasn't had any comments inside yet,
                // so it can be cloned without greatly affecting performance
                let story = s.stories[id].clone();
                Some(EventResult::with_cb({
                    let client = client.clone();
                    move |s| {
                        let async_view = async_view::get_comment_view_async(s, &client, &story);
                        s.pop_layer();
                        s.screen_mut().add_transparent_layer(Layer::new(async_view))
                    }
                }))
            }
        })
        .on_pre_event_inner('O', move |s, _| {
            let id = s.get_inner().get_focus_index();
            let url = &s.stories[id].url;
            if url.len() > 0 {
                match webbrowser::open(url) {
                    Ok(_) => Some(EventResult::Consumed(None)),
                    Err(err) => {
                        warn!("failed to open link {}: {}", url, err);
                        None
                    }
                }
            } else {
                Some(EventResult::Consumed(None))
            }
        })
        .on_pre_event_inner('g', move |s, _| match s.get_raw_command_as_number() {
            Ok(number) => {
                s.clear_raw_command();
                let s = s.get_inner_mut();
                if number == 0 {
                    return None;
                }
                let number = number - 1;
                if number < s.len() {
                    s.set_focus_index(number).unwrap();
                    Some(EventResult::Consumed(None))
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .full_height()
        .scrollable()
}

/// Return a StoryView given a story list and the view description
pub fn get_story_view(
    desc: &str,
    stories: Vec<hn_client::Story>,
    client: &hn_client::HNClient,
) -> impl View {
    let main_view = get_story_main_view(stories, client);
    let mut view = LinearLayout::vertical()
        .child(get_status_bar_with_desc(desc))
        .child(main_view)
        .child(construct_footer_view());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    OnEventView::new(view)
        .on_event(Event::AltChar('s'), {
            let client = client.clone();
            move |s| {
                let cb_sink = s.cb_sink().clone();
                s.pop_layer();
                s.screen_mut()
                    .add_transparent_layer(Layer::new(search_view::get_search_view(
                        &client, cb_sink,
                    )))
            }
        })
        .on_event(Event::AltChar('h'), |s| {
            s.add_layer(StoryView::construct_help_view())
        })
}
