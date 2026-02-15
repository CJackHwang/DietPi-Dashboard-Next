use maud::html;
use proto::backend::ServiceStatus;

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::{send_req, template};

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let data = send_req!(req, Services)?;

    let content = html! {
        section {
            h2 { "Services" }
            table {
                tr {
                    th { "Name" }
                    th { "Status" }
                    th { "Error Log" }
                    th { "Start Time" }
                }
                @for service in data.services {
                    @let (status_attr, status_label) = match service.status {
                        ServiceStatus::Active => ("active", "Active"),
                        ServiceStatus::Inactive => ("inactive", "Inactive"),
                        ServiceStatus::Failed => ("failed", "Failed"),
                        ServiceStatus::Unknown => ("unknown", "Unknown"),
                    };
                    tr {
                        td { (service.name) }
                        td {
                            span .status-badge data-status=(status_attr) { (status_label) }
                        }
                        td {
                            @if !service.err_log.is_empty() {
                                details {
                                    summary { "View log" }
                                    pre {
                                        (service.err_log)
                                    }
                                }
                            }
                        }
                        td { (service.start) }
                    }
                }
            }
        }
    };

    template(&req, content, "")
}
