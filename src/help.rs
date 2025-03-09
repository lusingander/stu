use ratatui::text::Span;

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
