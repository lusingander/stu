use ratatui::{
    style::{Color, Stylize},
    text::Span,
};

use crate::keys::{key_event_to_string, UserEvent, UserEventMapper};

pub struct BuildShortHelpsItem {
    event: BuildShortHelpsItemEvent,
    description: String,
    priority: usize,
}

impl BuildShortHelpsItem {
    pub fn single(event: UserEvent, description: impl Into<String>, priority: usize) -> Self {
        Self {
            event: BuildShortHelpsItemEvent::Single(event),
            description: description.into(),
            priority,
        }
    }

    pub fn group(events: Vec<UserEvent>, description: impl Into<String>, priority: usize) -> Self {
        Self {
            event: BuildShortHelpsItemEvent::Group(events),
            description: description.into(),
            priority,
        }
    }
}

enum BuildShortHelpsItemEvent {
    Single(UserEvent),
    Group(Vec<UserEvent>),
}

pub fn build_short_help_spans(
    helps: Vec<BuildShortHelpsItem>,
    mapper: &UserEventMapper,
) -> Vec<SpansWithPriority> {
    helps
        .into_iter()
        .flat_map(|item| match item.event {
            BuildShortHelpsItemEvent::Single(event) => mapper.find_first_key(event).map(|key| {
                let spans = vec![
                    "<".into(),
                    key_event_to_string(key, true).into(),
                    ">".into(),
                    ": ".into(),
                    item.description.into(),
                ];
                SpansWithPriority {
                    spans,
                    priority: item.priority,
                }
            }),
            BuildShortHelpsItemEvent::Group(events) => {
                let keys: Vec<_> = events
                    .iter()
                    .flat_map(|e| mapper.find_first_key(*e))
                    .collect();
                if keys.is_empty() {
                    None
                } else {
                    let keys_str = keys
                        .into_iter()
                        .map(|k| key_event_to_string(k, true))
                        .collect::<Vec<String>>()
                        .join("/");
                    let spans = vec![
                        "<".into(),
                        keys_str.into(),
                        ">".into(),
                        ": ".into(),
                        item.description.into(),
                    ];
                    Some(SpansWithPriority {
                        spans,
                        priority: item.priority,
                    })
                }
            }
        })
        .collect()
}

#[derive(Debug)]
pub struct SpansWithPriority {
    spans: Vec<Span<'static>>,
    priority: usize,
}

impl SpansWithPriority {
    fn width(&self) -> usize {
        self.spans.iter().map(|s| s.width()).sum::<usize>()
    }
}

pub fn prune_spans_to_fit_width(
    spans_with_priorities: &[SpansWithPriority],
    max_width: usize,
    delimiter: &str,
) -> Vec<Span<'static>> {
    let delimiter_width = console::measure_text_width(delimiter);
    let spans_total_length = spans_with_priorities
        .iter()
        .map(|sp| sp.width())
        .sum::<usize>();
    let delimiter_total_length = spans_with_priorities.len().saturating_sub(1) * delimiter_width;

    let mut total_length = spans_total_length + delimiter_total_length;

    let mut spans_with_priority_with_index: Vec<(usize, &SpansWithPriority)> =
        spans_with_priorities.iter().enumerate().collect();

    spans_with_priority_with_index.sort_by(|(_, sp1), (_, sp2)| sp2.priority.cmp(&sp1.priority));

    let mut prune: Vec<usize> = Vec::new();
    for (i, sp) in &spans_with_priority_with_index {
        if total_length <= max_width {
            break;
        }
        prune.push(*i);
        total_length -= sp.width();
        total_length -= delimiter_width;
    }

    let spans_iter = spans_with_priorities
        .iter()
        .enumerate()
        .filter(|(i, _)| !prune.contains(i))
        .map(|(_, sp)| sp.spans.clone());

    let mut spans = vec![];
    for (i, help) in spans_iter.enumerate() {
        if i > 0 {
            spans.push(Span::raw(delimiter.to_string()));
        }
        spans.extend(help);
    }
    spans
}

pub struct BuildHelpsItem {
    event: UserEvent,
    description: String,
}

impl BuildHelpsItem {
    pub fn new(event: UserEvent, description: impl Into<String>) -> Self {
        Self {
            event,
            description: description.into(),
        }
    }
}

pub fn build_help_spans(
    helps: Vec<BuildHelpsItem>,
    mapper: &UserEventMapper,
    key_fg: Color,
) -> Vec<Spans> {
    helps
        .into_iter()
        .filter_map(|item| {
            let keys = mapper.find_keys(item.event);
            if keys.is_empty() {
                None
            } else {
                let key_spans: Vec<Vec<Span>> = keys
                    .into_iter()
                    .map(|key| {
                        vec![
                            "<".into(),
                            key_event_to_string(key, false).fg(key_fg).bold(),
                            ">".into(),
                        ]
                    })
                    .collect();
                let mut spans = vec![];
                for (i, span) in key_spans.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::raw(" ".to_string()));
                    }
                    spans.extend(span.clone());
                }
                spans.push(Span::raw(": ".to_string()));
                spans.push(Span::raw(item.description));
                Some(Spans::new(spans))
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct Spans {
    spans: Vec<Span<'static>>,
}

impl Spans {
    pub fn new(spans: Vec<Span<'static>>) -> Self {
        Self { spans }
    }

    fn width(&self) -> usize {
        self.spans.iter().map(|s| s.width()).sum::<usize>()
    }
}

pub fn group_spans_to_fit_width(
    spanss: &[Spans],
    max_width: usize,
    delimiter: &str,
) -> Vec<Vec<Span<'static>>> {
    let mut groups: Vec<Vec<Spans>> = Vec::new();
    let mut current_length: usize = 0;
    let mut current_group: Vec<Spans> = Vec::new();
    let delimiter_width = console::measure_text_width(delimiter);
    for spans in spanss {
        if !current_group.is_empty() && current_length + spans.width() > max_width {
            groups.push(current_group);
            current_group = Vec::new();
            current_length = 0;
        }
        current_length += spans.width();
        current_length += delimiter_width;
        current_group.push(spans.clone());
    }
    groups.push(current_group);

    let mut spans = vec![];
    for group in groups {
        let mut group_spans = vec![];
        for (i, help) in group.iter().enumerate() {
            if i > 0 {
                group_spans.push(Span::raw(delimiter.to_string()));
            }
            group_spans.extend(help.spans.clone());
        }
        spans.push(group_spans);
    }
    spans
}
