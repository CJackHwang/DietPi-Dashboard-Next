use std::time::Duration;

use maud::html;

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::{send_req, template};

use hyper::StatusCode;
use tokio::fs;

async fn read_config() -> Result<String, ServerResponse> {
    let mut cfgpath = std::env::current_exe().map_err(|_| {
        ServerResponse::new()
            .body("failed to get config path")
            .status(StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    cfgpath.set_file_name("config-frontend.toml");

    fs::read_to_string(cfgpath).await.map_err(|_| {
        ServerResponse::new()
            .body("failed to read config")
            .status(StatusCode::INTERNAL_SERVER_ERROR)
    })
}

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let data = send_req!(req, Host)?;
    let frontend_cfg = read_config().await?;
    let backend_cfg = send_req!(req, ReadConfig)?;

    let pretty_time = humantime::format_duration(Duration::from_secs(data.uptime));

    let content = html! {
        section {
            h2 { "Host Information" }

            table .management-table {
                tr {
                    td { "Hostname" }
                    td { (data.hostname) }
                }
                tr {
                    td { "Network Interface" }
                    td { (data.nic) }
                }
                tr {
                    td { "Uptime" }
                    td { (pretty_time) }
                }
                tr {
                    td { "Installed Packages" }
                    td { (data.num_pkgs) }
                }
                tr {
                    td { "OS Version" }
                    td { (data.os_version) }
                }
                tr {
                    td { "Kernel Version" }
                    td { (data.kernel) }
                }
                tr {
                    td { "DietPi Version" }
                    td { (data.dp_version) }
                }
                tr {
                    td { "Architecture" }
                    td { (data.arch) }
                }
            }
        }
        br;
        section {
            h2 { "Frontend Config" }

            pre {
                (frontend_cfg)
            }
        }
        br;
        section {
            h2 { "Backend Config" }

            pre {
                (backend_cfg)
            }
        }
        @if req.config().enable_login {
            br;
            section {
                h2 { "Dashboard Administration" }

                form action="/logout" method="POST" {
                    button .logout { "Logout" }
                }
            }
        }
    };

    template(&req, content, "")
}
