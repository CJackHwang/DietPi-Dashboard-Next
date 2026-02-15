use maud::{Markup, html};
use pretty_bytes_typed::pretty_bytes_binary;
use proto::{backend::ProcessStatus, frontend::SignalAction};
use serde::{Deserialize, Serialize};

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::{Icon, send_act, send_req, template};

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ProcessQuery {
    sort: ColumnSort,
    reverse: bool,
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

fn table_header(name: &str, sort: ColumnSort, query: &ProcessQuery) -> Markup {
    let reverse = if query.sort == sort {
        !query.reverse
    } else {
        false
    };

    let sort_str = serde_plain::to_string(&sort).unwrap();

    html! {
        th data-reverse=(reverse) data-sort=(sort_str) {
            button nm-bind={ "onclick: () => $get('/process')" } {
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

    let query: ProcessQuery = req.extract_query()?;

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

    let sort_str = serde_plain::to_string(&query.sort).unwrap();

    let content = html! {
        section
            #process-swap
            data-reverse=(query.reverse) data-sort=(sort_str)
            nm-bind="oninit: () => $debounce(() => $get('/process'), 2000)"
        {
            h2 { "Processes" }

            table .process-table {
                tr {
                    (table_header("PID", ColumnSort::Pid, &query))
                    (table_header("Name", ColumnSort::Name, &query))
                    (table_header("Status", ColumnSort::Status, &query))
                    (table_header("CPU Usage", ColumnSort::Cpu, &query))
                    (table_header("RAM Usage", ColumnSort::Ram, &query))
                    th { "Actions" }
                }
                @for proc in processes {
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
    };

    template(&req, content, "")
}

pub async fn signal(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let signal: SignalAction = req.extract_query()?;

    send_act!(req, Signal(signal))?;

    Ok(ServerResponse::new())
}
