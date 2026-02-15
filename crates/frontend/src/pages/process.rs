use maud::{Markup, html};
use pretty_bytes_typed::pretty_bytes_binary;
use proto::{backend::ProcessStatus, frontend::SignalAction};
use serde::{Deserialize, Serialize};

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::{Icon, send_act, send_req, template};

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct ProcessQuery {
    sort: ColumnSort,
    reverse: bool,
    page: usize,
    per_page: usize,
}

impl Default for ProcessQuery {
    fn default() -> Self {
        Self {
            sort: ColumnSort::default(),
            reverse: false,
            page: 1,
            per_page: 25,
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Copy, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ColumnSort {
    #[default]
    Pid,
    Name,
    Status,
    Cpu,
    Ram,
}

fn process_link(query: &ProcessQuery, page: usize, per_page: usize) -> String {
    let mut next = query.clone();
    next.page = page;
    next.per_page = per_page;

    let query = serde_urlencoded::to_string(next).unwrap();

    format!("/process?{query}")
}

fn table_header(name: &str, sort: ColumnSort, query: &ProcessQuery) -> Markup {
    let reverse = if query.sort == sort {
        !query.reverse
    } else {
        false
    };

    let sort_str = serde_plain::to_string(&sort).unwrap();
    let href = format!(
        "/process?sort={sort_str}&reverse={reverse}&page=1&per_page={}",
        query.per_page
    );

    html! {
        th {
            a .sort-link href=(href) {
                (name)
                @if query.sort == sort {
                    @if query.reverse {
                        (Icon::new("fa6-solid-sort-down"))
                    } @else {
                        (Icon::new("fa6-solid-sort-up"))
                    }
                }
            }
        }
    }
}

fn process_status(status: ProcessStatus) -> (&'static str, &'static str) {
    match status {
        ProcessStatus::Running => ("running", "Running"),
        ProcessStatus::Paused => ("paused", "Paused"),
        ProcessStatus::Sleeping => ("sleeping", "Sleeping"),
        ProcessStatus::Other => ("other", "Other"),
    }
}

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let mut query: ProcessQuery = req.extract_query()?;
    query.page = query.page.max(1);
    query.per_page = query.per_page.clamp(15, 120);

    let mut processes = send_req!(req, Processes)?.processes;
    match query.sort {
        ColumnSort::Pid => processes.sort_by_key(|a| a.pid),
        ColumnSort::Name => processes.sort_by(|a, b| a.name.cmp(&b.name)),
        ColumnSort::Status => processes.sort_by_key(|a| a.status),
        ColumnSort::Cpu => processes.sort_by(|a, b| a.cpu.total_cmp(&b.cpu)),
        ColumnSort::Ram => processes.sort_by_key(|a| a.mem),
    }
    if query.reverse {
        processes.reverse();
    }

    let total_items = processes.len();
    let total_pages = total_items.div_ceil(query.per_page).max(1);
    query.page = query.page.min(total_pages);

    let start_idx = (query.page - 1) * query.per_page;
    let end_idx = (start_idx + query.per_page).min(total_items);
    let page_items: Vec<_> = processes
        .into_iter()
        .skip(start_idx)
        .take(query.per_page)
        .collect();

    let shown_start = if total_items == 0 { 0 } else { start_idx + 1 };

    let prev_page = query.page.saturating_sub(1).max(1);
    let next_page = (query.page + 1).min(total_pages);

    let first_link = process_link(&query, 1, query.per_page);
    let prev_link = process_link(&query, prev_page, query.per_page);
    let next_link = process_link(&query, next_page, query.per_page);
    let last_link = process_link(&query, total_pages, query.per_page);

    let content = html! {
        section
            #process-swap
            nm-data="denseRows: localStorage.getItem('processDenseRows') === 'true'"
            nm-bind="oninit: () => $debounce(() => $get(window.location.pathname + window.location.search), 2000)"
        {
            h2 { "Processes" }

            .process-toolbar {
                p .process-summary {
                    "Showing " (shown_start) "-" (end_idx) " of " (total_items) " processes"
                }

                .process-toolbar-actions {
                    button .row-density-btn
                        type="button"
                        nm-bind="
                            onclick: () => {
                                denseRows = !denseRows;
                                localStorage.setItem('processDenseRows', denseRows);
                            }
                        "
                    {
                        "Rows: "
                        span nm-bind="hidden: () => denseRows" { "Comfortable" }
                        span nm-bind="hidden: () => !denseRows" { "Compact" }
                    }

                    .process-page-size {
                        span { "Per Page" }
                        @for page_size in [15_usize, 25, 50, 100] {
                            @let size_link = process_link(&query, 1, page_size);
                            a .page-size-link
                                class=(if query.per_page == page_size { "active" } else { "" })
                                href=(size_link)
                            {
                                (page_size)
                            }
                        }
                    }
                }
            }

            .process-table-wrap {
                table .process-table nm-bind="'class.dense': () => denseRows" {
                    tr {
                        (table_header("PID", ColumnSort::Pid, &query))
                        (table_header("Name", ColumnSort::Name, &query))
                        (table_header("Status", ColumnSort::Status, &query))
                        (table_header("CPU Usage", ColumnSort::Cpu, &query))
                        (table_header("RAM Usage", ColumnSort::Ram, &query))
                        th { "Actions" }
                    }
                    @if page_items.is_empty() {
                        tr {
                            td colspan="6" { "No process data available" }
                        }
                    } @else {
                        @for proc in page_items {
                            @let pretty_mem = pretty_bytes_binary(proc.mem, Some(0));
                            @let (status_attr, status_label) = process_status(proc.status);

                            tr {
                                td { (proc.pid) }
                                td { (proc.name) }
                                td {
                                    span .status-badge data-status=(status_attr) { (status_label) }
                                }
                                td { (format!("{:.1}%", proc.cpu)) }
                                td { (pretty_mem) }
                                td nm-data data-pid=(proc.pid) {
                                    .actions-cell {
                                        button data-signal="kill" title="Kill process" aria-label="Kill process" nm-bind="onclick: () => $post('/process/signal')" {
                                            (Icon::new("fa6-solid-skull"))
                                        }
                                        button data-signal="term" title="Terminate process" aria-label="Terminate process" nm-bind="onclick: () => $post('/process/signal')" {
                                            (Icon::new("fa6-solid-ban"))
                                        }
                                        @if proc.status == ProcessStatus::Paused {
                                            button data-signal="resume" title="Resume process" aria-label="Resume process" nm-bind="onclick: () => $post('/process/signal')" {
                                                (Icon::new("fa6-solid-play"))
                                            }
                                        } @else {
                                            button data-signal="pause" title="Pause process" aria-label="Pause process" nm-bind="onclick: () => $post('/process/signal')" {
                                                (Icon::new("fa6-solid-pause"))
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            .process-pagination {
                a .pager-btn class=(if query.page == 1 { "disabled" } else { "" }) href=(first_link) aria-disabled=(if query.page == 1 { "true" } else { "false" }) { "First" }
                a .pager-btn class=(if query.page == 1 { "disabled" } else { "" }) href=(prev_link) aria-disabled=(if query.page == 1 { "true" } else { "false" }) { "Prev" }
                p .pager-info { "Page " (query.page) " / " (total_pages) }
                a .pager-btn class=(if query.page == total_pages { "disabled" } else { "" }) href=(next_link) aria-disabled=(if query.page == total_pages { "true" } else { "false" }) { "Next" }
                a .pager-btn class=(if query.page == total_pages { "disabled" } else { "" }) href=(last_link) aria-disabled=(if query.page == total_pages { "true" } else { "false" }) { "Last" }
            }
        }
    };

    template(&req, content, "")
}

pub async fn signal(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let signal: SignalAction = req.extract_query()?;

    send_act!(req, Signal(signal))?;

    Ok(ServerResponse::new())
}
